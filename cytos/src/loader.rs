//! Module to handle dynamic library loading

use crate::{MetadataProvider, NodeMetadata, PropInspector, Result};

use libloading::{Library, Symbol};

use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
/// A wrapper around a function that returns a transformer.
struct FactoryContainer(Arc<dyn Fn() -> Box<dyn PropInspector> + Send + Sync>);

impl FactoryContainer {
    /// Create a `FactoryContaier` from a generic factory
    fn new<K: PropInspector + 'static>(factory: impl (Fn() -> K) + 'static + Send + Sync) -> Self {
        Self(Arc::new(move || Box::new(factory())))
    }

    /// Get the transformer
    fn get(&self) -> Box<dyn PropInspector> {
        (self.0)()
    }
}

/// A wrapper to load factories from a dynamic library
pub struct DynamicLoadingRegistryWrapper<'a> {
    /// The registry reference
    registry: &'a mut Registry,

    /// The library
    lib: Arc<Library>,
}

impl DynamicLoadingRegistryWrapper<'_> {
    /// Add a dynamic factory by name removing the previous one.
    pub fn add<K: PropInspector + MetadataProvider + 'static>(
        &mut self,
        name: &str,
        factory: impl (Fn() -> K) + 'static + Send + Sync,
    ) -> &mut Self {
        let metadata = Some(K::metadata());
        self.registry.add_by_type(
            name,
            FactoryType::Dynamic(((FactoryContainer::new(factory), metadata), self.lib.clone())),
        );
        self
    }
}

/// Type of loaded factories
#[derive(Clone)]
enum FactoryType {
    /// A simple static factory
    Plain((FactoryContainer, Option<NodeMetadata>)),

    /// A factory that is in a library
    Dynamic(((FactoryContainer, Option<NodeMetadata>), Arc<Library>)),
}

/// Registry of transformers
///
/// This struct contains the available factories.
///
/// Factories can be loaded by name or dynamically loaded
#[derive(Default, Clone)]
pub struct Registry {
    /// Factories by name
    factories: HashMap<String, FactoryType>,
}

impl Registry {
    /// Add a factory by name removing the previous one.
    pub fn add<K: PropInspector + MetadataProvider + 'static>(
        &mut self,
        name: &str,
        factory: impl (Fn() -> K) + 'static + Send + Sync,
    ) -> &mut Self {
        let metadata = Some(K::metadata());
        self.add_by_type(
            name,
            FactoryType::Plain((FactoryContainer::new(factory), metadata)),
        );
        self
    }

    fn add_by_type(&mut self, name: &str, factory: FactoryType) {
        self.factories.entry(name.to_owned()).or_insert(factory);
    }

    /// Returns a factory by name
    ///
    /// # Errors
    ///
    /// Will return `Err` if the factory is missing
    pub fn load(&self, name: &str) -> Result<Box<dyn PropInspector>> {
        let factory = self
            .factories
            .get(name)
            .ok_or_else(|| format!("cannot find \"{name}\""))?;

        match factory {
            FactoryType::Plain((factory, _)) | FactoryType::Dynamic(((factory, _), _)) => {
                Ok(factory.get())
            }
        }
    }

    /// Returns metadata for a factory by name
    pub fn get_metadata(&self, name: &str) -> Option<&NodeMetadata> {
        self.factories.get(name).and_then(|factory| match factory {
            FactoryType::Plain((_, metadata)) | FactoryType::Dynamic(((_, metadata), _)) => {
                metadata.as_ref()
            }
        })
    }

    /// Return the list of afailable factories
    pub fn list_factories(&self) -> impl Iterator<Item = &String> {
        self.factories.keys()
    }

    /// Dynamically load a library
    ///
    /// # Errors
    ///
    /// Will return `Err` if the library cannot be loaded
    pub fn load_library(&mut self, file: &str) -> Result<()> {
        let lib = unsafe { Library::new(file) }.or(Err("cannot load library"))?;
        let lib = Arc::new(lib);

        let load_registry_fun: Symbol<fn(&mut DynamicLoadingRegistryWrapper) -> ()> = unsafe {
            lib.get(b"load_registry")
                .or(Err("missing load_registry function"))?
        };

        load_registry_fun(&mut DynamicLoadingRegistryWrapper {
            registry: self,
            lib,
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::test::Empty;

    use super::*;

    #[test]
    fn test_error_on_not_found() {
        let reg = Registry::default();
        assert!(reg.load("pippo").is_err());
    }

    #[test]
    fn test_retrieve_loaded() {
        let mut reg = Registry::default();

        reg.add("pippo", Empty::default);

        assert!(reg.load("pippo").is_ok());
    }
}
