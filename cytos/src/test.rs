use std::collections::HashMap;

use crate::props::GenericProp;
use crate::{MetadataProvider, NodeMetadata, ParamId, Prop, Result, Stepper, Transformer};

/// An empty transformer implementation for testing purposes.
///
/// This struct implements both `Stepper` and `Transformer` traits but always returns errors
/// for operations, making it useful for testing error handling in the system.
#[derive(Default)]
pub struct Empty {}

impl Stepper for Empty {
    fn step(&mut self) -> Result<()> {
        todo!()
    }
}

impl Transformer for Empty {
    fn get_prop(&self, _val: ParamId) -> Option<&dyn crate::props::GenericPropInterface> {
        None
    }

    fn get_prop_mut(
        &mut self,
        _val: ParamId,
    ) -> Option<&mut dyn crate::props::GenericPropInterface> {
        None
    }

    fn output(&self, _val: ParamId) -> Option<GenericProp> {
        None
    }

    fn input(&self, _val: ParamId) -> Option<GenericProp> {
        None
    }

    fn input_names(&self) -> Vec<ParamId> {
        vec![]
    }

    fn output_names(&self) -> Vec<ParamId> {
        vec![]
    }
}

impl MetadataProvider for Empty {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            name: "Empty".to_string(),
            description: "Test empty node".to_string(),
            params: HashMap::new(),
        }
    }
}

#[derive(Default)]
pub struct Constant {
    input: Prop<i32>,
    output: Prop<i32>,
}

impl Stepper for Constant {
    fn step(&mut self) -> Result<()> {
        *self.output = *self.input;
        Ok(())
    }
}

impl Transformer for Constant {
    fn output(&self, name: ParamId) -> Option<GenericProp> {
        match name {
            ParamId(1) => Some(self.output.as_generic()),
            _ => None,
        }
    }

    fn input(&self, name: ParamId) -> Option<GenericProp> {
        match name {
            ParamId(0) => Some(self.input.as_generic()),
            _ => None,
        }
    }

    fn input_names(&self) -> Vec<ParamId> {
        vec![ParamId(0)]
    }

    fn output_names(&self) -> Vec<ParamId> {
        vec![ParamId(1)]
    }

    fn get_prop(&self, val: ParamId) -> Option<&dyn crate::props::GenericPropInterface> {
        match val {
            ParamId(0) => Some(&self.input),
            ParamId(1) => Some(&self.output),
            _ => None,
        }
    }

    fn get_prop_mut(
        &mut self,
        val: ParamId,
    ) -> Option<&mut dyn crate::props::GenericPropInterface> {
        match val {
            ParamId(0) => Some(&mut self.input),
            ParamId(1) => Some(&mut self.output),
            _ => None,
        }
    }
}
