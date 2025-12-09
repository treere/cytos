use cytos::{Prop, Stepper, loader::DynamicLoadingRegistryWrapper};
use cytos_derive::CytosNode;

#[derive(CytosNode, Default)]
struct And {
    #[cytos(input)]
    op1: Prop<bool>,

    #[cytos(input)]
    op2: Prop<bool>,

    #[cytos(output)]
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
    #[cytos(input)]
    op1: Prop<bool>,

    #[cytos(input)]
    op2: Prop<bool>,

    #[cytos(output)]
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
    #[cytos(input)]
    op1: Prop<bool>,

    #[cytos(input)]
    op2: Prop<bool>,

    #[cytos(output)]
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
    #[cytos(input)]
    op1: Prop<bool>,

    #[cytos(output)]
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
