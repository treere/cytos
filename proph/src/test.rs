use crate::architecture::props::{GenericInputProp, GenericOutputProp};
use crate::architecture::{GenericOwnedProp, ParamId, Result, Stepper, Transformer, Value};

#[derive(Default)]
pub struct Empty {}

impl Stepper for Empty {
    fn step(&mut self) -> Result<()> {
        todo!()
    }
}

impl Transformer for Empty {
    fn link(&mut self, _name: ParamId, _val: GenericOutputProp) -> Result<()> {
        Err("no link".into())
    }

    fn load(&mut self, _name: ParamId, _val: Value) -> Result<()> {
        Err("load".into())
    }

    fn dump(&self, _name: ParamId) -> Result<Value> {
        Err("dump".into())
    }

    fn load_owned(&mut self, _name: ParamId, _val: GenericOwnedProp) -> Result<()> {
        Err("load_owned".into())
    }

    fn dump_owned(&self, _name: ParamId) -> Result<GenericOwnedProp> {
        Err("no data".into())
    }

    fn output(&self, _val: ParamId) -> Option<GenericOutputProp> {
        None
    }

    fn input(&self, _val: ParamId) -> Option<GenericInputProp> {
        None
    }

    fn input_names(&self) -> Vec<ParamId> {
        vec![]
    }

    fn output_names(&self) -> Vec<ParamId> {
        vec![]
    }
}
