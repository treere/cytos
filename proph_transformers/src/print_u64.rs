use proph::architecture::{Done, InputProp, Stepper};
use proph_derive::ProphNode;

#[derive(ProphNode, Default)]
pub struct PrintU64 {
    input: InputProp<u64>,
}

impl Stepper for PrintU64 {
    fn step(&mut self) -> Done {
        println!("{}", self.input.get());
        Ok(())
    }
}
