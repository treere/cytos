use std::fmt::Display;

use cytos::{Prop, Stepper, loader::DynamicLoadingRegistryWrapper, props::Ownable};
use cytos_derive::CytosNode;
use serde::{Serialize, de::DeserializeOwned};

#[derive(CytosNode, Default)]
struct Lt<T>
where
    T: Ownable + Display + Default + DeserializeOwned + Serialize + 'static + PartialOrd,
{
    #[cytos(input)]
    op1: Prop<T>,

    #[cytos(input)]
    op2: Prop<T>,

    #[cytos(output)]
    output: Prop<bool>,
}

impl<T> Stepper for Lt<T>
where
    T: Ownable + Display + Default + DeserializeOwned + Serialize + 'static + PartialOrd,
{
    fn step(&mut self) -> cytos::Result<()> {
        *self.output = *self.op1 < *self.op2;
        Ok(())
    }
}

#[derive(CytosNode, Default)]
struct Lte<T>
where
    T: Ownable + Display + Default + DeserializeOwned + Serialize + 'static + PartialOrd,
{
    #[cytos(input)]
    op1: Prop<T>,

    #[cytos(input)]
    op2: Prop<T>,

    #[cytos(output)]
    output: Prop<bool>,
}

impl<T> Stepper for Lte<T>
where
    T: Ownable + Display + Default + DeserializeOwned + Serialize + 'static + PartialOrd,
{
    fn step(&mut self) -> cytos::Result<()> {
        *self.output = *self.op1 <= *self.op2;
        Ok(())
    }
}

#[derive(CytosNode, Default)]
struct Gt<T>
where
    T: Ownable + Display + Default + DeserializeOwned + Serialize + 'static + PartialOrd,
{
    #[cytos(input)]
    op1: Prop<T>,

    #[cytos(input)]
    op2: Prop<T>,

    #[cytos(output)]
    output: Prop<bool>,
}

impl<T> Stepper for Gt<T>
where
    T: Ownable + Display + Default + DeserializeOwned + Serialize + 'static + PartialOrd,
{
    fn step(&mut self) -> cytos::Result<()> {
        *self.output = *self.op1 > *self.op2;
        Ok(())
    }
}

#[derive(CytosNode, Default)]
struct Gte<T>
where
    T: Ownable + Display + Default + DeserializeOwned + Serialize + 'static + PartialOrd,
{
    #[cytos(input)]
    op1: Prop<T>,

    #[cytos(input)]
    op2: Prop<T>,

    #[cytos(output)]
    output: Prop<bool>,
}

impl<T> Stepper for Gte<T>
where
    T: Ownable + Display + Default + DeserializeOwned + Serialize + 'static + PartialOrd,
{
    fn step(&mut self) -> cytos::Result<()> {
        *self.output = *self.op1 >= *self.op2;
        Ok(())
    }
}

#[derive(CytosNode, Default)]
struct Eq<T>
where
    T: Ownable + Display + Default + DeserializeOwned + Serialize + 'static + PartialEq,
{
    #[cytos(input)]
    op1: Prop<T>,

    #[cytos(input)]
    op2: Prop<T>,

    #[cytos(output)]
    output: Prop<bool>,
}

impl<T> Stepper for Eq<T>
where
    T: Ownable + Display + Default + DeserializeOwned + Serialize + 'static + PartialEq,
{
    fn step(&mut self) -> cytos::Result<()> {
        *self.output = *self.op1 == *self.op2;
        Ok(())
    }
}

macro_rules! load_reg {
    ($id: ident, $ty: ty) => {
        $id.add(stringify!(Eq$ty), Eq::<$ty>::default)
            .add(stringify!(Lt$ty), Lt::<$ty>::default)
            .add(stringify!(Lte$ty), Lte::<$ty>::default)
            .add(stringify!(Gt$ty), Gt::<$ty>::default)
            .add(stringify!(Gte$ty), Gte::<$ty>::default);
    };
}

pub fn load_registry(registry: &mut DynamicLoadingRegistryWrapper) {
    load_reg!(registry, bool);
    load_reg!(registry, u8);
    load_reg!(registry, u16);
    load_reg!(registry, u32);
    load_reg!(registry, u64);
    load_reg!(registry, usize);
    load_reg!(registry, i8);
    load_reg!(registry, i16);
    load_reg!(registry, i32);
    load_reg!(registry, i64);
    load_reg!(registry, isize);
    load_reg!(registry, f32);
    load_reg!(registry, f64);
}
