use crate::architecture::{Result, Stepper, Transformer};

#[derive(Default)]
pub struct Node {}

impl Stepper for Node {
    fn step(&mut self) -> Result<()> {
        todo!()
    }
}

impl Transformer for Node {
    fn link(
        &mut self,
        _name: crate::architecture::ParamId,
        _val: crate::architecture::props::GenericOutputProp,
    ) -> Result<()> {
        todo!()
    }

    fn load(
        &mut self,
        _name: crate::architecture::ParamId,
        _val: crate::architecture::Value,
    ) -> Result<()> {
        todo!()
    }

    fn dump(&self, _name: crate::architecture::ParamId) -> Result<crate::architecture::Value> {
        todo!()
    }

    fn load_owned(
        &mut self,
        _name: crate::architecture::ParamId,
        _val: crate::architecture::GenericOwnedProp,
    ) -> Result<()> {
        todo!()
    }

    fn dump_owned(
        &self,
        _name: crate::architecture::ParamId,
    ) -> Result<crate::architecture::GenericOwnedProp> {
        todo!()
    }

    fn output(
        &self,
        _val: crate::architecture::ParamId,
    ) -> Option<crate::architecture::props::GenericOutputProp> {
        todo!()
    }

    fn input(
        &self,
        _val: crate::architecture::ParamId,
    ) -> Option<crate::architecture::props::GenericInputProp> {
        todo!()
    }

    fn input_names(&self) -> Vec<crate::architecture::ParamId> {
        todo!()
    }

    fn output_names(&self) -> Vec<crate::architecture::ParamId> {
        todo!()
    }
}
