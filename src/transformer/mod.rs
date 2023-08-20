use crate::{
    architecture::{
        InputConfiguration, OutputConfiguration, Outputs, ParamId, Params, Transformer,
    },
    data::Data,
};

pub enum IncrementalGeneratorConfigOutput {
    OUTPUT = 0,
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
        &[IncrementalGeneratorConfigOutput::OUTPUT as u32]
    }

    fn outputs_default(&self) -> Vec<(ParamId, Data)> {
        vec![(
            IncrementalGeneratorConfigOutput::OUTPUT as u32,
            Data::U64(0),
        )]
    }
}

impl Transformer for IncrementalGenerator {
    fn process(&mut self, _inputs: Params, mut outputs: Outputs) -> Result<(), ()> {
        *outputs.get_mut(&(IncrementalGeneratorConfigOutput::OUTPUT as u32)) = Data::U64(self.0);

        self.0 += 1;
        Ok(())
    }
}

pub enum AddOneConfigInput {
    INPUT = 0,
}

pub enum AddOneConfigOutput {
    OUTPUT = 1,
}

pub struct AddOne;

impl AddOne {
    pub fn new() -> Self {
        AddOne
    }
}

impl InputConfiguration for AddOne {
    fn inputs(&self) -> &[ParamId] {
        &[AddOneConfigInput::INPUT as u32]
    }

    fn inputs_default(&self) -> Vec<(ParamId, Data)> {
        vec![(AddOneConfigOutput::OUTPUT as u32, Data::U64(0))]
    }
}

impl OutputConfiguration for AddOne {
    fn outputs(&self) -> &[ParamId] {
        &[AddOneConfigOutput::OUTPUT as u32]
    }

    fn outputs_default(&self) -> Vec<(ParamId, Data)> {
        vec![(AddOneConfigOutput::OUTPUT as u32, Data::U64(0))]
    }
}

impl Transformer for AddOne {
    fn process(&mut self, inputs: Params, mut outputs: Outputs) -> Result<(), ()> {
        match *inputs.get(&(AddOneConfigInput::INPUT as u32)) {
            Data::U64(v) => {
                *outputs.get_mut(&(AddOneConfigOutput::OUTPUT as u32)) = Data::U64(v + 1);

                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::architecture::{NodeId, Orchestrator, Path};

    use super::*;

    pub const SOURCE1: NodeId = 7;
    pub const SOURCE2: NodeId = 8;
    pub const SOURCE: NodeId = 1;
    pub const DOUBLER: NodeId = 9;
    pub const PIPPO: NodeId = 255;

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
                Path::new(SOURCE, IncrementalGeneratorConfigOutput::OUTPUT as u32),
                Path::new(DOUBLER, AddOneConfigInput::INPUT as u32)
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
                Path::new(SOURCE, IncrementalGeneratorConfigOutput::OUTPUT as u32),
                Path::new(PIPPO, PIPPO)
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
                Path::new(SOURCE, IncrementalGeneratorConfigOutput::OUTPUT as u32),
                Path::new(DOUBLER, PIPPO)
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
            .connect(
                Path::new(PIPPO, PIPPO),
                Path::new(DOUBLER, AddOneConfigInput::INPUT as u32)
            )
            .is_err())
    }

    #[test]
    fn test_connect_missing_source_value() {
        assert!(Orchestrator::new()
            .add(SOURCE, IncrementalGenerator::new())
            .expect("cannot add source")
            .add(DOUBLER, AddOne::new())
            .expect("cannot add doubler")
            .connect(
                Path::new(SOURCE, PIPPO),
                Path::new(DOUBLER, AddOneConfigInput::INPUT as u32)
            )
            .is_err())
    }
}
