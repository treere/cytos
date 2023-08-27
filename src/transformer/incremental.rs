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

    fn output(&self, val: ParamId) -> Option<GenericProp> {
        match val {
            IncrementalGeneratorConfigOutput::OUTPUT => Some(self.output.as_generic()),
            _ => None,
        }
    }

    fn link(&mut self, _name: ParamId, _val: GenericProp) -> Result<(), &'static str> {
        Err("missing param")
    }
}
