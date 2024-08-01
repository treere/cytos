use std::collections::HashMap;

use super::{NodeId, ParamId, Result, Transformer, Value};
use crate::loader::Registry;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct NodeRepr {
    /// Name of the node
    pub name: NodeId,

    /// Type of the node
    #[serde(rename = "type")]
    pub typ: String,

    /// Properties
    #[serde(default)]
    pub props: HashMap<ParamId, Value>,
}

/// A wrapper around a [`Transformer`] keeping trace of the node id.
pub struct Node {
    /// Wrapped transformer.
    transformer: Box<dyn Transformer>,
}

impl std::ops::Deref for Node {
    type Target = dyn Transformer;

    fn deref(&self) -> &Self::Target {
        &*self.transformer
    }
}

impl std::ops::DerefMut for Node {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.transformer
    }
}

impl Node {
    /// Create a new Processor.
    pub fn new(transformer: Box<dyn Transformer>) -> Self {
        Self { transformer }
    }

    pub fn try_from_repr(repr: NodeRepr, loader: &Registry) -> Result<Node> {
        let mut transformer = loader.load(repr.typ.as_str())?;

        for (prop, value) in repr.props {
            transformer.load(prop, value)?;
        }

        Ok(Node::new(transformer))
    }
}
