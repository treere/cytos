use cytos::{props::Ownable, Prop, Result, Stepper};
use cytos_derive::CytosNode;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Display;

#[derive(CytosNode, Default)]
pub struct WebSender<T>
where
    T: Ownable + Display + Default + DeserializeOwned + Serialize + 'static,
{
    #[input]
    url: Prop<String>,

    #[input]
    input: Prop<T>,
}

impl<T: Ownable + Display + Default + DeserializeOwned + Serialize + 'static> Stepper
    for WebSender<T>
{
    fn step(&mut self) -> Result<()> {
        reqwest::blocking::Client::new()
            .post(&*self.url)
            .json(&*self.input)
            .send()?;

        Ok(())
    }
}
