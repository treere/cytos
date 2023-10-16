use super::{
    props::{GenericInputProp, GenericOutputProp},
    Done, ParamId, Result, Value,
};

/// Stepper trait
pub trait Stepper {
    /// Initialize data
    fn initialize(&mut self) -> Done {
        Ok(())
    }

    /// Do one computation step
    fn step(&mut self) -> Done;

    /// Terminate execution
    fn terminate(&mut self) -> Done {
        Ok(())
    }
}

/// Transformer trait
pub trait Transformer: Stepper {
    /// Set input
    fn link(&mut self, name: ParamId, val: GenericOutputProp) -> Done;

    /// Load
    fn load(&mut self, name: ParamId, val: Value) -> Done;

    /// Dump
    fn dump(&self, name: ParamId) -> Result<String>;

    /// Get ouput by name
    fn output(&self, val: ParamId) -> Option<GenericOutputProp>;

    /// Get input by name
    fn input(&self, val: ParamId) -> Option<GenericInputProp>;

    /// Get input names
    fn input_names(&self) -> Vec<ParamId>;

    /// Get output names
    fn output_names(&self) -> Vec<ParamId>;
}
