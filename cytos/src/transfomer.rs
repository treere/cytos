use super::{
    props::{GenericOwnedProp, GenericProp},
    ParamId, Result, Value,
};

/// Stepper trait
pub trait Stepper {
    /// Initialize data
    ///
    /// # Errors
    /// If the initialize fails
    fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    /// Do one computation step
    ///
    /// # Errors
    /// If cannot process the step
    fn step(&mut self) -> Result<()>;

    /// Terminate execution
    ///
    /// # Errors
    /// If cannot stop
    fn terminate(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Transformer trait
pub trait Transformer: Stepper {
    /// Set input
    ///
    /// # Errors
    /// If cannot link
    fn link(&mut self, name: ParamId, val: GenericProp) -> Result<()>;

    /// Load
    ///
    /// # Errors
    /// If cannot load
    fn load(&mut self, name: ParamId, val: Value) -> Result<()>;

    /// Assign
    ///
    /// # Errors
    /// If cannot assign
    fn assign(&mut self, name: ParamId, val: Value) -> Result<()>;

    /// Dump
    ///
    /// # Errors
    /// If cannot dump
    fn dump(&self, name: ParamId) -> Result<Value>;

    /// Load owned
    ///
    /// # Errors
    /// If cannot load
    fn load_owned(&mut self, name: ParamId, val: GenericOwnedProp) -> Result<()>;

    /// Assign owned
    ///
    /// # Errors
    /// If cannot assig
    fn assign_owned(&mut self, name: ParamId, val: GenericOwnedProp) -> Result<()>;

    /// Dump owned
    ///
    /// # Errors
    /// If cannot dump as owned
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
