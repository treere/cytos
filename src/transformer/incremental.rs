use crate::architecture::{GenericInputProp, GenericOutputProp, OutputProp, ParamId, Transformer};

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

impl Default for IncrementalGenerator {
    fn default() -> Self {
        IncrementalGenerator::new()
    }
}

impl Transformer for IncrementalGenerator {
    fn step(&mut self) -> Result<(), &'static str> {
        *self.output.set() += 1;
        Ok(())
    }

    fn output(&self, val: ParamId) -> Option<GenericOutputProp> {
        match val {
            IncrementalGeneratorConfigOutput::OUTPUT => Some(self.output.as_generic()),
            _ => None,
        }
    }

    fn link(&mut self, _name: ParamId, _val: GenericOutputProp) -> Result<(), &'static str> {
        Err("missing param")
    }

    fn input(&self, _val: ParamId) -> Option<GenericInputProp> {
        None
    }

    fn output_names(&self) -> &[ParamId] {
        &[IncrementalGeneratorConfigOutput::OUTPUT]
    }
}
