//! Properties
use serde::{de::DeserializeOwned, Serialize};
use std::{any::Any, cell::UnsafeCell, ops::Deref, rc::Rc};

use super::{Result, Value};

#[derive(Default)]
struct Prop<T>(Rc<UnsafeCell<T>>);

impl<T: 'static> Prop<T> {
    pub fn new(val: T) -> Self {
        Self(Rc::new(UnsafeCell::new(val)))
    }

    pub fn change_value(&mut self, val: GenericOutputProp) -> Result<()> {
        if let Ok(v) = val.0 .0.downcast::<UnsafeCell<T>>() {
            self.0 = v;
            Ok(())
        } else {
            Err("invalid type")
        }
    }

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
    pub fn load(&mut self, val: Value) -> Result<()> {
        let value = val.convert()?;
        self.0 = Rc::new(UnsafeCell::new(value));
        Ok(())
    }
}

impl<T: 'static + Serialize> Prop<T> {
    fn dump(&self) -> Result<Value> {
        Value::from_t(self.deref())
    }
}

/// A property
#[derive(Default)]
pub struct InputProp<T>(Prop<T>);

impl<T: 'static> InputProp<T> {
    pub fn new(val: T) -> Self {
        Self(Prop::new(val))
    }

    pub fn change_value(&mut self, val: GenericOutputProp) -> Result<()> {
        self.0.change_value(val)
    }

    pub fn as_generic(&self) -> GenericInputProp {
        GenericInputProp(self.0.as_generic())
    }
}

impl<T: 'static + DeserializeOwned> InputProp<T> {
    pub fn load(&mut self, val: Value) -> Result<()> {
        self.0.load(val)
    }
}

impl<T: 'static + Serialize> InputProp<T> {
    pub fn dump(&self) -> Result<Value> {
        self.0.dump()
    }
}

impl<T: 'static> std::ops::Deref for InputProp<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

#[derive(Default)]
pub struct OutputProp<T>(Prop<T>);

impl<T: 'static> OutputProp<T> {
    pub fn new(val: T) -> Self {
        Self(Prop::new(val))
    }

    pub fn as_generic(&self) -> GenericOutputProp {
        GenericOutputProp(self.0.as_generic())
    }
}

impl<T: 'static + Serialize> OutputProp<T> {
    pub fn dump(&self) -> Result<Value> {
        self.0.dump()
    }
}

impl<T: 'static> std::ops::Deref for OutputProp<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<T: 'static> std::ops::DerefMut for OutputProp<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

struct GenericProp(Rc<dyn Any>);

impl GenericProp {
    pub fn try_read<T: 'static>(&self, f: impl Fn(&T)) -> Result<()> {
        if let Ok(v) = self.0.clone().downcast::<UnsafeCell<T>>() {
            f(unsafe { &*v.get() });
            Ok(())
        } else {
            Err("wrong type")
        }
    }

    pub fn is_same(&self, other: &GenericProp) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

/// Generic Property to be casted back
pub struct GenericOutputProp(GenericProp);

impl GenericOutputProp {
    pub fn try_read<T: 'static>(&self, f: impl Fn(&T)) -> Result<()> {
        self.0.try_read(f)
    }
}

pub struct GenericInputProp(GenericProp);

impl GenericInputProp {
    pub fn try_read<T: 'static>(&self, f: impl Fn(&T)) -> Result<()> {
        self.0.try_read(f)
    }

    pub fn is_same(&self, other: &GenericOutputProp) -> bool {
        self.0.is_same(&other.0)
    }
}
