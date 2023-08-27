use std::rc::Rc;

use crate::architecture::{GenericProp, OutputProp, ParamId, Transformer};

#[allow(non_snake_case)]
pub mod IncrementalGeneratorConfigOutput {
    use crate::architecture::ParamId;

    pub const OUTPUT: ParamId = "output";
}

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

impl Transformer for IncrementalGenerator {
    fn step(&mut self) -> Result<(), &'static str> {
        *self.output.set() += 1;
        Ok(())
    }

    fn input(&self, _val: ParamId) -> Option<Rc<GenericProp>> {
        None
    }

    fn output(&self, val: ParamId) -> Option<Rc<GenericProp>> {
        match val {
            IncrementalGeneratorConfigOutput::OUTPUT => Some(self.output.get_any()),
            _ => None,
        }
    }

    fn set_input(&mut self, name: ParamId, val: Rc<GenericProp>) -> Result<(), &'static str> {
        match name {
            IncrementalGeneratorConfigOutput::OUTPUT => self.output.change_value(val),
            _ => Err("missing param"),
        }
    }
}
