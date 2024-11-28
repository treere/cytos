use crate::{
    loader::Registry,
    repr::{GraphLink, GraphRepr, OnError},
};

use super::{node::Node, NodeId, ParamId, Result};

use indexmap::IndexMap;

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
    pub fn from_json(file: &str) -> Result<Self> {
        serde_json::from_str(file).map_err(Into::into)
    }

    /// Convert a [`GraphRepr`] into a [`Graph`]
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
#[derive(Default)]
pub struct Graph {
    /// Processors
    nodes: IndexMap<NodeId, Node>,

    /// OnErrors
    on_errors: IndexMap<NodeId, OnError>,
}

#[no_mangle]
#[inline(never)]
pub fn trace_node_step(_node_id: u64, node: &mut Node) -> Result<()> {
    node.step()
}

impl Graph {
    /// Initialize the nodes
    pub fn initialize(&mut self) -> Result<()> {
        for node in self.nodes.values_mut() {
            node.initialize()?;
        }
        Ok(())
    }

    /// Compute one step of processing
    pub fn step(&mut self) -> Result<StepResult> {
        for (node_id, node) in self.nodes.iter_mut() {
            match trace_node_step(node_id.0, node) {
                Ok(_) => continue,
                Err(x) => match self.on_errors.get(node_id).unwrap_or(&OnError::Fail) {
                    OnError::Skip => return Ok(StepResult::Skip),
                    OnError::Continue => continue,
                    OnError::Fail => return Err(x),
                },
            }
        }

        Ok(StepResult::Done)
    }

    /// Terminate the nodes
    pub fn terminate(&mut self) -> Result<()> {
        for node in self.nodes.values_mut() {
            node.terminate()?;
        }
        Ok(())
    }

    /// List nodes
    pub fn list_nodes(&self) -> Vec<NodeId> {
        self.nodes.keys().copied().collect()
    }

    /// Connects a output data to an input one.
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

    pub fn get_node(&self, node_id: NodeId) -> Result<&Node> {
        self.nodes.get(&node_id).ok_or("missing node".into())
    }

    pub fn get_node_mut(&mut self, node_id: NodeId) -> Result<&mut Node> {
        self.nodes.get_mut(&node_id).ok_or("missing node".into())
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
        assert!(graph
            .get_node_mut(node_id)
            .and_then(|n| n.load(ParamId(0), one))
            .is_ok());

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
