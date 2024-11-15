use proph::architecture::{props::Ownable, InputProp, Result, Stepper};
use proph_derive::ProphNode;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Display;

#[derive(ProphNode, Default)]
pub struct Print<T>
where
    T: Ownable + Display + Default + DeserializeOwned + Serialize + 'static,
{
    name: InputProp<String>,
    input: InputProp<T>,
}

impl<T: Ownable + Display + Default + DeserializeOwned + Serialize + 'static> Stepper for Print<T> {
    fn step(&mut self) -> Result<()> {
        println!("{} = {}", *self.name, *self.input);
        Ok(())
    }
}
