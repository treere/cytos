use proph::architecture::{Done, InputProp, Stepper};
use proph_derive::TransFn;

#[derive(TransFn, Default)]
pub struct PrintU64 {
    input: InputProp<u64>,
}

impl Stepper for PrintU64 {
    fn initialize(&mut self) -> Done {
        Ok(())
    }

    fn step(&mut self) -> Done {
        println!("{}", self.input.get());
        Ok(())
    }
}
