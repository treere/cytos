use crate::architecture::{
    GenericInputProp, GenericOutputProp, InputProp, OutputProp, ParamId, Transformer,
};

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

    fn link(&mut self, name: ParamId, val: GenericOutputProp) -> Result<(), &'static str> {
        match name {
            AddValueConfigInput::INPUT => self.input.change_value(val),
            AddValueConfigInput::INCREMENT => self.increment.change_value(val),
            _ => Err("no param"),
        }
    }

    fn output(&self, val: ParamId) -> Option<GenericOutputProp> {
        match val {
            AddValueConfigOutput::OUTPUT => Some(self.output.as_generic()),
            _ => None,
        }
    }

    fn input(&self, val: ParamId) -> Option<GenericInputProp> {
        match val {
            AddValueConfigInput::INPUT => Some(self.input.as_generic()),
            AddValueConfigInput::INCREMENT => Some(self.increment.as_generic()),
            _ => None,
        }
    }

    fn input_names(&self) -> &[ParamId] {
        &[AddValueConfigInput::INPUT, AddValueConfigInput::INCREMENT]
    }

    fn output_names(&self) -> &[ParamId] {
        &[AddValueConfigOutput::OUTPUT]
    }
}
