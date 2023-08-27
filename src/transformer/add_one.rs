use crate::architecture::{GenericProp, InputProp, OutputProp, ParamId, Transformer};

#[allow(non_snake_case)]
pub mod AddValueConfigInput {
    use crate::architecture::ParamId;

    pub const INPUT: ParamId = "input";
    pub const INCREMENT: ParamId = "increment";
}

#[allow(non_snake_case)]
pub mod AddValueConfigOutput {
    use crate::architecture::ParamId;

    pub const OUTPUT: ParamId = "output";
}

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

impl Transformer for AddValue {
    fn step(&mut self) -> Result<(), &'static str> {
        *self.output.set() = *self.input.get() + *self.increment.get();

        Ok(())
    }

    fn output(&self, val: ParamId) -> Option<GenericProp> {
        match val {
            AddValueConfigOutput::OUTPUT => Some(self.output.as_generic()),
            _ => None,
        }
    }

    fn link(&mut self, name: ParamId, val: GenericProp) -> Result<(), &'static str> {
        match name {
            AddValueConfigInput::INPUT => self.input.change_value(val),
            AddValueConfigInput::INCREMENT => self.increment.change_value(val),
            _ => Err("no param"),
        }
    }
}
