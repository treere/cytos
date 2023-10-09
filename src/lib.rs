extern crate proph_derive;

mod add_one;
mod incremental;

pub use add_one::AddValue;
pub use incremental::{IncrementalGenerator, IncrementalGeneratorConfigOutput};
