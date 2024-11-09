use std::collections::HashMap;

use crate::loader::Registry;

use super::{
    node::{Node, NodeRepr},
    NodeId, ParamId, Result, Value,
};

use indexmap::IndexMap;
use serde::Deserialize;

/// Graph representatio to be loaded
#[derive(Deserialize, Debug)]
pub struct GraphRepr {
    /// List of links between nodes
    #[serde(default)]
    links: Vec<Link>,

    /// Map of nodes with it's id
    nodes: HashMap<NodeId, NodeRepr>,
}

impl GraphRepr {
    /// Load a graph from a file
    pub fn from_json(file: &str) -> Result<Self> {
        serde_json::from_str(file).map_err(Into::into)
    }

    /// Convert a [`GraphRepr`] into a [`Graph`]
    pub fn into_graph(self, loader: &Registry) -> Result<Graph> {
        let mut nodes = IndexMap::default();

        for (node_id, node_repr) in self.nodes {
            let node = node_repr.into_node(loader)?;
            nodes.insert(node_id, node);
        }

        let mut graph = Graph { nodes };

        for Link { src, dst } in self.links {
            graph.internal_link(src, dst)?;
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
}

impl Graph {
    /// Connects a output data to an input one.
    pub fn internal_link(
        &mut self,
        (src_node_id, src_param_id): (NodeId, ParamId),
        (dst_node_id, dst_param_id): (NodeId, ParamId),
    ) -> Result<()> {
        let output = self
            .nodes
            .get(&src_node_id)
            .ok_or("cannot find source")?
            .output(src_param_id)
            .ok_or("cannot find param")?;

        self.nodes
            .get_mut(&dst_node_id)
            .ok_or("cannot find dest")?
            .link(dst_param_id, output)?;

        Ok(())
    }

    /// Load a value into the param of a node
    pub fn load(&mut self, (node_id, param_id): (NodeId, ParamId), value: Value) -> Result<()> {
        self.nodes
            .get_mut(&node_id)
            .ok_or("cannot find node")?
            .load(param_id, value)?;

        Ok(())
    }

    /// Dump the param of a node
    pub fn dumper_for(&self, (node_id, param_id): (NodeId, ParamId)) -> Result<Value> {
        self.nodes
            .get(&node_id)
            .ok_or("cannot find node")?
            .dump(param_id)
    }

    /// Initialize the nodes
    pub fn initialize(&mut self) -> Result<()> {
        for node in self.nodes.values_mut() {
            node.initialize()?;
        }
        Ok(())
    }

    /// Terminate the nodes
    pub fn terminate(&mut self) -> Result<()> {
        for node in self.nodes.values_mut() {
            node.terminate()?;
        }
        Ok(())
    }

    /// Compute one step of processing
    pub fn step(&mut self) -> Result<()> {
        for node in self.nodes.values_mut() {
            node.step()?;
        }

        Ok(())
    }

    /// List nodes
    pub fn list_nodes(&self) -> Vec<NodeId> {
        self.nodes.keys().copied().collect()
    }

    /// List node inputs
    pub fn list_node_inputs(&self, node: NodeId) -> Result<Vec<ParamId>> {
        self.nodes
            .get(&node)
            .map(|n| n.input_names())
            .ok_or("missing node".into())
    }

    /// List node outputs
    pub fn list_node_outputs(&self, node: NodeId) -> Result<Vec<ParamId>> {
        self.nodes
            .get(&node)
            .map(|n| n.output_names())
            .ok_or("missing node".into())
    }
}
