use crate::architecture::{GenericInputProp, GenericOutputProp, OutputProp, ParamId, Transformer};

#[allow(non_snake_case)]
pub mod IncrementalGeneratorConfigOutput {
    pub const OUTPUT: &str = "output";
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
        match val.as_str() {
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

    fn output_names(&self) -> Vec<ParamId> {
        vec![IncrementalGeneratorConfigOutput::OUTPUT.to_owned()]
    }
}
