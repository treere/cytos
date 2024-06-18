//! Properties
use serde::{de::DeserializeOwned, Serialize};
use std::{any::Any, cell::UnsafeCell, rc::Rc};

use super::{    Result, Value};

pub trait Dump {
    fn dump(&self) -> Result<Value>;
}

pub struct Dumper {
    prop: Box<dyn Dump>,
}

impl Dumper {
    pub fn dump(&self) -> Result<Value> {
        self.prop.dump()
    }
}

/// A property
pub struct InputProp<T> {
    val: Rc<UnsafeCell<T>>,
}

impl<T: 'static> InputProp<T> {
    pub fn new(val: T) -> Self {
        Self {
            val: Rc::new(UnsafeCell::new(val)),
        }
    }

    pub fn get(&self) -> &T {
        unsafe { &*self.val.get() }
    }

    pub fn change_value(&mut self, val: GenericOutputProp) -> Result<()> {
        if let Ok(v) = val.prop.downcast::<UnsafeCell<T>>() {
            self.val = v;
            Ok(())
        } else {
            Err("invalid type")
        }
    }

    pub fn as_generic(&self) -> GenericInputProp {
        GenericInputProp {
            prop: self.val.clone(),
        }
    }
}

impl<T: 'static + DeserializeOwned> InputProp<T> {
    pub fn load(&mut self, val: Value) -> Result<()> {
        let value = val.convert()?;
        self.val = Rc::new(UnsafeCell::new(value));
        Ok(())
    }
}

impl<T: 'static + Serialize> Dump for InputProp<T> {
    fn dump(&self) -> Result<Value> {
        Value::from_t(self.get())
    }
}

impl<T: 'static + Serialize> InputProp<T> {
    pub fn dump(&self) -> Result<Value> {
        Value::from_t(self.get())
    }

    pub fn as_dumper(&self) -> Dumper {
        Dumper {
            prop: Box::new(Self {
                val: self.val.clone(),
            }),
        }
    }
}

impl<T: Default + 'static> Default for InputProp<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

pub struct OutputProp<T> {
    val: Rc<UnsafeCell<T>>,
}

impl<T: 'static> OutputProp<T> {
    pub fn new(val: T) -> Self {
        Self {
            val: Rc::new(UnsafeCell::new(val)),
        }
    }

    pub fn get(&self) -> &T {
        unsafe { &*self.val.get() }
    }

    pub fn set(&mut self) -> &mut T {
        unsafe { &mut *self.val.get() }
    }

    pub fn as_generic(&self) -> GenericOutputProp {
        GenericOutputProp {
            prop: self.val.clone(),
        }
    }
}

impl<T: 'static + Serialize> Dump for OutputProp<T> {
    fn dump(&self) -> Result<Value> {
        Value::from_t(self.get())
    }
}

impl<T: 'static + Serialize> OutputProp<T> {
    pub fn dump(&self) -> Result<Value> {
        Value::from_t(self.get())
    }

    pub fn as_dumper(&self) -> Dumper {
        Dumper {
            prop: Box::new(Self {
                val: self.val.clone(),
            }),
        }
    }
}

impl<T: Default + 'static> Default for OutputProp<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

/// Generic Property to be casted back
pub struct GenericOutputProp {
    prop: Rc<dyn Any>,
}

impl GenericOutputProp {
    pub fn try_read<T: 'static>(&self, f: impl Fn(&T)) -> Result<()> {
        if let Ok(v) = self.prop.clone().downcast::<UnsafeCell<T>>() {
            f(unsafe { &*v.get() });
            Ok(())
        } else {
            Err("wrong type")
        }
    }
}

pub struct GenericInputProp {
    prop: Rc<dyn Any>,
}

impl GenericInputProp {
    pub fn try_read<T: 'static>(&self, f: impl Fn(&T)) -> Result<()> {
        if let Ok(v) = self.prop.clone().downcast::<UnsafeCell<T>>() {
            f(unsafe { &*v.get() });
            Ok(())
        } else {
            Err("wrong type")
        }
    }

    pub fn is_linked_to(&self, other: &GenericOutputProp) -> bool {
        Rc::ptr_eq(&self.prop, &other.prop)
    }
}
