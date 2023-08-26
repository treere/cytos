use crate::architecture::{ParamId, Prop, SharedData, Transformer};

#[allow(non_snake_case)]
pub mod IncrementalGeneratorConfigOutput {
    use crate::architecture::ParamId;

    pub const OUTPUT: ParamId = "output";
}

pub struct IncrementalGenerator {
    output: Prop<u64>,
}

impl IncrementalGenerator {
    pub fn new() -> Self {
        IncrementalGenerator {
            output: Prop::new(0),
        }
    }
}

impl Transformer for IncrementalGenerator {
    fn step(&mut self) -> Result<(), &'static str> {
        *self.output.set() += 1;
        Ok(())
    }

    fn inputs_name(&self) -> &[ParamId] {
        &[]
    }

    fn input(&self, _val: ParamId) -> SharedData {
        unreachable!()
    }

    fn outputs_name(&self) -> &[ParamId] {
        &[IncrementalGeneratorConfigOutput::OUTPUT]
    }

    fn output(&self, val: ParamId) -> SharedData {
        match val {
            IncrementalGeneratorConfigOutput::OUTPUT => self.output.get_shared(),
            _ => unreachable!(),
        }
    }

    fn set_input(&mut self, name: ParamId, val: SharedData) -> Result<(), &'static str> {
        match name {
            IncrementalGeneratorConfigOutput::OUTPUT => self.output.change_value(val),
            _ => unreachable!(),
        }
    }
}
