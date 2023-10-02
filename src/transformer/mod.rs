mod add_one;
mod incremental;

pub use add_one::{AddValue, AddValueConfigInput, AddValueConfigOutput};
pub use incremental::{IncrementalGenerator, IncrementalGeneratorConfigOutput};

use crate::architecture::Processor;

pub fn load(name: &str, typ: &str) -> Processor {
    match typ {
        "IncrementalGenerator" => Processor::new(name.to_owned(), IncrementalGenerator::new()),
        "AddValue" => Processor::new(name.to_owned(), AddValue::new()),
        _ => unimplemented!("missing type"),
    }
}
