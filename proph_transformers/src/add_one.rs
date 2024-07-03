use proph::architecture::{InputProp, OutputProp, Result, Stepper};
use proph_derive::TransFn;

#[derive(TransFn)]
pub struct AddValue {
    input: InputProp<u64>,
    increment: InputProp<u64>,
    output: OutputProp<u64>,
}

impl Default for AddValue {
    fn default() -> Self {
        AddValue {
            input: InputProp::default(),
            increment: InputProp::new(1),
            output: OutputProp::default(),
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
