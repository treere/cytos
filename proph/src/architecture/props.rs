//! Properties
use std::{any::Any, cell::UnsafeCell, rc::Rc};

pub fn are_linked(input: &GenericInputProp, output: &GenericOutputProp) -> bool {
    Rc::ptr_eq(&input.prop, &output.prop)
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

    pub fn change_value(&mut self, val: GenericOutputProp) -> Result<(), &'static str> {
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
    pub fn try_read<T: 'static>(&self, f: impl Fn(&T)) -> Result<(), &'static str> {
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
    pub fn try_read<T: 'static>(&self, f: impl Fn(&T)) -> Result<(), &'static str> {
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
