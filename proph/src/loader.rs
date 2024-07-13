use crate::architecture::{Result, Transformer};

use libloading::{Library, Symbol};

use std::{collections::HashMap, sync::Arc};

type Factory = Arc<dyn Fn() -> Box<dyn Transformer> + Send + Sync>;

/// Registry of transformers
#[derive(Default, Clone)]
pub struct Registry {
    /// Factories
    factories: HashMap<String, Factory>,

    /// Libs
    libs: Vec<Arc<Library>>,
}

impl Registry {
    /// Add a factory
    pub fn add<K: Transformer + 'static>(
        &mut self,
        name: impl AsRef<str>,
        factory: impl (Fn() -> K) + 'static + Send + Sync,
    ) -> &mut Self {
        self.factories
            .entry(name.as_ref().to_owned())
            .or_insert(Arc::new(move || Box::new(factory())));
        self
    }

    /// Load a Processor
    pub fn load(&self, typ: &str) -> Result<Box<dyn Transformer>> {
        let factory = self.factories.get(typ).ok_or("missing type")?;
        Ok(factory())
    }

    pub fn list_factories(&self) -> impl Iterator<Item = &String> {
        self.factories.keys()
    }

    pub fn load_library(&mut self, file: &str) -> Result<()> {
        let lib = unsafe {
            Library::new(libloading::library_filename(file)).or(Err("cannot load library"))?
        };
        let lib = Arc::new(lib);

        let load_registry: Symbol<fn(&mut Registry) -> ()> = unsafe {
            lib.get(b"load_registry")
                .or(Err("missing load_registry function"))?
        };

        load_registry(self);
        self.libs.push(lib);
        Ok(())
    }
}
