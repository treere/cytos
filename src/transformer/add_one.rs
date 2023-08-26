use std::rc::Rc;

use crate::architecture::{GenericProp, ParamId, Prop, Transformer};

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
    input: Prop<u64>,
    increment: Prop<u64>,
    output: Prop<u64>,
}

impl AddValue {
    pub fn new() -> Self {
        AddValue {
            input: Prop::new(0u64),
            increment: Prop::new(1u64),
            output: Prop::new(0u64),
        }
    }
}

impl Transformer for AddValue {
    fn step(&mut self) -> Result<(), &'static str> {
        *self.output.set() = *self.input.get() + *self.increment.get();

        Ok(())
    }

    fn inputs_name(&self) -> &[ParamId] {
        &[AddValueConfigInput::INPUT]
    }

    fn input(&self, val: ParamId) -> Rc<GenericProp> {
        match val {
            AddValueConfigInput::INPUT => self.input.get_any(),
            AddValueConfigInput::INCREMENT => self.increment.get_any(),
            _ => unreachable!(),
        }
    }

    fn outputs_name(&self) -> &[ParamId] {
        &[AddValueConfigOutput::OUTPUT]
    }

    fn output(&self, val: ParamId) -> Rc<GenericProp> {
        match val {
            AddValueConfigOutput::OUTPUT => self.output.get_any(),
            _ => unreachable!(),
        }
    }

    fn set_input(&mut self, name: ParamId, val: Rc<GenericProp>) -> Result<(), &'static str> {
        match name {
            AddValueConfigInput::INPUT => self.input.change_value(val),
            AddValueConfigInput::INCREMENT => self.increment.change_value(val),
            _ => Err("no param"),
        }
    }
}
