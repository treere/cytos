use crate::{
    loader::Registry,
    props::GenericProp,
    repr::{GraphLink, GraphRepr, OnError},
};

use super::{NodeId, ParamId, Result, node::Node};

use indexmap::IndexMap;
use tracing::trace;

/// Result of a step
#[derive(Debug)]
pub enum StepResult {
    /// All is ok
    Done,
    /// Is skipping the processing
    Skip,
}

impl GraphRepr {
    /// Load a graph from a file
    ///
    /// # Errors
    ///
    /// Will return `Err` if `data` is not a valid json
    pub fn from_json(data: &str) -> Result<Self> {
        serde_json::from_str(data).map_err(Into::into)
    }

    /// Convert a [`GraphRepr`] into a [`Graph`]
    ///
    /// # Errors
    ///
    /// Will return `Err` if cannot load one of the nodes
    pub fn into_graph(self, loader: &Registry) -> Result<Graph> {
        let mut nodes = IndexMap::default();
        let mut on_errors = IndexMap::default();

        for node_repr in self.nodes {
            let node = node_repr.node.into_node(loader)?;
            nodes.insert(node_repr.name, node);
            on_errors.insert(node_repr.name, node_repr.on_error);
        }

        let mut graph = Graph { nodes, on_errors };

        for GraphLink { src, dst } in self.links {
            graph.link(src, dst)?;
        }
        Ok(graph)
    }
}

/// Graph.
///
/// A graph is a list of nodes. They are processed in the list sequence.
/// For each node there can be a behaviour that tells what to do in case of an exception
#[derive(Default)]
pub struct Graph {
    /// Nodes
    nodes: IndexMap<NodeId, Node>,

    /// Defines the error handling strategy for each node in case of failure during `step()`.
    on_errors: IndexMap<NodeId, OnError>,
}

/// A frapper arount node step to make it observable
///
/// # Errors
///
/// Will return `Err` if the `node` step returns `Err`
#[unsafe(no_mangle)]
#[inline(never)]
pub extern "Rust" fn trace_node_step(node_id: u64, node: &mut Node) -> Result<()> {
    std::hint::black_box(node_id);
    node.step()
}

impl Graph {
    /// Initialize the nodes
    ///
    /// # Errors
    ///
    /// Will return `Err` if the `node` initialize returns `Err`
    pub fn initialize(&mut self) -> Result<()> {
        trace!("start initialize");
        for node in self.nodes.values_mut() {
            node.initialize()?;
        }

        trace!("end initialize");
        Ok(())
    }

    /// Compute one step of processing
    ///
    /// # Errors
    ///
    /// Will return `Err` if one of the node returns an `Err` and it cannot be handled.
    pub fn step(&mut self) -> Result<StepResult> {
        trace!("start step");
        for (node_id, node) in &mut self.nodes {
            match trace_node_step(node_id.0, node) {
                Ok(()) => (),
                Err(x) => match self.on_errors.get(node_id).unwrap_or(&OnError::Fail) {
                    OnError::Skip => return Ok(StepResult::Skip),
                    OnError::Continue => (),
                    OnError::Fail => return Err(x),
                },
            }
        }
        trace!("end step");
        Ok(StepResult::Done)
    }

    /// Terminate the nodes
    ///
    /// # Errors
    ///
    /// Will return `Err` if one `node` terminate returns `Err`
    pub fn terminate(&mut self) -> Result<()> {
        trace!("start terminate");
        for node in self.nodes.values_mut() {
            node.terminate()?;
        }

        trace!("end terminate");
        Ok(())
    }

    /// Collect the links between nodes
    ///
    /// # Panics
    /// Cannot take a node
    pub fn collect_links(&self) -> Vec<Vec<(NodeId, ParamId)>> {
        self.nodes
            .iter()
            .flat_map(|(n, p)| {
                let output = p
                    .output_names()
                    .into_iter()
                    .map(|q| (p.output(q).unwrap(), (*n, q)));
                let input = p
                    .input_names()
                    .into_iter()
                    .map(|q| (p.input(q).unwrap(), (*n, q)));

                output.chain(input)
            })
            .fold(
                vec![],
                |mut links: Vec<(GenericProp, Vec<(NodeId, ParamId)>)>, (prop, vals)| {
                    if let Some((_, arr)) = links
                        .iter_mut()
                        .find(|(key_prop, _)| key_prop.is_same(&prop))
                    {
                        arr.push(vals);
                    } else {
                        links.push((prop, vec![vals]));
                    }
                    links
                },
            )
            .into_iter()
            .map(|(_, v)| v)
            .collect()
    }

    /// List nodes
    pub fn list_nodes(&self) -> Vec<NodeId> {
        self.nodes.keys().copied().collect()
    }

    /// Connects a output data to an input one.
    ///
    /// # Errors
    ///
    /// Will return `Err` if a node is missing or if the linking process fails
    pub fn link(
        &mut self,
        (src_node_id, src_param_id): (NodeId, ParamId),
        (dst_node_id, dst_param_id): (NodeId, ParamId),
    ) -> Result<()> {
        let output = self
            .get_node(src_node_id)?
            .output(src_param_id)
            .ok_or("cannot find param")?;

        self.get_node_mut(dst_node_id)?.link(dst_param_id, output)?;

        Ok(())
    }

    /// Get a node
    ///
    /// # Errors
    ///
    /// Will return `Err` if a node is missing
    pub fn get_node(&self, node_id: NodeId) -> Result<&Node> {
        self.nodes
            .get(&node_id)
            .ok_or_else(|| format!("missing node {node_id:?}").into())
    }

    /// Get a node as mutable
    ///
    /// # Errors
    ///
    /// Will return `Err` if a node is missing
    pub fn get_node_mut(&mut self, node_id: NodeId) -> Result<&mut Node> {
        self.nodes
            .get_mut(&node_id)
            .ok_or_else(|| format!("missing node {node_id:?}").into())
    }

    /// Remove a node
    ///
    /// # Errors
    ///
    /// Will return `Err` if a node is missing
    pub fn remove(&mut self, node_id: NodeId) -> Result<()> {
        self.nodes
            .shift_remove(&node_id)
            .and(Some(()))
            .ok_or_else(|| format!("missing node {node_id:?}").into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        Value,
        test::{Constant, Empty},
    };

    #[test]
    fn test_default_graph() {
        let g = Graph::default();
        assert_eq!(0, g.list_nodes().len());
    }

    #[test]
    fn test_graph_with_empty_node() {
        let node: Node = Box::new(Empty::default());

        let node_id = NodeId(0);

        let graph = Graph {
            nodes: IndexMap::from_iter(vec![(node_id, node)].into_iter()),
            on_errors: IndexMap::from_iter(vec![(node_id, OnError::Continue)].into_iter()),
        };

        assert_eq!(1, graph.list_nodes().len());
        assert_eq!(
            0,
            graph
                .get_node(node_id)
                .expect("missing node")
                .input_names()
                .len()
        );
        assert_eq!(
            0,
            graph
                .get_node(node_id)
                .expect("missing node")
                .output_names()
                .len()
        );
    }

    #[test]
    fn test_graph_with_constant_node() {
        let node: Node = Box::new(Constant::default());

        let node_id = NodeId(0);

        let mut graph = Graph {
            nodes: IndexMap::from_iter(vec![(node_id, node)].into_iter()),
            on_errors: IndexMap::from_iter(vec![(node_id, OnError::Continue)].into_iter()),
        };

        assert!(graph.initialize().is_ok());
        assert!(graph.step().is_ok());
        assert!(graph.terminate().is_ok());

        assert_eq!(vec![node_id], graph.list_nodes());
        assert_eq!(
            vec![ParamId(0)],
            graph.get_node(node_id).expect("missing node").input_names()
        );
        assert_eq!(
            vec![ParamId(1)],
            graph
                .get_node(node_id)
                .expect("missing node")
                .output_names()
        );

        let one = Value::load(&1).expect("cannot load");
        assert!(
            graph
                .get_node_mut(node_id)
                .and_then(|n| n.load(ParamId(0), one))
                .is_ok()
        );

        assert!(graph.initialize().is_ok());
        assert!(graph.step().is_ok());
        assert!(graph.terminate().is_ok());

        let input: i32 = graph
            .get_node(node_id)
            .expect("missing node")
            .dump(ParamId(0))
            .expect("dump")
            .dump()
            .expect("value");

        let output: i32 = graph
            .get_node(node_id)
            .expect("missing node")
            .dump(ParamId(1))
            .expect("dump")
            .dump()
            .expect("value");

        assert_eq!(input, 1);
        assert_eq!(output, 1);
    }
}
