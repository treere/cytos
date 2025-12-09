use cytos::{Prop, Result, Stepper, loader::DynamicLoadingRegistryWrapper, props::Ownable};
use cytos_derive::CytosNode;
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Display;

#[derive(CytosNode, Default)]
struct WebSender<T>
where
    T: Ownable + Display + Default + DeserializeOwned + Serialize + 'static,
{
    #[cytos(input)]
    url: Prop<String>,

    #[cytos(input)]
    header: Prop<Option<(String, String)>>,

    #[cytos(input)]
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
        .add("WebSenderF64", WebSender::<f64>::default)
        .add("WebSenderString", WebSender::<String>::default)
        .add("WebSenderI8", WebSender::<i8>::default)
        .add("WebSenderI16", WebSender::<i16>::default)
        .add("WebSenderI32", WebSender::<i32>::default)
        .add("WebSenderI64", WebSender::<i64>::default)
        .add("WebSenderU8", WebSender::<u8>::default)
        .add("WebSenderU16", WebSender::<u16>::default)
        .add("WebSenderU32", WebSender::<u32>::default)
        .add("WebSenderUSize", WebSender::<usize>::default)
        .add("WebSenderF32", WebSender::<f32>::default)
        .add("WebSenderBool", WebSender::<bool>::default)
        .add("WebSenderChar", WebSender::<char>::default);
}
