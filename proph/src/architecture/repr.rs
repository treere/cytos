use std::collections::HashMap;

use serde::Deserialize;

use crate::architecture::Result;

use super::Value;

#[derive(Deserialize, Debug)]
pub struct SystemRepr {
    pub graphs: Vec<GraphRepr>,
}

impl SystemRepr {
    pub fn from_json(file: &str) -> Result<Self> {
        serde_json::from_str(file).or(Err("cannot read file"))
    }
}

/// Graph representatio to be loaded
#[derive(Deserialize, Debug)]
pub struct GraphRepr {
    /// Graph name
    pub name: String,

    /// List of nodes
    pub nodes: Vec<NodeRepr>,

    /// List of links between nodes
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
    Internal(String, String),
    External(String, String, String),
}

/// Link between nodes
#[derive(Deserialize, Debug)]
pub struct Link {
    /// Source node param
    pub src: LinkSource,

    /// Destination node param
    pub dst: (String, String),
}

#[derive(Deserialize, Debug)]
pub struct NodeRepr {
    /// Name of the node
    pub name: String,

    /// Type of the node
    #[serde(rename = "type")]
    pub typ: String,

    /// Properties
    #[serde(default)]
    pub props: HashMap<String, Value>,
}
