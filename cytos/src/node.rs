use super::{Result, Transformer};
use crate::{Stepper, loader::Registry, repr::NodeRepr};

/// A node in the processing graph.
///
/// A node wraps a transformer and provides access to its stepper interface.
/// It uses unsafe pointers for performance when calling step methods.
pub struct Node {
    internal: Box<dyn Transformer>,
    stepper: *mut dyn Stepper,
}

impl Node {
    /// Creates a new node from a transformer.
    ///
    /// # Arguments
    ///
    /// * `internal` - The transformer to wrap in this node.
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

    /// Gets a mutable reference to the stepper interface.
    ///
    /// # Returns
    ///
    /// A mutable reference to the stepper trait object.
    pub fn stepper(&mut self) -> &mut dyn Stepper {
        unsafe { &mut *self.stepper }
    }

    /// Gets an immutable reference to the transformer interface.
    ///
    /// # Returns
    ///
    /// An immutable reference to the transformer trait object.
    pub fn transformer(&self) -> &dyn Transformer {
        &*self.internal
    }

    /// Gets a mutable reference to the transformer interface.
    ///
    /// # Returns
    ///
    /// A mutable reference to the transformer trait object.
    pub fn transformer_mut(&mut self) -> &mut dyn Transformer {
        &mut *self.internal
    }
}

impl NodeRepr {
    /// Convert a [`NodeRepr`] into a Node loading factories from a [`Registry`]
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
