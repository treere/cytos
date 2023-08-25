use crate::architecture::{
    new_shared, InputConfiguration, OutputConfiguration, ParamId, Params, Results, SharedData,
    Transformer,
};

#[allow(non_snake_case)]
pub mod IncrementalGeneratorConfigOutput {
    use crate::architecture::ParamId;

    pub const OUTPUT: ParamId = 0;
}

pub struct IncrementalGenerator(u64);

impl IncrementalGenerator {
    pub fn new() -> Self {
        IncrementalGenerator(0)
    }
}

impl InputConfiguration for IncrementalGenerator {
    fn inputs(&self) -> &[ParamId] {
        &[]
    }

    fn input_default(&self, _val: ParamId) -> SharedData {
        unreachable!()
    }
}

impl OutputConfiguration for IncrementalGenerator {
    fn outputs(&self) -> &[ParamId] {
        &[IncrementalGeneratorConfigOutput::OUTPUT]
    }

    fn output_default(&self, val: ParamId) -> SharedData {
        match val {
            IncrementalGeneratorConfigOutput::OUTPUT => new_shared(0u64),
            _ => unreachable!(),
        }
    }
}

impl Transformer for IncrementalGenerator {
    fn process(&mut self, _inputs: Params, mut outputs: Results) -> Result<(), &'static str> {
        let mut output = outputs.get_mut(&(IncrementalGeneratorConfigOutput::OUTPUT))?;

        *output = self.0;

        self.0 += 1;
        Ok(())
    }
}
