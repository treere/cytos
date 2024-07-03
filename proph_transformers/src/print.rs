use proph::architecture::{InputProp, Result, Stepper};
use proph_derive::TransFn;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Display;

#[derive(TransFn, Default)]
pub struct Print<T>
where
    T: Display + Default + DeserializeOwned + Serialize + 'static,
{
    name: InputProp<String>,
    input: InputProp<T>,
}

impl<T: Display + Default + DeserializeOwned + Serialize + 'static> Stepper for Print<T> {
    fn step(&mut self) -> Result<()> {
        println!("{} = {}", *self.name, *self.input);
        Ok(())
    }
}
