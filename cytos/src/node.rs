use super::{NodeMetadata, Result, Transformer};
use crate::{Stepper, loader::Registry, repr::NodeRepr};

/// A node in the processing graph.
///
/// A node wraps a transformer and provides access to its stepper interface.
/// It uses unsafe pointers for performance when calling step methods.
pub struct Node {
    internal: Box<dyn Transformer>,
    stepper: *mut dyn Stepper,
    factory_name: String,
}

impl Node {
    /// Creates a new node from a transformer.
    ///
    /// # Arguments
    ///
    /// * `internal` - The transformer to wrap in this node.
    /// * `factory_name` - The name of the factory used to create this node.
    pub fn new(mut internal: Box<dyn Transformer + 'static>, factory_name: String) -> Self {
        let vtable = {
            let data: &dyn Stepper = &*internal;

            ptr_meta::metadata(data)
        };
        let stepper = ptr_meta::from_raw_parts_mut::<dyn Stepper>(
            std::ptr::from_mut::<dyn Stepper>(&mut *internal).cast(),
            vtable,
        );

        Self {
            internal,
            stepper,
            factory_name,
        }
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

    /// Gets the metadata for this node.
    ///
    /// # Arguments
    ///
    /// * `registry` - The registry containing factory metadata.
    ///
    /// # Returns
    ///
    /// The node's metadata if available.
    pub fn metadata<'a>(&self, registry: &'a Registry) -> Option<&'a NodeMetadata> {
        registry.get_metadata(&self.factory_name)
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
            if let Some(prop) = transformer.get_prop_mut(prop) {
                prop.load(value)?;
            }
        }

        Ok(Node::new(transformer, self.typ))
    }
}
