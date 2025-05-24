use cytos::{loader::DynamicLoadingRegistryWrapper, Prop, Result, Stepper};
use cytos_derive::CytosNode;

#[derive(CytosNode)]
struct AddValue {
    #[input]
    input: Prop<u64>,

    #[input]
    increment: Prop<u64>,

    #[output]
    output: Prop<u64>,
}

impl Default for AddValue {
    fn default() -> Self {
        Self {
            input: Prop::default(),
            increment: Prop::new(1),
            output: Prop::default(),
        }
    }
}

impl Stepper for AddValue {
    fn step(&mut self) -> Result<()> {
        *self.output = *self.input + *self.increment;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starts_at_zero() {
        let add = AddValue::default();

        assert_eq!(*add.output, 0);
    }

    #[test]
    fn test_first_add_is_one() {
        let mut add = AddValue::default();

        add.step().expect("canont fail");

        assert_eq!(*add.output, 1);
    }
}

#[derive(CytosNode, Default)]
struct IncrementalGenerator {
    #[output]
    output: Prop<u64>,
}

impl Stepper for IncrementalGenerator {
    fn step(&mut self) -> Result<()> {
        *self.output += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests2 {
    use super::*;

    #[test]
    fn test_create_at_zero() {
        let incremental = IncrementalGenerator::default();

        assert_eq!(*incremental.output, 0)
    }

    #[test]
    fn test_increment_once() {
        let mut incremental = IncrementalGenerator::default();

        incremental.step().expect("cannot fail");
        assert_eq!(*incremental.output, 1)
    }

    #[test]
    fn test_increment_twice() {
        let mut incremental = IncrementalGenerator::default();

        incremental.step().expect("cannot fail");
        incremental.step().expect("cannot fail");
        assert_eq!(*incremental.output, 2)
    }
}

pub fn load_registry(registry: &mut DynamicLoadingRegistryWrapper) {
    registry
        .add("IncrementalGenerator", IncrementalGenerator::default)
        .add("AddValue", AddValue::default);
}
