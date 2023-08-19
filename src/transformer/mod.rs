pub mod IncrementalGenerator {
    use crate::{
        architecture::{Outputs, ParamId, Params, Transformer},
        data::Data,
    };

    pub mod Config {
        use crate::architecture::ParamId;

        pub const OUTPUT: ParamId = 0;
    }

    pub struct Module(u64);

    impl Module {
        pub fn new() -> Self {
            Module(0)
        }
    }

    impl Transformer for Module {
        fn inputs(&self) -> &[ParamId] {
            &[]
        }

        fn outputs(&self) -> &[ParamId] {
            &[Config::OUTPUT]
        }

        fn outputs_default(&self) -> Vec<(ParamId, Data)> {
            vec![(Config::OUTPUT, Data::U64(0))]
        }

        fn process(&mut self, _inputs: Params, mut outputs: Outputs) -> Result<(), ()> {
            *outputs.get_mut(&Config::OUTPUT) = Data::U64(self.0);

            self.0 += 1;
            Ok(())
        }
    }
}

pub mod AddOne {
    use crate::{
        architecture::{Outputs, ParamId, Params, Transformer},
        data::Data,
    };

    pub mod Config {
        use crate::architecture::ParamId;

        pub const INPUT: ParamId = 0;
        pub const OUTPUT: ParamId = 1;
    }

    pub struct Module;

    impl Module {
        pub fn new() -> Self {
            Module
        }
    }

    impl Transformer for Module {
        fn inputs(&self) -> &[ParamId] {
            &[Config::INPUT]
        }

        fn outputs(&self) -> &[ParamId] {
            &[Config::OUTPUT]
        }

        fn outputs_default(&self) -> Vec<(ParamId, Data)> {
            vec![(Config::OUTPUT, Data::U64(0))]
        }

        fn process(&mut self, inputs: Params, mut outputs: Outputs) -> Result<(), ()> {
            match *inputs.get(&Config::INPUT) {
                Data::U64(v) => {
                    *outputs.get_mut(&Config::OUTPUT) = Data::U64(v + 1);

                    Ok(())
                }
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
    pub const PIPPO: NodeId = 0;

    #[test]
    fn test_add_success() {
        assert!(Orchestrator::new()
            .add(SOURCE1, IncrementalGenerator::Module::new())
            .expect("cannot insert")
            .add(SOURCE2, IncrementalGenerator::Module::new())
            .is_ok())
    }

    #[test]
    fn test_add_same_name() {
        assert!(Orchestrator::new()
            .add(SOURCE, IncrementalGenerator::Module::new())
            .expect("cannot insert")
            .add(SOURCE, IncrementalGenerator::Module::new())
            .is_err())
    }

    #[test]
    fn test_connect_success() {
        assert!(Orchestrator::new()
            .add(SOURCE, IncrementalGenerator::Module::new())
            .expect("cannot add source")
            .add(DOUBLER, AddOne::Module::new())
            .expect("cannot add doubler")
            .connect(
                Path::new(SOURCE, AddOne::Config::OUTPUT),
                Path::new(DOUBLER, AddOne::Config::INPUT)
            )
            .is_ok())
    }

    #[test]
    fn test_connect_missing_destination_source() {
        assert!(Orchestrator::new()
            .add(SOURCE, IncrementalGenerator::Module::new())
            .expect("cannot add source")
            .add(DOUBLER, AddOne::Module::new())
            .expect("cannot add doubler")
            .connect(
                Path::new(SOURCE, AddOne::Config::OUTPUT),
                Path::new(PIPPO, PIPPO)
            )
            .is_err())
    }

    #[test]
    fn test_connect_missing_destination_value() {
        assert!(Orchestrator::new()
            .add(SOURCE, IncrementalGenerator::Module::new())
            .expect("cannot add source")
            .add(DOUBLER, AddOne::Module::new())
            .expect("cannot add doubler")
            .connect(
                Path::new(SOURCE, AddOne::Config::OUTPUT),
                Path::new(DOUBLER, PIPPO)
            )
            .is_err())
    }

    #[test]
    fn test_connect_missing_source_source() {
        assert!(Orchestrator::new()
            .add(SOURCE, IncrementalGenerator::Module::new())
            .expect("cannot add source")
            .add(DOUBLER, AddOne::Module::new())
            .expect("cannot add doubler")
            .connect(
                Path::new(PIPPO, PIPPO),
                Path::new(DOUBLER, AddOne::Config::INPUT)
            )
            .is_err())
    }

    #[test]
    fn test_connect_missing_source_value() {
        assert!(Orchestrator::new()
            .add(SOURCE, IncrementalGenerator::Module::new())
            .expect("cannot add source")
            .add(DOUBLER, AddOne::Module::new())
            .expect("cannot add doubler")
            .connect(
                Path::new(SOURCE, PIPPO),
                Path::new(DOUBLER, AddOne::Config::INPUT)
            )
            .is_err())
    }
}
