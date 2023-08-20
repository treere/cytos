use crate::{
    architecture::{
        InputConfiguration, OutputConfiguration, Outputs, ParamId, Params, Transformer,
    },
    data::Data,
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

    fn inputs_default(&self, val: ParamId) -> Data {
        match val {
            AddConfigConfigInput::INPUT => Data::U64(0),
            AddConfigConfigInput::INCREMENT => Data::U64(1),
            _ => unreachable!(),
        }
    }
}

impl OutputConfiguration for AddValue {
    fn outputs(&self) -> &[ParamId] {
        &[AddValueConfigOutput::OUTPUT]
    }

    fn outputs_default(&self, val: ParamId) -> Data {
        match val {
            AddValueConfigOutput::OUTPUT => Data::U64(0),
            _ => unreachable!(),
        }
    }
}

impl Transformer for AddValue {
    fn process(&mut self, inputs: Params, mut outputs: Outputs) -> Result<(), ()> {
        match *inputs.get(&(AddConfigConfigInput::INPUT)).unwrap() {
            Data::U64(v) => {
                *outputs.get_mut(&(AddValueConfigOutput::OUTPUT)).unwrap() = Data::U64(v + 1);

                Ok(())
            }
        }
    }
}
