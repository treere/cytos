use super::{
    props::{GenericInputProp, GenericOutputProp},
    ParamId,
};

pub trait Stepper {
    fn initialize(&mut self) -> Result<(), &'static str>;

    fn step(&mut self) -> Result<(), &'static str>;
}

/// Transformer trait
pub trait Transformer: Stepper {
    /// Set input
    fn link(&mut self, name: ParamId, val: GenericOutputProp) -> Result<(), &'static str>;

    /// Get the default of a parameter
    fn output(&self, val: ParamId) -> Option<GenericOutputProp>;

    fn input(&self, val: ParamId) -> Option<GenericInputProp>;

    fn input_names(&self) -> Vec<ParamId> {
        vec![]
    }

    fn output_names(&self) -> Vec<ParamId> {
        vec![]
    }
}
