use crate::architecture::props::GenericProp;
use crate::architecture::{GenericOwnedProp, ParamId, Prop, Result, Stepper, Transformer, Value};

#[derive(Default)]
pub struct Empty {}

impl Stepper for Empty {
    fn step(&mut self) -> Result<()> {
        todo!()
    }
}

impl Transformer for Empty {
    fn link(&mut self, _name: ParamId, _val: GenericProp) -> Result<()> {
        Err("no link".into())
    }

    fn assign(&mut self, _name: ParamId, _val: Value) -> Result<()> {
        Err("load".into())
    }

    fn load(&mut self, _name: ParamId, _val: Value) -> Result<()> {
        Err("load".into())
    }

    fn dump(&self, _name: ParamId) -> Result<Value> {
        Err("dump".into())
    }

    fn assign_owned(&mut self, _name: ParamId, _val: GenericOwnedProp) -> Result<()> {
        Err("load_owned".into())
    }

    fn load_owned(&mut self, _name: ParamId, _val: GenericOwnedProp) -> Result<()> {
        Err("load_owned".into())
    }

    fn dump_owned(&self, _name: ParamId) -> Result<GenericOwnedProp> {
        Err("no data".into())
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
    fn link(&mut self, name: ParamId, val: GenericProp) -> Result<()> {
        match name {
            ParamId(0) => self.input.link_value(val),
            _ => Err("".into()),
        }
    }

    fn assign(&mut self, name: ParamId, val: Value) -> Result<()> {
        match name {
            ParamId(0) => self.input.assign(val),
            _ => Err("".into()),
        }
    }

    fn load(&mut self, name: ParamId, val: Value) -> Result<()> {
        match name {
            ParamId(0) => self.input.load(val),
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
}
