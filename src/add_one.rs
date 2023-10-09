use proph::architecture::{InputProp, OutputProp, Stepper};
use proph_derive::TransFn;

#[derive(TransFn)]
pub struct AddValue {
    input: InputProp<u64>,
    increment: InputProp<u64>,
    output: OutputProp<u64>,
}

impl Default for AddValue {
    fn default() -> Self {
        AddValue {
            input: InputProp::default(),
            increment: InputProp::new(1),
            output: OutputProp::default(),
        }
    }
}

impl Stepper for AddValue {
    fn step(&mut self) -> Result<(), &'static str> {
        *self.output.set() = *self.input.get() + *self.increment.get();

        Ok(())
    }
}
