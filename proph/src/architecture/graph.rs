use crate::loader::Registry;

use super::{
    node::{Node, NodeRepr},
    GenericOwnedProp, NodeId, ParamId, Result, Value,
};

use indexmap::IndexMap;
use serde::Deserialize;

/// Result of a step
#[derive(Debug)]
pub enum StepResult {
    /// All is ok
    Done,
    /// Is skipping the processing
    Skip,
}

/// Graph behaviour on node failure
#[derive(Debug, Deserialize)]
enum OnError {
    /// Skip this processing
    Skip,

    /// Continue processing the fraph
    Continue,

    /// Forward the error
    Fail,
}

impl Default for OnError {
    fn default() -> Self {
        Self::Fail
    }
}

/// Node depresentation from the braph point
#[derive(Deserialize, Debug)]
pub struct InternalNodeRepr {
    /// Name
    name: NodeId,

    /// Node repr
    #[serde(flatten)]
    node: NodeRepr,

    /// On error expect behaviour
    #[serde(default)]
    on_error: OnError,
}

/// Graph representatio to be loaded
#[derive(Deserialize, Debug)]
pub struct GraphRepr {
    /// List of links between nodes
    #[serde(default)]
    links: Vec<Link>,

    /// Map of nodes with it's id
    nodes: Vec<InternalNodeRepr>,
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

        for Link { src, dst } in self.links {
            graph.link(src, dst)?;
        }
        Ok(graph)
    }
}

/// Link between nodes
#[derive(Deserialize, Debug)]
struct Link {
    /// Source node param
    src: (NodeId, ParamId),

    /// Destination node param
    dst: (NodeId, ParamId),
}

/// Graph.
#[derive(Default)]
pub struct Graph {
    /// Processors
    nodes: IndexMap<NodeId, Node>,

    /// OnErrors
    on_errors: IndexMap<NodeId, OnError>,
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
            match node.step() {
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

    /// Load a value into the param of a node
    pub fn assign(&mut self, (node_id, param_id): (NodeId, ParamId), value: Value) -> Result<()> {
        self.get_node_mut(node_id)?.assign(param_id, value)
    }

    /// Load a value into the param of a node
    pub fn load(&mut self, (node_id, param_id): (NodeId, ParamId), value: Value) -> Result<()> {
        self.get_node_mut(node_id)?.load(param_id, value)
    }

    /// Dump the param of a node
    pub fn dump(&self, (node_id, param_id): (NodeId, ParamId)) -> Result<Value> {
        self.get_node(node_id)?.dump(param_id)
    }

    /// Load an owned value into the param of a node
    pub fn load_owned(
        &mut self,
        (node_id, param_id): (NodeId, ParamId),
        value: GenericOwnedProp,
    ) -> Result<()> {
        self.get_node_mut(node_id)?.load_owned(param_id, value)
    }

    /// Assign an owned value into the param of a node
    pub fn assign_owned(
        &mut self,
        (node_id, param_id): (NodeId, ParamId),
        value: GenericOwnedProp,
    ) -> Result<()> {
        self.get_node_mut(node_id)?.assign_owned(param_id, value)
    }

    /// Dump the param as owned of a node
    pub fn dump_owned(&self, (node_id, param_id): (NodeId, ParamId)) -> Result<GenericOwnedProp> {
        self.get_node(node_id)?.dump_owned(param_id)
    }

    /// List nodes
    pub fn list_nodes(&self) -> Vec<NodeId> {
        self.nodes.keys().copied().collect()
    }

    /// List node inputs
    pub fn list_node_inputs(&self, node_id: NodeId) -> Result<Vec<ParamId>> {
        self.get_node(node_id).map(|n| n.input_names())
    }

    /// List node outputs
    pub fn list_node_outputs(&self, node_id: NodeId) -> Result<Vec<ParamId>> {
        self.get_node(node_id).map(|n| n.output_names())
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

    fn get_node(&self, node_id: NodeId) -> Result<&Node> {
        self.nodes.get(&node_id).ok_or("missing node".into())
    }

    fn get_node_mut(&mut self, node_id: NodeId) -> Result<&mut Node> {
        self.nodes.get_mut(&node_id).ok_or("missing node".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test::{Constant, Empty};

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
        assert_eq!(0, graph.list_node_inputs(node_id).expect("nodes").len());
        assert_eq!(0, graph.list_node_outputs(node_id).expect("nodes").len());
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
            graph.list_node_inputs(node_id).expect("no node")
        );
        assert_eq!(
            vec![ParamId(1)],
            graph.list_node_outputs(node_id).expect("no node")
        );

        let one = Value::load(&1).expect("cannot load");
        assert!(graph.load((node_id, ParamId(0)), one).is_ok());

        assert!(graph.initialize().is_ok());
        assert!(graph.step().is_ok());
        assert!(graph.terminate().is_ok());

        let input: i32 = graph
            .dump((node_id, ParamId(0)))
            .expect("dump")
            .dump()
            .expect("value");

        let output: i32 = graph
            .dump((node_id, ParamId(1)))
            .expect("dump")
            .dump()
            .expect("value");

        assert_eq!(input, 1);
        assert_eq!(output, 1);
    }
}
