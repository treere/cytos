//! Properties
use serde::{de::DeserializeOwned, Serialize};
use std::{any::Any, cell::UnsafeCell, rc::Rc};

use super::{Done, Result};

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

    pub fn change_value(&mut self, val: GenericOutputProp) -> Done {
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
    pub fn load(&mut self, val: &str) -> Done {
        self.val = Rc::new(UnsafeCell::new(serde_json::from_str(val).unwrap()));
        Ok(())
    }
}

impl<T: 'static + Serialize> InputProp<T> {
    pub fn dump(&self) -> Result<String> {
        serde_json::to_string(self.get()).or(Err("cannot dump value"))
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

impl<T: 'static + Serialize> OutputProp<T> {
    pub fn dump(&self) -> Result<String> {
        serde_json::to_string(self.get()).or(Err("cannot dump value"))
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
    pub fn try_read<T: 'static>(&self, f: impl Fn(&T)) -> Done {
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
    pub fn try_read<T: 'static>(&self, f: impl Fn(&T)) -> Done {
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
