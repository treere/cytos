mod add_one;
mod incremental;
mod list_dir;

pub use add_one::{AddValue, AddValueConfigInput, AddValueConfigOutput};
pub use incremental::{IncrementalGenerator, IncrementalGeneratorConfigOutput};
pub use list_dir::{ListDir, ListDirConfigOutput};

use crate::architecture::Processor;

pub fn load(name: &str, typ: &str) -> Processor {
    match typ {
        "IncrementalGenerator" => Processor::new(name.to_owned(), IncrementalGenerator::new()),
        "AddValue" => Processor::new(name.to_owned(), AddValue::new()),
        _ => unimplemented!("missing type"),
    }
}
