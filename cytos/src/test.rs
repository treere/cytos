use std::collections::HashMap;

use crate::props::GenericProp;
use crate::{
    GenericOwnedProp, MetadataProvider, NodeMetadata, ParamId, Prop, Result, Stepper, Transformer,
    Value,
};

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
    fn dump(&self, _name: ParamId) -> Result<Value> {
        Err("dump".into())
    }

    fn load_owned(&mut self, _name: ParamId, _val: GenericOwnedProp) -> Result<()> {
        Err("load_owned".into())
    }

    fn assign_owned(&mut self, _name: ParamId, _val: GenericOwnedProp) -> Result<()> {
        Err("load_owned".into())
    }

    fn dump_owned(&self, _name: ParamId) -> Result<GenericOwnedProp> {
        Err("no data".into())
    }

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

    fn assign(&mut self, _: ParamId, _: Value) -> Result<()> {
        Err("no data".into())
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
    fn assign(&mut self, name: ParamId, val: Value) -> Result<()> {
        match name {
            ParamId(0) => self.input.assign(val),
            _ => Err("".into()),
        }
    }

    fn dump(&self, name: ParamId) -> Result<Value> {
        match name {
            ParamId(0) => self.input.dump(),
            ParamId(1) => self.output.dump(),
            _ => Err("".into()),
        }
    }

    fn assign_owned(&mut self, name: ParamId, val: GenericOwnedProp) -> Result<()> {
        match name {
            ParamId(0) => self.input.assign_owned_generic(val),
            _ => Err("".into()),
        }
    }

    fn load_owned(&mut self, name: ParamId, val: GenericOwnedProp) -> Result<()> {
        match name {
            ParamId(0) => self.input.load_owned_generic(val),
            _ => Err("".into()),
        }
    }

    fn dump_owned(&self, name: ParamId) -> Result<GenericOwnedProp> {
        match name {
            ParamId(0) => Ok(self.input.into_owned_generic()),
            ParamId(1) => Ok(self.output.into_owned_generic()),
            _ => Err("".into()),
        }
    }

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
            ParamId(0) => Some(&self.output),
            ParamId(1) => Some(&self.input),
            _ => None,
        }
    }

    fn get_prop_mut(
        &mut self,
        val: ParamId,
    ) -> Option<&mut dyn crate::props::GenericPropInterface> {
        match val {
            ParamId(0) => Some(&mut self.output),
            ParamId(1) => Some(&mut self.input),
            _ => None,
        }
    }
}
