//! Module to handle dynamic library loading

use crate::architecture::{Result, Transformer};

use libloading::{Library, Symbol};

use std::{collections::HashMap, sync::Arc};

/// A `Factory` is  function that returns a Box to a pointer
type Factory = Arc<dyn Fn() -> Box<dyn Transformer> + Send + Sync>;

/// Registry of transformers
///
/// This struct contains the available factories.
///
/// Factories can be loaded by name or dynamically loaded
#[derive(Default, Clone)]
pub struct Registry {
    /// Factories by name
    factories: HashMap<String, Factory>,

    /// Reference to libs
    libs: Vec<Arc<Library>>,
}

impl Registry {
    /// Add a factory by name removing the previous one.
    pub fn add<K: Transformer + 'static>(
        &mut self,
        name: &str,
        factory: impl (Fn() -> K) + 'static + Send + Sync,
    ) -> &mut Self {
        self.factories
            .entry(name.to_owned())
            .or_insert(Arc::new(move || Box::new(factory())));
        self
    }

    /// Returns a factory by name
    pub fn load(&self, name: &str) -> Result<Box<dyn Transformer>> {
        let factory = self
            .factories
            .get(name)
            .ok_or_else(|| format!("cannot find \"{name}\""))?;
        Ok(factory())
    }

    /// Return the list of afailable factories
    pub fn list_factories(&self) -> impl Iterator<Item = &String> {
        self.factories.keys()
    }

    /// Dynamically load a library
    pub fn load_library(&mut self, file: &str) -> Result<()> {
        let lib = unsafe { Library::new(libloading::library_filename(file)) }
            .or(Err("cannot load library"))?;
        let lib = Arc::new(lib);

        let load_registry_fun: Symbol<fn(&mut Registry) -> ()> = unsafe {
            lib.get(b"load_registry")
                .or(Err("missing load_registry function"))?
        };

        load_registry_fun(self);
        self.libs.push(lib);
        Ok(())
    }
}
