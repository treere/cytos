use std::collections::HashMap;

use super::{ParamId, Result, Stepper, Transformer, Value};
use crate::loader::Registry;
use serde::Deserialize;

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

/// A wrapper around a [`Transformer`] keeping trace of the node id.
pub struct Node {
    /// Wrapped transformer.
    pub transformer: Box<dyn Transformer>,
}

impl Node {
    /// Create a new Processor.
    pub fn new(transformer: Box<dyn Transformer>) -> Self {
        Self { transformer }
    }

    pub fn try_from_repr(repr: NodeRepr, loader: &Registry) -> Result<Node> {
        let mut transformer = loader.load(repr.typ.as_str())?;

        for (prop, value) in repr.props {
            let propid = ParamId::try_from(&prop)?;
            transformer.load(propid, value)?;
        }

        Ok(Node::new(transformer))
    }
}

impl Stepper for Node {
    fn initialize(&mut self) -> Result<()> {
        self.transformer.initialize()
    }

    fn step(&mut self) -> Result<()> {
        self.transformer.step()
    }

    fn terminate(&mut self) -> Result<()> {
        self.transformer.terminate()
    }
}
