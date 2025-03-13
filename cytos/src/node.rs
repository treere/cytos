use super::{Result, Transformer};
use crate::{loader::Registry, repr::NodeRepr};

/// Node
///
/// A node is only a pointer to a transformer
pub type Node = Box<dyn Transformer>;

impl NodeRepr {
    /// Convert a [`NodeRepr`] into a [`Node`] loading factories from a [`Registry`]
    pub fn into_node(self, loader: &Registry) -> Result<Node> {
        let mut transformer = loader.load(self.typ.as_str())?;

        for (prop, value) in self.props {
            transformer.load(prop, value)?;
        }

        Ok(transformer)
    }
}
