use std::collections::HashMap;

use crate::architecture::{GraphId, NodeId, ParamId, Value};
use serde::Deserialize;

/// Node deserializable rapresentation
#[derive(Deserialize, Debug)]
pub struct NodeRepr {
    /// Type of the node
    #[serde(rename = "type")]
    pub typ: String,

    /// Properties
    #[serde(default)]
    pub props: HashMap<ParamId, Value>,
}

/// Graph representatio to be loaded
#[derive(Deserialize, Debug)]
pub struct GraphRepr {
    /// List of links between nodes
    #[serde(default)]
    pub links: Vec<GraphLink>,

    /// Map of nodes with it's id
    pub nodes: Vec<InternalNodeRepr>,
}

/// Link between nodes
#[derive(Deserialize, Debug)]
pub struct GraphLink {
    /// Source node param
    pub src: (NodeId, ParamId),

    /// Destination node param
    pub dst: (NodeId, ParamId),
}

/// Node depresentation from the braph point
#[derive(Deserialize, Debug)]
pub struct InternalNodeRepr {
    /// Name
    pub name: NodeId,

    /// Node repr
    #[serde(flatten)]
    pub node: NodeRepr,

    /// On error expect behaviour
    #[serde(default)]
    pub on_error: OnError,
}

/// Graph behaviour on node failure
#[derive(Debug, Deserialize)]
pub enum OnError {
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

/// SystemRepr
///
/// Deserializable System Representation
#[derive(Deserialize, Debug)]
pub struct SystemRepr {
    /// Graphs by name
    #[serde(default)]
    pub graphs: HashMap<GraphId, GraphRepr>,

    #[serde(default)]
    /// Request between graphs
    pub requests: Vec<SystemLink>,

    #[serde(default)]
    /// Send between graphs
    pub sends: Vec<SystemLink>,
}

impl SystemRepr {
    /// Create a SystemRepr loading a file
    pub fn from_json(file: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(file)
    }
}

/// SystemLink between params of different graphs
#[derive(Deserialize, Debug, Clone)]
pub struct SystemLink {
    /// Source node
    pub src: (GraphId, NodeId, ParamId),

    /// Destination node
    pub dst: (GraphId, NodeId, ParamId),
}
