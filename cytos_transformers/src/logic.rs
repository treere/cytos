/// Boolean logic transformer nodes for Cytos.
///
/// This module provides basic boolean logic operations: AND, OR, XOR, NOT.
/// All nodes take boolean inputs and produce boolean outputs.
use cytos::{Prop, Stepper, loader::DynamicLoadingRegistryWrapper};
use cytos_derive::CytosNode;

/// Boolean AND operation node.
/// Outputs `true` if both `op1` and `op2` are `true`.
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

/// Boolean OR operation node.
/// Outputs `true` if either `op1` or `op2` is `true`.
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

/// Boolean XOR operation node.
/// Outputs `true` if exactly one of `op1` or `op2` is `true`.
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

/// Boolean NOT operation node.
/// Outputs the logical negation of `op1`.
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

/// Registers the boolean logic nodes into the Cytos registry.
pub fn load_registry(registry: &mut DynamicLoadingRegistryWrapper) {
    registry
        .add("And", And::default)
        .add("Or", Or::default)
        .add("Xor", Xor::default)
        .add("Not", Not::default);
}
