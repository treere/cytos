use cytos::{Prop, Result, Stepper, loader::DynamicLoadingRegistryWrapper, props::Ownable};
use cytos_derive::CytosNode;
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Display;

#[derive(CytosNode, Default)]
struct MarkovFilter<T>
where
    T: Ownable + Display + Default + DeserializeOwned + PartialOrd + Serialize + 'static,
{
    #[cytos(input)]
    input: Prop<T>,

    #[cytos(input)]
    low: Prop<T>,

    #[cytos(input)]
    high: Prop<T>,

    #[cytos(output)]
    output: Prop<bool>,
}

impl<T: Ownable + Display + Default + DeserializeOwned + Serialize + PartialOrd + 'static> Stepper
    for MarkovFilter<T>
{
    fn step(&mut self) -> Result<()> {
        if *self.output {
            *self.output = *self.input > *self.high;
        } else {
            *self.output = *self.low < *self.input;
        }
        Ok(())
    }
}

pub fn load_registry(registry: &mut DynamicLoadingRegistryWrapper) {
    registry.add("MarkovFilterF32", MarkovFilter::<f32>::default);
}
