use cytos::{Prop, Result, Stepper, loader::DynamicLoadingRegistryWrapper, props::Ownable};
use cytos_derive::CytosNode;
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Display;

#[derive(CytosNode, Default)]
struct Print<T>
where
    T: Ownable + Display + Default + DeserializeOwned + Serialize + 'static,
{
    #[cytos(input)]
    name: Prop<String>,

    #[cytos(input)]
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
        .add("PrintString", Print::<String>::default)
        .add("PrintI8", Print::<i8>::default)
        .add("PrintI16", Print::<i16>::default)
        .add("PrintI32", Print::<i32>::default)
        .add("PrintI64", Print::<i64>::default)
        .add("PrintU8", Print::<u8>::default)
        .add("PrintU16", Print::<u16>::default)
        .add("PrintU32", Print::<u32>::default)
        .add("PrintUSize", Print::<usize>::default)
        .add("PrintF32", Print::<f32>::default)
        .add("PrintBool", Print::<bool>::default)
        .add("PrintChar", Print::<char>::default);
}
