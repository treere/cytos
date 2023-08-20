pub(crate) use crate::{
    architecture::{
        InputConfiguration, OutputConfiguration, Outputs, ParamId, Params, Transformer,
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

    fn inputs_default(&self) -> Vec<(ParamId, Data)> {
        vec![]
    }
}

impl OutputConfiguration for IncrementalGenerator {
    fn outputs(&self) -> &[ParamId] {
        &[IncrementalGeneratorConfigOutput::OUTPUT]
    }

    fn outputs_default(&self) -> Vec<(ParamId, Data)> {
        vec![(IncrementalGeneratorConfigOutput::OUTPUT, Data::U64(0))]
    }
}

impl Transformer for IncrementalGenerator {
    fn process(&mut self, _inputs: Params, mut outputs: Outputs) -> Result<(), ()> {
        *outputs
            .get_mut(&(IncrementalGeneratorConfigOutput::OUTPUT))
            .unwrap() = Data::U64(self.0);

        self.0 += 1;
        Ok(())
    }
}

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

    fn inputs_default(&self) -> Vec<(ParamId, Data)> {
        vec![(AddOneConfigInput::INPUT, Data::U64(0))]
    }
}

impl OutputConfiguration for AddOne {
    fn outputs(&self) -> &[ParamId] {
        &[AddOneConfigOutput::OUTPUT]
    }

    fn outputs_default(&self) -> Vec<(ParamId, Data)> {
        vec![(AddOneConfigOutput::OUTPUT, Data::U64(0))]
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

#[cfg(test)]
mod tests {
    use crate::architecture::{NodeId, Orchestrator};

    use super::*;

    pub const SOURCE1: NodeId = 7;
    pub const SOURCE2: NodeId = 8;
    pub const SOURCE: NodeId = 1;
    pub const DOUBLER: NodeId = 9;
    pub const PIPPO: NodeId = 255;
    pub const PLUTO: ParamId = 255;

    #[test]
    fn test_add_success() {
        assert!(Orchestrator::new()
            .add(SOURCE1, IncrementalGenerator::new())
            .expect("cannot insert")
            .add(SOURCE2, IncrementalGenerator::new())
            .is_ok())
    }

    #[test]
    fn test_add_same_name() {
        assert!(Orchestrator::new()
            .add(SOURCE, IncrementalGenerator::new())
            .expect("cannot insert")
            .add(SOURCE, IncrementalGenerator::new())
            .is_err())
    }

    #[test]
    fn test_connect_success() {
        assert!(Orchestrator::new()
            .add(SOURCE, IncrementalGenerator::new())
            .expect("cannot add source")
            .add(DOUBLER, AddOne::new())
            .expect("cannot add doubler")
            .connect(
                (SOURCE, IncrementalGeneratorConfigOutput::OUTPUT),
                (DOUBLER, AddOneConfigInput::INPUT)
            )
            .is_ok())
    }

    #[test]
    fn test_connect_missing_destination_source() {
        assert!(Orchestrator::new()
            .add(SOURCE, IncrementalGenerator::new())
            .expect("cannot add source")
            .add(DOUBLER, AddOne::new())
            .expect("cannot add doubler")
            .connect(
                (SOURCE, IncrementalGeneratorConfigOutput::OUTPUT),
                (PIPPO, PLUTO)
            )
            .is_err())
    }

    #[test]
    fn test_connect_missing_destination_value() {
        assert!(Orchestrator::new()
            .add(SOURCE, IncrementalGenerator::new())
            .expect("cannot add source")
            .add(DOUBLER, AddOne::new())
            .expect("cannot add doubler")
            .connect(
                (SOURCE, IncrementalGeneratorConfigOutput::OUTPUT),
                (DOUBLER, PLUTO)
            )
            .is_err())
    }

    #[test]
    fn test_connect_missing_source_source() {
        assert!(Orchestrator::new()
            .add(SOURCE, IncrementalGenerator::new())
            .expect("cannot add source")
            .add(DOUBLER, AddOne::new())
            .expect("cannot add doubler")
            .connect((PIPPO, PLUTO), (DOUBLER, AddOneConfigInput::INPUT))
            .is_err())
    }

    #[test]
    fn test_connect_missing_source_value() {
        assert!(Orchestrator::new()
            .add(SOURCE, IncrementalGenerator::new())
            .expect("cannot add source")
            .add(DOUBLER, AddOne::new())
            .expect("cannot add doubler")
            .connect((SOURCE, PLUTO), (DOUBLER, AddOneConfigInput::INPUT))
            .is_err())
    }
}
