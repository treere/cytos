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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::Empty;

    #[derive(Default)]
    struct TestStepper {
        initialized: bool,
        stepped: bool,
        terminated: bool,
    }

    impl Stepper for TestStepper {
        fn initialize(&mut self) -> Result<()> {
            self.initialized = true;
            Ok(())
        }

        fn step(&mut self) -> Result<()> {
            self.stepped = true;
            Ok(())
        }

        fn terminate(&mut self) -> Result<()> {
            self.terminated = true;
            Ok(())
        }
    }

    #[test]
    fn test_stepper_default_implementations() {
        let mut empty = Empty::default();

        // initialize and terminate have default implementations that return Ok(())
        assert!(empty.initialize().is_ok());
        assert!(empty.terminate().is_ok());
    }

    #[test]
    fn test_stepper_full_lifecycle() {
        let mut stepper = TestStepper::default();

        assert!(!stepper.initialized);
        assert!(!stepper.stepped);
        assert!(!stepper.terminated);

        stepper.initialize().expect("initialize should succeed");
        assert!(stepper.initialized);

        stepper.step().expect("step should succeed");
        assert!(stepper.stepped);

        stepper.terminate().expect("terminate should succeed");
        assert!(stepper.terminated);
    }

    #[test]
    fn test_stepper_step_returns_ok() {
        let mut stepper = Empty::default();
        let result = stepper.step();
        assert!(result.is_ok());
    }

    #[test]
    fn test_prop_inspector_with_empty() {
        let empty = Empty::default();

        // Empty has no props, so all ParamId values should return None
        assert!(empty.get_prop(ParamId(0)).is_none());
        assert!(empty.get_prop(ParamId(1)).is_none());
        assert!(empty.get_prop(ParamId(100)).is_none());
    }

    #[test]
    fn test_prop_inspector_mut_with_empty() {
        let mut empty = Empty::default();

        // Empty has no props, so all ParamId values should return None
        assert!(empty.get_prop_mut(ParamId(0)).is_none());
        assert!(empty.get_prop_mut(ParamId(1)).is_none());
        assert!(empty.get_prop_mut(ParamId(100)).is_none());
    }

    #[test]
    fn test_prop_inspector_metadata() {
        let empty = Empty::default();
        let metadata = empty.metadata();

        assert_eq!(metadata.name, "Empty");
        assert_eq!(metadata.description, "Test empty node");
        assert!(metadata.params.is_empty());
    }

    struct FailingStepper;

    impl Stepper for FailingStepper {
        fn step(&mut self) -> Result<()> {
            Err("step failed".into())
        }
    }

    #[test]
    fn test_stepper_step_error() {
        let mut stepper = FailingStepper;
        let result = stepper.step();
        assert!(result.is_err());
    }

    struct FailingInitStepper;

    impl Stepper for FailingInitStepper {
        fn initialize(&mut self) -> Result<()> {
            Err("init failed".into())
        }

        fn step(&mut self) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_stepper_initialize_error() {
        let mut stepper = FailingInitStepper;
        let result = stepper.initialize();
        assert!(result.is_err());
    }

    struct FailingTerminateStepper;

    impl Stepper for FailingTerminateStepper {
        fn step(&mut self) -> Result<()> {
            Ok(())
        }

        fn terminate(&mut self) -> Result<()> {
            Err("terminate failed".into())
        }
    }

    #[test]
    fn test_stepper_terminate_error() {
        let mut stepper = FailingTerminateStepper;
        let result = stepper.terminate();
        assert!(result.is_err());
    }

    #[test]
    fn test_stepper_initialize_default_success() {
        // Test that the default implementation of initialize returns Ok(())
        struct DefaultInitStepper;
        impl Stepper for DefaultInitStepper {
            fn step(&mut self) -> Result<()> {
                Ok(())
            }
        }

        let mut stepper = DefaultInitStepper;
        let result = stepper.initialize();
        assert!(result.is_ok());
    }

    #[test]
    fn test_stepper_terminate_default_success() {
        // Test that the default implementation of terminate returns Ok(())
        struct DefaultTerminateStepper;
        impl Stepper for DefaultTerminateStepper {
            fn step(&mut self) -> Result<()> {
                Ok(())
            }
        }

        let mut stepper = DefaultTerminateStepper;
        let result = stepper.terminate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_steps() {
        let mut stepper = TestStepper::default();
        stepper.initialize().expect("initialize should succeed");

        // Multiple steps should all succeed
        for _ in 0..10 {
            stepper.step().expect("step should succeed");
        }

        assert!(stepper.stepped);
        stepper.terminate().expect("terminate should succeed");
    }

    #[test]
    fn test_metadata_trait() {
        // Test implementing the MetadataProvider trait
        use crate::{MetadataProvider, NodeMetadata};

        struct TestMetadataProvider;

        impl MetadataProvider for TestMetadataProvider {
            fn metadata() -> NodeMetadata {
                NodeMetadata {
                    name: "Test".to_string(),
                    description: "Test description".to_string(),
                    params: vec![],
                }
            }
        }

        let metadata = TestMetadataProvider::metadata();
        assert_eq!(metadata.name, "Test");
    }
}
