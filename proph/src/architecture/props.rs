//! Properties
use serde::{de::DeserializeOwned, Serialize};
use std::{any::Any, cell::UnsafeCell, rc::Rc};

use super::{Result, Value};

/// Convert to and from a Send+Sync type
pub trait Ownable {
    type Value: Send + Sync + 'static;

    fn to_ownable(&self) -> Self::Value;

    fn from_owned(v: &Self::Value) -> Self;
}

impl Ownable for u8 {
    type Value = u8;

    fn to_ownable(&self) -> Self::Value {
        *self
    }

    fn from_owned(v: &Self::Value) -> Self {
        *v
    }
}
impl Ownable for u64 {
    type Value = u64;

    fn to_ownable(&self) -> Self::Value {
        *self
    }

    fn from_owned(v: &Self::Value) -> Self {
        *v
    }
}
impl Ownable for i32 {
    type Value = i32;

    fn to_ownable(&self) -> Self::Value {
        *self
    }

    fn from_owned(v: &Self::Value) -> Self {
        *v
    }
}
impl Ownable for f64 {
    type Value = f64;

    fn to_ownable(&self) -> Self::Value {
        *self
    }

    fn from_owned(v: &Self::Value) -> Self {
        *v
    }
}
impl Ownable for String {
    type Value = String;

    fn to_ownable(&self) -> Self::Value {
        self.clone()
    }

    fn from_owned(v: &Self::Value) -> Self {
        v.clone()
    }
}
impl Ownable for (u32, u32) {
    type Value = (u32, u32);

    fn to_ownable(&self) -> Self::Value {
        self.clone()
    }

    fn from_owned(v: &Self::Value) -> Self {
        v.clone()
    }
}
impl Ownable for std::time::Duration {
    type Value = std::time::Duration;

    fn to_ownable(&self) -> Self::Value {
        self.clone()
    }

    fn from_owned(v: &Self::Value) -> Self {
        v.clone()
    }
}

pub struct GenericOwnedProp(Box<dyn Any + Send + Sync + 'static>);

/// Internal prop structure
#[derive(Default)]
struct Prop<T>(Rc<UnsafeCell<T>>);

impl<T: 'static> Prop<T> {
    /// Create a prop
    pub fn new(val: T) -> Self {
        Self(Rc::new(UnsafeCell::new(val)))
    }

    /// Link that prop to another prop
    pub fn link_value(&mut self, val: GenericProp) -> Result<()> {
        if let Ok(v) = val.0.downcast::<UnsafeCell<T>>() {
            self.0 = v;
            Ok(())
        } else {
            Err("invalid type".into())
        }
    }

    /// Convert this prop to be a generic prop
    pub fn as_generic(&self) -> GenericProp {
        GenericProp(self.0.clone())
    }
}

impl<T: Ownable> Prop<T> {
    pub fn into_owned_generic(&self) -> GenericOwnedProp {
        GenericOwnedProp(Box::new(self.to_ownable()))
    }

    pub fn load_owned_generic(&mut self, val: GenericOwnedProp) -> Result<()> {
        if let Ok(v) = val.0.downcast::<T::Value>() {
            self.0 = Rc::new(UnsafeCell::new(Ownable::from_owned(&*v)));
            Ok(())
        } else {
            Err("invalid type".into())
        }
    }
}

impl<T> std::ops::Deref for Prop<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.get() }
    }
}

impl<T> std::ops::DerefMut for Prop<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0.get() }
    }
}

impl<T: 'static + DeserializeOwned> Prop<T> {
    /// Load a value into a prop
    pub fn load(&mut self, val: Value) -> Result<()> {
        let value = val.dump()?;
        self.0 = Rc::new(UnsafeCell::new(value));
        Ok(())
    }
}

impl<T: 'static + Serialize> Prop<T> {
    /// Dump the value of a prop
    fn dump(&self) -> Result<Value> {
        Value::load(&**self)
    }
}

/// An input property
///
/// This is a property that can be used for input
#[derive(Default)]
pub struct InputProp<T>(Prop<T>);

impl<T: 'static> InputProp<T> {
    /// Create an input prop
    pub fn new(val: T) -> Self {
        Self(Prop::new(val))
    }

    /// Link an input to an output
    pub fn link_value(&mut self, val: GenericOutputProp) -> Result<()> {
        self.0.link_value(val.0)
    }

    /// Convert this props to a generic prop
    pub fn as_generic(&self) -> GenericInputProp {
        GenericInputProp(self.0.as_generic())
    }
}

impl<T: 'static + DeserializeOwned> InputProp<T> {
    /// Load a value
    pub fn load(&mut self, val: Value) -> Result<()> {
        self.0.load(val)
    }
}

impl<T: 'static + Serialize> InputProp<T> {
    /// Dump a value
    pub fn dump(&self) -> Result<Value> {
        self.0.dump()
    }
}

impl<T: Ownable> InputProp<T> {
    pub fn into_owned_generic(&self) -> GenericOwnedProp {
        self.0.into_owned_generic()
    }
    pub fn load_owned_generic(&mut self, val: GenericOwnedProp) -> Result<()> {
        self.0.load_owned_generic(val)
    }
}

impl<T: 'static> std::ops::Deref for InputProp<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// OutputProp
///
/// This is a property that can be used for input
#[derive(Default)]
pub struct OutputProp<T>(Prop<T>);

impl<T: 'static> OutputProp<T> {
    /// Create an output prop
    pub fn new(val: T) -> Self {
        Self(Prop::new(val))
    }

    /// Convert an output prop to a generic prop
    pub fn as_generic(&self) -> GenericOutputProp {
        GenericOutputProp(self.0.as_generic())
    }
}

impl<T: 'static + Serialize> OutputProp<T> {
    /// Dump the value
    pub fn dump(&self) -> Result<Value> {
        self.0.dump()
    }
}

impl<T: Ownable> OutputProp<T> {
    pub fn into_owned_generic(&self) -> GenericOwnedProp {
        self.0.into_owned_generic()
    }
    pub fn load_owned_generic(&mut self, val: GenericOwnedProp) -> Result<()> {
        self.0.load_owned_generic(val)
    }
}

impl<T: 'static> std::ops::Deref for OutputProp<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: 'static> std::ops::DerefMut for OutputProp<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Generic prop
struct GenericProp(Rc<dyn Any>);

impl GenericProp {
    /// Verify if two generic props are the same
    pub fn is_same(&self, other: &GenericProp) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

/// Generic Property to be casted back
pub struct GenericOutputProp(GenericProp);

impl GenericOutputProp {
    /// Verify if two generic props are the same
    pub fn is_same(&self, other: &GenericInputProp) -> bool {
        self.0.is_same(&other.0)
    }
}

pub struct GenericInputProp(GenericProp);

impl GenericInputProp {
    /// Verify if two generic props are the same
    pub fn is_same(&self, other: &GenericOutputProp) -> bool {
        self.0.is_same(&other.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_props() {
        let output_prop = OutputProp::new(12);
        let gen = output_prop.as_generic();

        let mut input_prop = InputProp::new(1);
        input_prop.link_value(gen).expect("cannot link");

        assert_eq!(12, *input_prop);
    }

    #[test]
    fn test_dump_load_prop() {
        let prop = Prop::new(1);
        let dump = prop.dump().expect("cannot dump");

        let mut prop = Prop::new(2);
        prop.load(dump).expect("cannot load");

        assert_eq!(1, *prop);
    }

    #[test]
    fn test_multi_thread() {
        let prop = Prop::new(1);

        let gen = prop.into_owned_generic();

        std::thread::spawn(|| {
            let mut thread_prop = Prop::new(2);
            thread_prop.load_owned_generic(gen).expect("cannot link");

            assert_eq!(1, *thread_prop);
        })
        .join()
        .expect("cannot join");
    }
}
