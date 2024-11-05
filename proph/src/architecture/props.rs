//! Properties
use serde::{de::DeserializeOwned, Serialize};
use std::{any::Any, cell::UnsafeCell, rc::Rc};

use super::{Result, Value};

/// Internal prop structure
#[derive(Default)]
struct Prop<T>(Rc<UnsafeCell<T>>);

impl<T: 'static> Prop<T> {
    /// Create a prop
    pub fn new(val: T) -> Self {
        Self(Rc::new(UnsafeCell::new(val)))
    }

    /// Link that prop to another prop
    pub fn link_value(&mut self, val: GenericOutputProp) -> Result<()> {
        if let Ok(v) = val.0 .0.downcast::<UnsafeCell<T>>() {
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
        self.0.link_value(val)
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
