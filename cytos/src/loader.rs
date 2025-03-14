//! Module to handle dynamic library loading

use crate::{Result, Transformer};

use libloading::{Library, Symbol};

use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
/// A wrapper around a function that returns a transfomer
struct FactoryContainer(Arc<dyn Fn() -> Box<dyn Transformer> + Send + Sync>);

impl FactoryContainer {
    /// Create a `FactoryContaier` from a generic factory
    fn new<K: Transformer + 'static>(factory: impl (Fn() -> K) + 'static + Send + Sync) -> Self {
        Self(Arc::new(move || Box::new(factory())))
    }

    /// Get the transformer
    fn get(&self) -> Box<dyn Transformer> {
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
    pub fn add<K: Transformer + 'static>(
        &mut self,
        name: &str,
        factory: impl (Fn() -> K) + 'static + Send + Sync,
    ) -> &mut Self {
        self.registry.add_by_type(
            name,
            FactoryType::Dynamic((FactoryContainer::new(factory), self.lib.clone())),
        );
        self
    }
}

/// Type of loaded factories
#[derive(Clone)]
enum FactoryType {
    /// A simple static factory
    Plain(FactoryContainer),

    /// A factory that is in a library
    Dynamic((FactoryContainer, Arc<Library>)),
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
    pub fn add<K: Transformer + 'static>(
        &mut self,
        name: &str,
        factory: impl (Fn() -> K) + 'static + Send + Sync,
    ) -> &mut Self {
        self.add_by_type(name, FactoryType::Plain(FactoryContainer::new(factory)));
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
    pub fn load(&self, name: &str) -> Result<Box<dyn Transformer>> {
        let factory = self
            .factories
            .get(name)
            .ok_or_else(|| format!("cannot find \"{name}\""))?;

        match factory {
            FactoryType::Plain(factory) | FactoryType::Dynamic((factory, _)) => Ok(factory.get()),
        }
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
