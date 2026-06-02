use super::{NodeMetadata, PropInspector, Result};
use crate::{Stepper, loader::Registry, repr::NodeRepr};

/// A node in the processing graph.
///
/// A node wraps a transformer and provides access to its stepper interface.
/// It uses unsafe pointers for performance when calling step methods.
pub struct Node {
    internal: Box<dyn PropInspector>,
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
    pub fn new(mut internal: Box<dyn PropInspector + 'static>, factory_name: String) -> Self {
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
    pub fn transformer(&self) -> &dyn PropInspector {
        &*self.internal
    }

    /// Gets a mutable reference to the transformer interface.
    ///
    /// # Returns
    ///
    /// A mutable reference to the transformer trait object.
    pub fn transformer_mut(&mut self) -> &mut dyn PropInspector {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ParamId;
    use crate::test::{Constant, Empty};

    #[test]
    fn test_node_creation() {
        let transformer: Box<dyn PropInspector> = Box::new(Empty::default());
        let node = Node::new(transformer, "Empty".to_string());

        // Should be able to get transformer reference
        let _transformer_ref = node.transformer();
    }

    #[test]
    fn test_node_stepper() {
        let transformer: Box<dyn PropInspector> = Box::new(Empty::default());
        let mut node = Node::new(transformer, "Empty".to_string());

        // Should be able to get stepper and call step
        let stepper = node.stepper();
        assert!(stepper.step().is_ok());
    }

    #[test]
    fn test_node_transformer_mut() {
        let transformer: Box<dyn PropInspector> = Box::new(Empty::default());
        let mut node = Node::new(transformer, "Empty".to_string());

        // Should be able to get mutable transformer reference
        let _transformer_mut = node.transformer_mut();
    }

    #[test]
    fn test_node_metadata_from_registry() {
        let mut registry = Registry::default();
        registry.add("Empty", Empty::default);

        let transformer: Box<dyn PropInspector> = Box::new(Empty::default());
        let node = Node::new(transformer, "Empty".to_string());

        // Should be able to get metadata from registry
        let metadata = node.metadata(&registry);
        assert!(metadata.is_some());
        assert_eq!(metadata.unwrap().name, "Empty");
    }

    #[test]
    fn test_node_metadata_not_found() {
        let registry = Registry::default();

        let transformer: Box<dyn PropInspector> = Box::new(Empty::default());
        let node = Node::new(transformer, "NonExistent".to_string());

        // Should return None for unknown factory name
        let metadata = node.metadata(&registry);
        assert!(metadata.is_none());
    }

    #[test]
    fn test_node_with_constant() {
        let transformer: Box<dyn PropInspector> = Box::new(Constant::default());
        let node = Node::new(transformer, "Constant".to_string());

        // Test that we can access the transformer's properties
        let transformer = node.transformer();
        let metadata = transformer.metadata();
        assert_eq!(metadata.name, "Constant");
        assert_eq!(metadata.params.len(), 2);
    }

    #[test]
    fn test_node_stepper_lifecycle() {
        let transformer: Box<dyn PropInspector> = Box::new(Constant::default());
        let mut node = Node::new(transformer, "Constant".to_string());

        // Test full lifecycle
        let stepper = node.stepper();
        assert!(stepper.initialize().is_ok());
        assert!(stepper.step().is_ok());
        assert!(stepper.terminate().is_ok());
    }

    #[test]
    fn test_node_repr_into_node() {
        let mut registry = Registry::default();
        registry.add("Empty", Empty::default);

        let node_repr = NodeRepr {
            typ: "Empty".to_string(),
            props: std::collections::HashMap::new(),
            ..Default::default()
        };

        let result = node_repr.into_node(&registry);
        assert!(result.is_ok());

        let node = result.unwrap();
        let metadata = node.metadata(&registry);
        assert!(metadata.is_some());
    }

    #[test]
    fn test_node_repr_into_node_with_props() {
        let mut registry = Registry::default();
        registry.add("Constant", Constant::default);

        let mut props = std::collections::HashMap::new();
        props.insert(ParamId(0), crate::Value::load(&42i32).unwrap());

        let node_repr = NodeRepr {
            typ: "Constant".to_string(),
            props,
            ..Default::default()
        };

        let result = node_repr.into_node(&registry);
        assert!(result.is_ok());
    }

    #[test]
    fn test_node_repr_into_node_not_found() {
        let registry = Registry::default();

        let node_repr = NodeRepr {
            typ: "Empty".to_string(),
            ..Default::default()
        };

        let result = node_repr.into_node(&registry);
        assert!(result.is_err());
    }

    #[test]
    fn test_node_multiple_steps() {
        let transformer: Box<dyn PropInspector> = Box::new(Empty::default());
        let mut node = Node::new(transformer, "Empty".to_string());

        // Multiple steps should all succeed
        for _ in 0..10 {
            assert!(node.stepper().step().is_ok());
        }
    }

    #[test]
    fn test_node_factory_name() {
        let transformer: Box<dyn PropInspector> = Box::new(Empty::default());
        let node = Node::new(transformer, "TestFactory".to_string());

        // The factory name should be used for metadata lookup
        let mut registry = Registry::default();
        registry.add("TestFactory", Empty::default);

        let metadata = node.metadata(&registry);
        assert!(metadata.is_some());
    }

    #[test]
    fn test_node_prop_access() {
        let transformer: Box<dyn PropInspector> = Box::new(Constant::default());
        let node = Node::new(transformer, "Constant".to_string());

        // Should be able to access properties through transformer
        let transformer = node.transformer();
        let prop = transformer.get_prop(ParamId(0));
        assert!(prop.is_some());
    }

    #[test]
    fn test_node_prop_mut_access() {
        let transformer: Box<dyn PropInspector> = Box::new(Constant::default());
        let mut node = Node::new(transformer, "Constant".to_string());

        // Should be able to access mutable properties through transformer
        let transformer = node.transformer_mut();
        let prop = transformer.get_prop_mut(ParamId(0));
        assert!(prop.is_some());
    }

    #[test]
    fn test_node_prop_not_found() {
        let transformer: Box<dyn PropInspector> = Box::new(Empty::default());
        let node = Node::new(transformer, "Empty".to_string());

        // Should return None for non-existent property
        let transformer = node.transformer();
        let prop = transformer.get_prop(ParamId(999));
        assert!(prop.is_none());
    }

    #[test]
    fn test_node_repr_empty_props() {
        let mut registry = Registry::default();
        registry.add("Empty", Empty::default);

        let node_repr = NodeRepr {
            typ: "Empty".to_string(),
            ..Default::default()
        };

        let result = node_repr.into_node(&registry);
        assert!(result.is_ok());

        let mut node = result.unwrap();
        // Verify the node works correctly
        assert!(node.stepper().step().is_ok());
    }
}
