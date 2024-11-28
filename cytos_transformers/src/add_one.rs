use cytos::{Prop, Result, Stepper};
use cytos_derive::CytosNode;

#[derive(CytosNode)]
pub struct AddValue {
    #[input]
    input: Prop<u64>,

    #[input]
    increment: Prop<u64>,

    #[output]
    output: Prop<u64>,
}

impl Default for AddValue {
    fn default() -> Self {
        AddValue {
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
