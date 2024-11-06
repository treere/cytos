use std::collections::HashMap;

use super::{ParamId, Result, Transformer, Value};
use crate::loader::Registry;
use serde::Deserialize;

/// Node
pub type Node = Box<dyn Transformer>;

/// Node deserializable rapresentation
#[derive(Deserialize, Debug)]
pub struct NodeRepr {
    /// Type of the node
    #[serde(rename = "type")]
    typ: String,

    /// Properties
    #[serde(default)]
    props: HashMap<ParamId, Value>,
}

impl NodeRepr {
    /// Convert a [`NodeRepr`] into a [`Node`] loading factories from a [`Registry`]
    pub fn to_node(self, loader: &Registry) -> Result<Node> {
        let mut transformer = loader.load(self.typ.as_str())?;

        for (prop, value) in self.props {
            transformer.load(prop, value)?;
        }

        Ok(transformer)
    }
}
