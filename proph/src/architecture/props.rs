//! Properties
use serde::{de::DeserializeOwned, Serialize};
use std::{any::Any, cell::UnsafeCell, rc::Rc};

use super::{Result, Value};

pub trait Dump {
    fn dump(&self) -> Result<Value>;
}

pub struct Dumper(Box<dyn Dump>);

impl Dumper {
    pub fn dump(&self) -> Result<Value> {
        self.0.dump()
    }
}

struct Prop<T>(Rc<UnsafeCell<T>>);

impl<T: 'static> Prop<T> {
    pub fn new(val: T) -> Self {
        Self(Rc::new(UnsafeCell::new(val)))
    }

    pub fn get(&self) -> &T {
        unsafe { &*self.0.get() }
    }

    pub fn set(&mut self) -> &mut T {
        unsafe { &mut *self.0.get() }
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

impl<T: 'static + DeserializeOwned> Prop<T> {
    pub fn load(&mut self, val: Value) -> Result<()> {
        let value = val.convert()?;
        self.0 = Rc::new(UnsafeCell::new(value));
        Ok(())
    }
}

impl<T: 'static + Serialize> Dump for Prop<T> {
    fn dump(&self) -> Result<Value> {
        Value::from_t(self.get())
    }
}

impl<T: 'static + Serialize> Prop<T> {
    pub fn as_dumper(&self) -> Dumper {
        Dumper(Box::new(Self(self.0.clone())))
    }
}

impl<T: Default + 'static> Default for Prop<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

/// A property
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

impl<T: 'static + Serialize> Dump for InputProp<T> {
    fn dump(&self) -> Result<Value> {
        self.0.dump()
    }
}

impl<T: 'static + Serialize> InputProp<T> {
    pub fn as_dumper(&self) -> Dumper {
        self.0.as_dumper()
    }
}

impl<T: Default + 'static> Default for InputProp<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T:'static> std::ops::Deref for InputProp<T> {
    type Target=T;

    fn deref(&self) -> &Self::Target {
        self.0.get()
    }
}

pub struct OutputProp<T>(Prop<T>);

impl<T: 'static> OutputProp<T> {
    pub fn new(val: T) -> Self {
        Self(Prop::new(val))
    }

    pub fn as_generic(&self) -> GenericOutputProp {
        GenericOutputProp(self.0.as_generic())
    }
}

impl<T: 'static + Serialize> Dump for OutputProp<T> {
    fn dump(&self) -> Result<Value> {
        Value::from_t(self.0.get())
    }
}

impl<T: 'static + Serialize> OutputProp<T> {
    pub fn as_dumper(&self) -> Dumper {
        self.0.as_dumper()
    }
}

impl<T: Default + 'static> Default for OutputProp<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T:'static> std::ops::Deref for OutputProp<T> {
    type Target=T;

    fn deref(&self) -> &Self::Target {
        self.0.get()
    }
}

impl<T:'static> std::ops::DerefMut for OutputProp<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.set()
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
