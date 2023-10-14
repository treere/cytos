use proph::architecture::{Done, InputProp, Stepper};
use proph_derive::TransFn;
use std::fmt::Display;

#[derive(TransFn, Default)]
pub struct Print<T: Display + Default + 'static> {
    name: InputProp<String>,
    input: InputProp<T>,
}

impl<T: Display + Default + 'static> Stepper for Print<T> {
    fn initialize(&mut self) -> Done {
        Ok(())
    }

    fn step(&mut self) -> Done {
        println!("{} = {}", self.name.get(), self.input.get());
        Ok(())
    }
}
