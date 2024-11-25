use cytos::architecture::{Done, InputProp, Stepper};
use cytos_derive::CytosNode;

#[derive(CytosNode, Default)]
pub struct PrintU64 {
    input: InputProp<u64>,
}

impl Stepper for PrintU64 {
    fn step(&mut self) -> Done {
        println!("{}", self.input.get());
        Ok(())
    }
}
