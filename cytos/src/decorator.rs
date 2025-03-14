use crate::{node::Node, Prop, Stepper, Transformer};

struct SkipDecorator {
    skip: Prop<bool>,

    node: Node,
}

impl Stepper for SkipDecorator {
    fn step(&mut self) -> crate::Result<()> {
        if *self.skip {
            Ok(())
        } else {
            self.node.step()
        }
    }

    fn initialize(&mut self) -> crate::Result<()> {
        self.node.initialize()
    }

    fn terminate(&mut self) -> crate::Result<()> {
        self.node.terminate()
    }
}

impl Transformer for SkipDecorator {
    fn link(&mut self, name: crate::ParamId, val: crate::props::GenericProp) -> crate::Result<()> {
        if name == crate::ParamId::from("step".to_owned()) {
            self.skip.link_value(val)
        } else {
            self.node.link(name, val)
        }
    }

    fn load(&mut self, name: crate::ParamId, val: crate::Value) -> crate::Result<()> {
        if name == crate::ParamId::from("step".to_owned()) {
            self.skip.load(val)
        } else {
            self.node.load(name, val)
        }
    }

    fn assign(&mut self, name: crate::ParamId, val: crate::Value) -> crate::Result<()> {
        if name == crate::ParamId::from("step".to_owned()) {
            self.skip.assign(val)
        } else {
            self.node.assign(name, val)
        }
    }

    fn dump(&self, name: crate::ParamId) -> crate::Result<crate::Value> {
        if name == crate::ParamId::from("step".to_owned()) {
            self.skip.dump()
        } else {
            self.node.dump(name)
        }
    }

    fn load_owned(
        &mut self,
        name: crate::ParamId,
        val: crate::GenericOwnedProp,
    ) -> crate::Result<()> {
        if name == crate::ParamId::from("step".to_owned()) {
            self.skip.load_owned_generic(val)
        } else {
            self.node.load_owned(name, val)
        }
    }

    fn assign_owned(
        &mut self,
        name: crate::ParamId,
        val: crate::GenericOwnedProp,
    ) -> crate::Result<()> {
        if name == crate::ParamId::from("step".to_owned()) {
            self.skip.assign_owned_generic(val)
        } else {
            self.node.assign_owned(name, val)
        }
    }

    fn dump_owned(&self, name: crate::ParamId) -> crate::Result<crate::GenericOwnedProp> {
        if name == crate::ParamId::from("step".to_owned()) {
            Ok(self.skip.into_owned_generic())
        } else {
            self.node.dump_owned(name)
        }
    }

    fn output(&self, val: crate::ParamId) -> Option<crate::props::GenericProp> {
        self.node.output(val)
    }

    fn input(&self, val: crate::ParamId) -> Option<crate::props::GenericProp> {
        if val == crate::ParamId::from("step".to_owned()) {
            Some(self.skip.as_generic())
        } else {
            self.node.input(val)
        }
    }

    fn input_names(&self) -> Vec<crate::ParamId> {
        let mut r = self.node.input_names();
        r.push(crate::ParamId::from("step".to_owned()));
        r
    }

    fn output_names(&self) -> Vec<crate::ParamId> {
        self.node.output_names()
    }
}
