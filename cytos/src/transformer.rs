//! Transformer module defining the core traits for processing nodes.
//!
//! This module provides the `Stepper` and `Transformer` traits that define
//! the interface for nodes in the processing pipeline.

use crate::props::GenericPropInterface;

use super::{ParamId, Result};

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
pub trait PropInspector: Stepper {
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

    /// Returns a reference to the metadata for this node instance.
    ///
    /// # Returns
    ///
    /// A reference to the node's metadata.
    fn metadata(&self) -> &super::NodeMetadata;
}
