use proph::architecture::{OutputProp, Result, Stepper};
use proph_derive::ProphNode;

#[derive(ProphNode, Default)]
pub struct IncrementalGenerator {
    output: OutputProp<u64>,
}

impl Stepper for IncrementalGenerator {
    fn step(&mut self) -> Result<()> {
        *self.output += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
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
