use super::{
    props::{GenericInputProp, GenericOutputProp},
    ParamId,
};

/// Stepper trait
pub trait Stepper {
    /// Initialize data
    fn initialize(&mut self) -> Result<(), &'static str>;

    /// Do one computation step
    fn step(&mut self) -> Result<(), &'static str>;
}

/// Transformer trait
pub trait Transformer: Stepper {
    /// Set input
    fn link(&mut self, name: ParamId, val: GenericOutputProp) -> Result<(), &'static str>;

    /// Get ouput by name
    fn output(&self, val: ParamId) -> Option<GenericOutputProp>;

    /// Get input by name
    fn input(&self, val: ParamId) -> Option<GenericInputProp>;

    /// Get input names
    fn input_names(&self) -> Vec<ParamId>;

    /// Get output names
    fn output_names(&self) -> Vec<ParamId>;
}
