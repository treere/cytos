use cytos::{Prop, Stepper, loader::DynamicLoadingRegistryWrapper};
use cytos_derive::CytosNode;

#[derive(CytosNode, Default)]
struct And {
    #[input]
    op1: Prop<bool>,

    #[input]
    op2: Prop<bool>,

    #[output]
    output: Prop<bool>,
}

impl Stepper for And {
    fn step(&mut self) -> cytos::Result<()> {
        *self.output = *self.op1 && *self.op2;
        Ok(())
    }
}

#[derive(CytosNode, Default)]
struct Or {
    #[input]
    op1: Prop<bool>,

    #[input]
    op2: Prop<bool>,

    #[output]
    output: Prop<bool>,
}

impl Stepper for Or {
    fn step(&mut self) -> cytos::Result<()> {
        *self.output = *self.op1 || *self.op2;
        Ok(())
    }
}

#[derive(CytosNode, Default)]
struct Xor {
    #[input]
    op1: Prop<bool>,

    #[input]
    op2: Prop<bool>,

    #[output]
    output: Prop<bool>,
}

impl Stepper for Xor {
    fn step(&mut self) -> cytos::Result<()> {
        *self.output = *self.op1 ^ *self.op2;
        Ok(())
    }
}

#[derive(CytosNode, Default)]
struct Not {
    #[input]
    op1: Prop<bool>,

    #[output]
    output: Prop<bool>,
}

impl Stepper for Not {
    fn step(&mut self) -> cytos::Result<()> {
        *self.output = !*self.op1;
        Ok(())
    }
}

pub fn load_registry(registry: &mut DynamicLoadingRegistryWrapper) {
    registry
        .add("And", And::default)
        .add("Or", Or::default)
        .add("Xor", Xor::default)
        .add("Not", Not::default);
}
