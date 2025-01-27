use cytos::{loader::DynamicLoadingRegistryWrapper, props::Ownable, Prop, Result, Stepper};
use cytos_derive::CytosNode;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Display;

#[derive(CytosNode, Default)]
struct Print<T>
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

pub extern "C" fn load_registry(registry: &mut DynamicLoadingRegistryWrapper) {
    registry
        .add("PrintU64", Print::<u64>::default)
        .add("PrintF64", Print::<f64>::default)
        .add("PrintString", Print::<String>::default);
}
