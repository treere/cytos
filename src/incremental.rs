use proph::architecture::{OutputProp, Stepper};
use proph_derive::TransFn;

#[derive(TransFn)]
pub struct IncrementalGenerator {
    output: OutputProp<u64>,
}

impl IncrementalGenerator {
    pub fn new() -> Self {
        IncrementalGenerator {
            output: OutputProp::new(0),
        }
    }
}

impl Default for IncrementalGenerator {
    fn default() -> Self {
        IncrementalGenerator::new()
    }
}

impl Stepper for IncrementalGenerator {
    fn step(&mut self) -> Result<(), &'static str> {
        *self.output.set() += 1;
        Ok(())
    }
}
