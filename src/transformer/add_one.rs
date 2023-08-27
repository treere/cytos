use std::rc::Rc;

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

impl Transformer for AddValue {
    fn step(&mut self) -> Result<(), &'static str> {
        *self.output.set() = *self.input.get() + *self.increment.get();

        Ok(())
    }

    fn input(&self, val: ParamId) -> Option<Rc<GenericProp>> {
        match val {
            AddValueConfigInput::INPUT => Some(self.input.get_any()),
            AddValueConfigInput::INCREMENT => Some(self.increment.get_any()),
            _ => None,
        }
    }

    fn output(&self, val: ParamId) -> Option<Rc<GenericProp>> {
        match val {
            AddValueConfigOutput::OUTPUT => Some(self.output.get_any()),
            _ => None,
        }
    }

    fn link(&mut self, name: ParamId, val: Rc<GenericProp>) -> Result<(), &'static str> {
        match name {
            AddValueConfigInput::INPUT => self.input.change_value(val),
            AddValueConfigInput::INCREMENT => self.increment.change_value(val),
            _ => Err("no param"),
        }
    }
}
