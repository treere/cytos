use proph::architecture::{
    GenericInputProp, GenericOutputProp, InputProp, OutputProp, ParamId, Stepper, Transformer,
};

#[allow(non_snake_case)]
pub mod AddValueConfigInput {
    pub const INPUT: &str = "input";
    pub const INCREMENT: &str = "increment";
}

#[allow(non_snake_case)]
pub mod AddValueConfigOutput {
    pub const OUTPUT: &str = "output";
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

impl Stepper for AddValue {
    fn step(&mut self) -> Result<(), &'static str> {
        *self.output.set() = *self.input.get() + *self.increment.get();

        Ok(())
    }
}

impl Transformer for AddValue {
    fn link(&mut self, name: ParamId, val: GenericOutputProp) -> Result<(), &'static str> {
        match name.as_str() {
            AddValueConfigInput::INPUT => self.input.change_value(val),
            AddValueConfigInput::INCREMENT => self.increment.change_value(val),
            _ => Err("no param"),
        }
    }

    fn output(&self, val: ParamId) -> Option<GenericOutputProp> {
        match val.as_str() {
            AddValueConfigOutput::OUTPUT => Some(self.output.as_generic()),
            _ => None,
        }
    }

    fn input(&self, val: ParamId) -> Option<GenericInputProp> {
        match val.as_str() {
            AddValueConfigInput::INPUT => Some(self.input.as_generic()),
            AddValueConfigInput::INCREMENT => Some(self.increment.as_generic()),
            _ => None,
        }
    }

    fn input_names(&self) -> Vec<ParamId> {
        vec![
            AddValueConfigInput::INPUT.to_owned(),
            AddValueConfigInput::INCREMENT.to_owned(),
        ]
    }

    fn output_names(&self) -> Vec<ParamId> {
        vec![AddValueConfigOutput::OUTPUT.to_owned()]
    }
}
