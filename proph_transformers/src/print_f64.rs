use proph::architecture::{Done, InputProp, Stepper};
use proph_derive::ProphNode;

#[derive(ProphNode, Default)]
pub struct PrintF64 {
    name: InputProp<String>,
    input: InputProp<f64>,
}

impl Stepper for PrintF64 {
    fn step(&mut self) -> Done {
        println!("{} = {}", self.name.get(), self.input.get());
        Ok(())
    }
}
