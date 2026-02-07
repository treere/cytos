//! Transformer module defining the core traits for processing nodes.
//!
//! This module provides the `Stepper` and `Transformer` traits that define
//! the interface for nodes in the processing pipeline.

use crate::props::GenericPropInterface;

use super::{
    ParamId, Result, Value,
    props::{GenericOwnedProp, GenericProp},
};

/// A trait for objects that can perform computation steps.
///
/// Implementors of this trait can be initialized, stepped through computations,
/// and terminated.
#[ptr_meta::pointee]
pub trait Stepper {
    /// Initializes the stepper before processing begins.
    ///
    /// This method is called once before any steps are performed.
    /// The default implementation does nothing.
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails.
    fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    /// Performs one computation step.
    ///
    /// This method is called repeatedly to process data.
    ///
    /// # Errors
    ///
    /// Returns an error if the step cannot be processed.
    fn step(&mut self) -> Result<()>;

    /// Terminates the stepper after processing is complete.
    ///
    /// This method is called once after all steps are done.
    /// The default implementation does nothing.
    ///
    /// # Errors
    ///
    /// Returns an error if termination fails.
    fn terminate(&mut self) -> Result<()> {
        Ok(())
    }
}

/// A trait for objects that can transform data with inputs and outputs.
///
/// Transformers can link to other transformers, load configuration values,
/// and provide access to their input and output parameters.
pub trait Transformer: Stepper {
    /// Assigns a runtime value to a parameter.
    ///
    /// # Arguments
    ///
    /// * `name` - The parameter ID to assign the value to.
    /// * `val` - The value to assign.
    ///
    /// # Errors
    ///
    /// Returns an error if the value cannot be assigned.
    fn assign(&mut self, name: ParamId, val: Value) -> Result<()>;

    /// Dumps the current value of a parameter.
    ///
    /// # Arguments
    ///
    /// * `name` - The parameter ID to dump.
    ///
    /// # Errors
    ///
    /// Returns an error if the value cannot be dumped.
    fn dump(&self, name: ParamId) -> Result<Value>;

    /// Loads an owned property for a parameter.
    ///
    /// # Arguments
    ///
    /// * `name` - The parameter ID to load the property for.
    /// * `val` - The owned property to load.
    ///
    /// # Errors
    ///
    /// Returns an error if the property cannot be loaded.
    fn load_owned(&mut self, name: ParamId, val: GenericOwnedProp) -> Result<()>;

    /// Assigns an owned property to a parameter.
    ///
    /// # Arguments
    ///
    /// * `name` - The parameter ID to assign the property to.
    /// * `val` - The owned property to assign.
    ///
    /// # Errors
    ///
    /// Returns an error if the property cannot be assigned.
    fn assign_owned(&mut self, name: ParamId, val: GenericOwnedProp) -> Result<()>;

    /// Dumps the current owned property of a parameter.
    ///
    /// # Arguments
    ///
    /// * `name` - The parameter ID to dump.
    ///
    /// # Errors
    ///
    /// Returns an error if the property cannot be dumped.
    fn dump_owned(&self, name: ParamId) -> Result<GenericOwnedProp>;

    /// Gets a reference to a property by parameter ID.
    ///
    /// # Arguments
    ///
    /// * `val` - The parameter ID of the property.
    ///
    /// # Returns
    ///
    /// A reference to the property as a `GenericPropInterface` if it exists, `None` otherwise.
    fn get_prop(&self, val: ParamId) -> Option<&dyn GenericPropInterface>;

    /// Gets a mutable reference to a property by parameter ID.
    ///
    /// # Arguments
    ///
    /// * `val` - The parameter ID of the property.
    ///
    /// # Returns
    ///
    /// A mutable reference to the property as a `GenericPropInterface` if it exists, `None` otherwise.
    fn get_prop_mut(&mut self, val: ParamId) -> Option<&mut dyn GenericPropInterface>;

    /// Gets an output property by parameter name.
    ///
    /// # Arguments
    ///
    /// * `val` - The parameter ID of the output.
    ///
    /// # Returns
    ///
    /// The output property if it exists, `None` otherwise.
    fn output(&self, val: ParamId) -> Option<GenericProp>;

    /// Gets an input property by parameter name.
    ///
    /// # Arguments
    ///
    /// * `val` - The parameter ID of the input.
    ///
    /// # Returns
    ///
    /// The input property if it exists, `None` otherwise.
    fn input(&self, val: ParamId) -> Option<GenericProp>;

    /// Gets the names of all input parameters.
    ///
    /// # Returns
    ///
    /// A vector of parameter IDs for all inputs.
    fn input_names(&self) -> Vec<ParamId>;

    /// Gets the names of all output parameters.
    ///
    /// # Returns
    ///
    /// A vector of parameter IDs for all outputs.
    fn output_names(&self) -> Vec<ParamId>;
}
