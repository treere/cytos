use super::{Result, Transformer};
use crate::{Stepper, loader::Registry, repr::NodeRepr};

/// Node
///
/// A node is only a pointer to a transformer
pub struct Node {
    internal: Box<dyn Transformer>,
    stepper: *mut dyn Stepper,
}

impl Node {
    pub fn new(mut internal: Box<dyn Transformer + 'static>) -> Self {
        let vtable = {
            let data: &dyn Stepper = &*internal;

            ptr_meta::metadata(data)
        };
        let stepper = ptr_meta::from_raw_parts_mut::<dyn Stepper>(
            std::ptr::from_mut::<dyn Stepper>(&mut *internal).cast(),
            vtable,
        );

        Self { internal, stepper }
    }

    pub fn stepper(&mut self) -> &mut dyn Stepper {
        unsafe { &mut *self.stepper }
    }

    pub fn transformer(&self) -> &dyn Transformer {
        &*self.internal
    }
    pub fn transformer_mut(&mut self) -> &mut dyn Transformer {
        &mut *self.internal
    }
}

impl NodeRepr {
    /// Convert a [`NodeRepr`] into a [`Node`] loading factories from a [`Registry`]
    ///
    /// # Errors
    ///
    /// Will return `Err` if the value cannot be loaded
    pub fn into_node(self, loader: &Registry) -> Result<Node> {
        let mut transformer = loader.load(self.typ.as_str())?;

        for (prop, value) in self.props {
            transformer.load(prop, value)?;
        }

        Ok(Node::new(transformer))
    }
}
