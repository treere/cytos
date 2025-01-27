use cytos::{loader::DynamicLoadingRegistryWrapper, props::Ownable, Prop, Result, Stepper};
use cytos_derive::CytosNode;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Display;

#[derive(CytosNode, Default)]
struct WebSender<T>
where
    T: Ownable + Display + Default + DeserializeOwned + Serialize + 'static,
{
    #[input]
    url: Prop<String>,

    #[input]
    header: Prop<Option<(String, String)>>,

    #[input]
    input: Prop<T>,
}

impl<T: Ownable + Display + Default + DeserializeOwned + Serialize + 'static> Stepper
    for WebSender<T>
{
    fn step(&mut self) -> Result<()> {
        let post = reqwest::blocking::Client::new().post(&*self.url);

        let post = if let Some((k, v)) = &*self.header {
            post.header(k, v)
        } else {
            post
        };

        post.json(&*self.input).send()?;

        Ok(())
    }
}

pub fn load_registry(registry: &mut DynamicLoadingRegistryWrapper) {
    registry
        .add("WebSenderU64", WebSender::<u64>::default)
        .add("WebSenderF64", WebSender::<f64>::default);
}
