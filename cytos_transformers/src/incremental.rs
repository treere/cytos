use cytos::architecture::{Prop, Result, Stepper};
use cytos_derive::CytosNode;

#[derive(CytosNode, Default)]
pub struct IncrementalGenerator {
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
