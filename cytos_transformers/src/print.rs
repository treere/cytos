use cytos::architecture::{props::Ownable, Prop, Result, Stepper};
use cytos_derive::CytosNode;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Display;

#[derive(CytosNode, Default)]
pub struct Print<T>
where
    T: Ownable + Display + Default + DeserializeOwned + Serialize + 'static,
{
    #[input]
    name: Prop<String>,

    #[input]
    input: Prop<T>,
}

impl<T: Ownable + Display + Default + DeserializeOwned + Serialize + 'static> Stepper for Print<T> {
    fn step(&mut self) -> Result<()> {
        println!("{} = {}", *self.name, *self.input);
        Ok(())
    }
}
