use super::{
    props::{GenericOwnedProp, GenericProp},
    ParamId, Result, Value,
};

/// Stepper trait
pub trait Stepper {
    /// Initialize data
    fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    /// Do one computation step
    fn step(&mut self) -> Result<()>;

    /// Terminate execution
    fn terminate(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Transformer trait
pub trait Transformer: Stepper {
    /// Set input
    fn link(&mut self, name: ParamId, val: GenericProp) -> Result<()>;

    /// Load
    fn load(&mut self, name: ParamId, val: Value) -> Result<()>;

    /// assign
    fn assign(&mut self, name: ParamId, val: Value) -> Result<()>;

    /// Dump
    fn dump(&self, name: ParamId) -> Result<Value>;

    /// Load
    fn load_owned(&mut self, name: ParamId, val: GenericOwnedProp) -> Result<()>;

    /// Dump
    fn dump_owned(&self, name: ParamId) -> Result<GenericOwnedProp>;

    /// Get ouput by name
    fn output(&self, val: ParamId) -> Option<GenericProp>;

    /// Get input by name
    fn input(&self, val: ParamId) -> Option<GenericProp>;

    /// Get input names
    fn input_names(&self) -> Vec<ParamId>;

    /// Get output names
    fn output_names(&self) -> Vec<ParamId>;
}
