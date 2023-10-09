use proph::architecture::{InputProp, OutputProp, Stepper};
use proph_derive::TransFn;

#[derive(TransFn)]
pub struct AddValue {
    input: InputProp<u64>,
    increment: InputProp<u64>,
    output: OutputProp<u64>,
}

impl AddValue {
    pub fn new() -> Self {
        AddValue {
            input: InputProp::new(0u64),
            increment: InputProp::new(1u64),
            output: OutputProp::new(0u64),
        }
    }
}

impl Default for AddValue {
    fn default() -> Self {
        AddValue::new()
    }
}

impl Stepper for AddValue {
    fn step(&mut self) -> Result<(), &'static str> {
        *self.output.set() = *self.input.get() + *self.increment.get();

        Ok(())
    }
}
