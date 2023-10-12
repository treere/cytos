use proph::architecture::{OutputProp, Stepper};
use proph_derive::TransFn;

#[derive(TransFn, Default)]
pub struct IncrementalGenerator {
    output: OutputProp<u64>,
}

impl Stepper for IncrementalGenerator {
    fn step(&mut self) -> Result<(), &'static str> {
        *self.output.set() += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        let mut incremental = IncrementalGenerator::default();
        incremental.step().expect("cannot fail");

        assert_eq!(*incremental.output.get(), 1)
    }
}
