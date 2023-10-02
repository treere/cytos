mod add_one;
mod incremental;

use std::collections::HashMap;

pub use add_one::{AddValue, AddValueConfigInput, AddValueConfigOutput};
pub use incremental::{IncrementalGenerator, IncrementalGeneratorConfigOutput};

use crate::architecture::{Processor, Transformer};

pub struct Loader {
    transformers: HashMap<String, Box<dyn Fn() -> Box<dyn Transformer>>>,
}

impl Loader {
    pub fn new() -> Self {
        Self {
            transformers: HashMap::new(),
        }
    }
    pub fn add<K: Transformer + 'static>(
        mut self,
        name: impl AsRef<str>,
        f: impl (Fn() -> K) + 'static,
    ) -> Self {
        self.transformers
            .entry(name.as_ref().to_owned())
            .or_insert(Box::new(move || Box::new(f())));
        self
    }

    pub fn load(&self, name: &str, typ: &str) -> Result<Processor, &'static str> {
        let factory = self.transformers.get(typ).ok_or("missing type")?;
        Ok(Processor::load(name.to_owned(), factory()))
    }
}

impl Default for Loader {
    fn default() -> Self {
        Self::new()
    }
}
