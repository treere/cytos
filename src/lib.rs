extern crate proph_derive;

mod add_one;
mod incremental;

pub use add_one::{AddValue, AddValueConfigInput, AddValueConfigOutput};
pub use incremental::{IncrementalGenerator, IncrementalGeneratorConfigOutput};
use proph::architecture::{InputProp, OutputProp, Stepper};
use proph_derive::TransFn;

#[derive(TransFn)]
pub struct AddValue2 {
    pub input: InputProp<u64>,

    pub increment: InputProp<u64>,

    pub output: OutputProp<u64>,
}

impl Stepper for AddValue2 {
    fn step(&mut self) -> Result<(), &'static str> {
        *self.output.set() = *self.input.get() + *self.increment.get();

        Ok(())
    }
}

pub fn a(_x: AddValue2) {}
