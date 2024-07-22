use crate::loader::Registry;

use super::{
    node::{Node, NodeRepr},
    GraphId, NodeId, ParamId, Result, Stepper, Value,
};

use serde::Deserialize;

/// Graph representatio to be loaded
#[derive(Deserialize, Debug)]
pub struct GraphRepr {
    /// Graph name
    pub name: GraphId,

    /// List of nodes
    pub nodes: Vec<NodeRepr>,

    /// List of links between nodes
    #[serde(default)]
    pub links: Vec<Link>,
}

impl GraphRepr {
    pub fn from_json(file: &str) -> Result<Self> {
        serde_json::from_str(file).or(Err("cannot load file"))
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum LinkSource {
    Internal(NodeId, ParamId),
    External(GraphId, NodeId, ParamId),
}

/// Link between nodes
#[derive(Deserialize, Debug)]
pub struct Link {
    /// Source node param
    pub src: LinkSource,

    /// Destination node param
    pub dst: (NodeId, ParamId),
}

/// Graph.
#[derive(Default)]
pub struct Graph {
    /// Processors
    nodes: Vec<(NodeId, Node)>,
}

impl Graph {
    /// Add a processor with a given id to the graph.
    pub fn insert(mut self, id: NodeId, processor: Node) -> Result<Self> {
        if self.nodes.iter().all(|x| x.0 != id) {
            self.nodes.push((id, processor));

            Ok(self)
        } else {
            Err("already exist")
        }
    }

    /// Connects a output data to an input one.
    pub fn internal_link(
        &mut self,
        (src_node_id, src_param_id): (NodeId, ParamId),
        (dst_node_id, dst_param_id): (NodeId, ParamId),
    ) -> Result<()> {
        let output = self
            .nodes
            .iter()
            .find(|p| p.0 == src_node_id)
            .ok_or("cannot find source")?
            .1
            .transformer
            .output(src_param_id)
            .ok_or("cannot find param")?;

        self.nodes
            .iter_mut()
            .find(|p| p.0 == dst_node_id)
            .ok_or("cannot find dest")?
            .1
            .transformer
            .link(dst_param_id, output)?;

        Ok(())
    }

    pub fn load(&mut self, (node_id, param_id): (NodeId, ParamId), value: Value) -> Result<()> {
        self.nodes
            .iter_mut()
            .find(|p| p.0 == node_id)
            .ok_or("cannot find node")?
            .1
            .transformer
            .load(param_id, value)?;

        Ok(())
    }

    pub fn dumper_for(&self, (node_id, param_id): (NodeId, ParamId)) -> Result<Value> {
        self.nodes
            .iter()
            .find(|p| p.0 == node_id)
            .ok_or("cannot find node")?
            .1
            .transformer
            .dump(param_id)
    }

    /// Initialize the nodes
    pub fn initialize(&mut self) -> Result<()> {
        for node in &mut self.nodes {
            node.1.initialize()?;
        }
        Ok(())
    }

    /// Terminate the nodes
    pub fn terminate(&mut self) -> Result<()> {
        for node in &mut self.nodes {
            node.1.terminate()?;
        }
        Ok(())
    }

    /// Compute one step of processing
    pub fn step(&mut self) -> Result<()> {
        for node in &mut self.nodes {
            node.1.step()?;
        }

        Ok(())
    }

    /// List nodes
    pub fn list_nodes(&self) -> Vec<NodeId> {
        self.nodes.iter().map(|x| x.0).collect()
    }

    /// List node inputs
    pub fn list_node_inputs(&self, node: NodeId) -> Result<Vec<ParamId>> {
        self.nodes
            .iter()
            .find(|n| n.0 == node)
            .map(|n| n.1.transformer.input_names())
            .ok_or("missing node")
    }

    /// List node outputs
    pub fn list_node_outputs(&self, node: NodeId) -> Result<Vec<ParamId>> {
        self.nodes
            .iter()
            .find(|n| n.0 == node)
            .map(|n| n.1.transformer.output_names())
            .ok_or("missing node")
    }

    pub fn try_from_repr(repr: GraphRepr, loader: &Registry) -> Result<Graph> {
        let mut graph = Graph::default();
        for node in repr.nodes {
            let id  = node.name;
            let processor = Node::try_from_repr(node, loader)?;
            graph = graph.insert(id, processor)?;
        }

        for Link { src, dst: (d0, d1) } in repr.links {
            match src {
                LinkSource::Internal(s0, s1) => {
                    graph.internal_link((s0, s1), (d0, d1))?;
                }
                LinkSource::External(_g0, _s0, _s1) => {
                    // let g0 = GraphId::try_from(&g0)?;
                    // let s0 = NodeId::try_from(&s0)?;
                    // let s1 = ParamId::try_from(&s1)?;

                    // graph.external_link((g0, s0, s1), (d0, d1))?;
                    todo!();
                }
            }
        }
        Ok(graph)
    }
}
