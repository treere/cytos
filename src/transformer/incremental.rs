use crate::{
    architecture::{
        InputConfiguration, OutputConfiguration, ParamId, Params, Results, Transformer,
    },
    data::Data,
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

    fn input_default(&self, _val: ParamId) -> Data {
        unreachable!()
    }
}

impl OutputConfiguration for IncrementalGenerator {
    fn outputs(&self) -> &[ParamId] {
        &[IncrementalGeneratorConfigOutput::OUTPUT]
    }

    fn output_default(&self, val: ParamId) -> Data {
        match val {
            IncrementalGeneratorConfigOutput::OUTPUT => Data::U64(0),
            _ => unreachable!(),
        }
    }
}

impl Transformer for IncrementalGenerator {
    fn process(&mut self, _inputs: Params, mut outputs: Results) -> Result<(), &'static str> {
        *outputs
            .get_mut(&(IncrementalGeneratorConfigOutput::OUTPUT))
            .unwrap() = Data::U64(self.0);

        self.0 += 1;
        Ok(())
    }
}
