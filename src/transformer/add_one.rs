use crate::architecture::{
    new_shared, InputConfiguration, OutputConfiguration, ParamId, Params, Results, SharedData,
    Transformer,
};

#[allow(non_snake_case)]
pub mod AddConfigConfigInput {
    use crate::architecture::ParamId;

    pub const INPUT: ParamId = 0;
    pub const INCREMENT: ParamId = 2;
}

#[allow(non_snake_case)]
pub mod AddValueConfigOutput {
    use crate::architecture::ParamId;

    pub const OUTPUT: ParamId = 1;
}

pub struct AddValue;

impl AddValue {
    pub fn new() -> Self {
        AddValue
    }
}

impl InputConfiguration for AddValue {
    fn inputs(&self) -> &[ParamId] {
        &[AddConfigConfigInput::INPUT]
    }

    fn input_default(&self, val: ParamId) -> SharedData {
        match val {
            AddConfigConfigInput::INPUT => new_shared(0u64),
            AddConfigConfigInput::INCREMENT => new_shared(1u64),
            _ => unreachable!(),
        }
    }
}

impl OutputConfiguration for AddValue {
    fn outputs(&self) -> &[ParamId] {
        &[AddValueConfigOutput::OUTPUT]
    }

    fn output_default(&self, val: ParamId) -> SharedData {
        match val {
            AddValueConfigOutput::OUTPUT => new_shared(0u64),
            _ => unreachable!(),
        }
    }
}

impl Transformer for AddValue {
    fn process(&mut self, inputs: Params, mut outputs: Results) -> Result<(), &'static str> {
        let v = inputs.get::<u64>(&(AddConfigConfigInput::INPUT))?;

        let mut output = outputs.get_mut::<u64>(&(AddValueConfigOutput::OUTPUT))?;

        *output = *v + 1;

        Ok(())
    }
}
