use crate::{
    architecture::{
        InputConfiguration, OutputConfiguration, Outputs, ParamId, Params, Transformer,
    },
    data::Data,
};

#[allow(non_snake_case)]
pub mod AddOneConfigInput {
    use crate::architecture::ParamId;

    pub const INPUT: ParamId = 0;
}

#[allow(non_snake_case)]
pub mod AddOneConfigOutput {
    use crate::architecture::ParamId;

    pub const OUTPUT: ParamId = 1;
}

pub struct AddOne;

impl AddOne {
    pub fn new() -> Self {
        AddOne
    }
}

impl InputConfiguration for AddOne {
    fn inputs(&self) -> &[ParamId] {
        &[AddOneConfigInput::INPUT]
    }

    fn inputs_default(&self, val: ParamId) -> Data {
        match val {
            AddOneConfigInput::INPUT => Data::U64(0),
            _ => unreachable!(),
        }
    }
}

impl OutputConfiguration for AddOne {
    fn outputs(&self) -> &[ParamId] {
        &[AddOneConfigOutput::OUTPUT]
    }

    fn outputs_default(&self, val: ParamId) -> Data {
        match val {
            AddOneConfigOutput::OUTPUT => Data::U64(0),
            _ => unreachable!(),
        }
    }
}

impl Transformer for AddOne {
    fn process(&mut self, inputs: Params, mut outputs: Outputs) -> Result<(), ()> {
        match *inputs.get(&(AddOneConfigInput::INPUT)).unwrap() {
            Data::U64(v) => {
                *outputs.get_mut(&(AddOneConfigOutput::OUTPUT)).unwrap() = Data::U64(v + 1);

                Ok(())
            }
        }
    }
}
