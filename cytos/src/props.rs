//! Properties
use serde::{de::DeserializeOwned, Serialize};
use std::{any::Any, cell::UnsafeCell, rc::Rc};

use super::{Result, Value};

/// Convert to and from a Send+Sync type
pub trait Ownable {
    /// Send sync type
    type Value: Send + Sync + 'static;

    /// Convert to ownable
    fn to_ownable(&self) -> Self::Value;

    /// Convert from ownable
    fn from_owned(v: &Self::Value) -> Self;
}

impl<T: Ownable> Ownable for Option<T> {
    type Value = Option<T::Value>;

    fn to_ownable(&self) -> Self::Value {
        self.as_ref().map(Ownable::to_ownable)
    }

    fn from_owned(v: &Self::Value) -> Self {
        v.as_ref().map(Ownable::from_owned)
    }
}

macro_rules! create_ownable_copy {
    ($ty:ty) => {
        impl Ownable for $ty {
            type Value = $ty;

            fn to_ownable(&self) -> Self::Value {
                *self
            }

            fn from_owned(v: &Self::Value) -> Self {
                *v
            }
        }
    };
}

macro_rules! create_ownable_clone {
    ($ty:ty) => {
        impl Ownable for $ty {
            type Value = $ty;

            fn to_ownable(&self) -> Self::Value {
                self.clone()
            }

            fn from_owned(v: &Self::Value) -> Self {
                v.clone()
            }
        }
    };
}

macro_rules! create_ownable_clone_container {
    ($ty:ty) => {
        impl<T: Clone + 'static + Send + Sync> Ownable for $ty {
            type Value = $ty;

            fn to_ownable(&self) -> Self::Value {
                self.clone()
            }

            fn from_owned(v: &Self::Value) -> Self {
                v.clone()
            }
        }
    };
}

macro_rules! create_ownable_clone_container_doubled {
    ($ty:ty) => {
        impl<K: Clone + 'static + Send + Sync, V: Clone + 'static + Send + Sync> Ownable for $ty {
            type Value = $ty;

            fn to_ownable(&self) -> Self::Value {
                self.clone()
            }

            fn from_owned(v: &Self::Value) -> Self {
                v.clone()
            }
        }
    };
}

create_ownable_copy!(bool);
create_ownable_copy!(u8);
create_ownable_copy!(u16);
create_ownable_copy!(u32);
create_ownable_copy!(u64);
create_ownable_copy!(usize);
create_ownable_copy!(i8);
create_ownable_copy!(i16);
create_ownable_copy!(i32);
create_ownable_copy!(i64);
create_ownable_copy!(isize);
create_ownable_copy!(f32);
create_ownable_copy!(f64);
create_ownable_copy!(char);
create_ownable_copy!(std::time::Duration);

create_ownable_clone!(String);

create_ownable_clone_container!((T,));
create_ownable_clone_container!((T, T));
create_ownable_clone_container!((T, T, T));
create_ownable_clone_container!((T, T, T, T));
create_ownable_clone_container!((T, T, T, T, T));
create_ownable_clone_container!((T, T, T, T, T, T));
create_ownable_clone_container!((T, T, T, T, T, T, T));
create_ownable_clone_container!((T, T, T, T, T, T, T, T));
create_ownable_clone_container!((T, T, T, T, T, T, T, T, T));
create_ownable_clone_container!((T, T, T, T, T, T, T, T, T, T));
create_ownable_clone_container!((T, T, T, T, T, T, T, T, T, T, T));
create_ownable_clone_container!((T, T, T, T, T, T, T, T, T, T, T, T));
create_ownable_clone_container!((T, T, T, T, T, T, T, T, T, T, T, T, T));
create_ownable_clone_container!((T, T, T, T, T, T, T, T, T, T, T, T, T, T));
create_ownable_clone_container!(Vec<T>);
create_ownable_clone_container!(std::collections::VecDeque<T>);
create_ownable_clone_container!(std::collections::HashSet<T>);
create_ownable_clone_container!(std::collections::BTreeSet<T>);
create_ownable_clone_container!(std::collections::LinkedList<T>);
create_ownable_clone_container!(std::collections::BinaryHeap<T>);

create_ownable_clone_container_doubled!(std::collections::HashMap<K,V>);
create_ownable_clone_container_doubled!(std::collections::BTreeMap<K,V>);

/// Generic Property as owned
pub struct GenericOwnedProp(Box<dyn Any + Send + Sync + 'static>);

impl std::fmt::Debug for GenericOwnedProp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GenericOwnedProp")
    }
}

/// Internal prop structure
#[derive(Default)]
pub struct Prop<T>(Rc<UnsafeCell<T>>);

impl<T: 'static> Prop<T> {
    /// Create a prop
    pub fn new(val: T) -> Self {
        Self(Rc::new(UnsafeCell::new(val)))
    }

    /// Link that prop to another prop
    ///
    /// # Errors
    ///
    /// Will return `Err` if the type is invalid
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

    /// Load a `GenericOwnedProp` into `self`
    ///
    /// # Errors
    ///
    /// Will return `Err` if the type is invalid
    pub fn load_owned_generic(&mut self, val: GenericOwnedProp) -> Result<()> {
        if let Ok(v) = val.0.downcast::<T::Value>() {
            self.0 = Rc::new(UnsafeCell::new(Ownable::from_owned(&*v)));
            Ok(())
        } else {
            Err("invalid type".into())
        }
    }

    /// Assign a `GenericOwnedProp` into `self`
    ///
    /// # Errors
    ///
    /// Will return `Err` if cannot load self into a `Value`
    pub fn assign_owned_generic(&mut self, val: GenericOwnedProp) -> Result<()> {
        val.0.downcast::<T::Value>().map_or_else(
            |_| Err("invalid type".into()),
            |v| {
                *std::ops::DerefMut::deref_mut(self) = Ownable::from_owned(&*v);

                Ok(())
            },
        )
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
    ///
    /// # Errors
    ///
    /// Will return `Err` if cannot dump the `val`
    pub fn load(&mut self, val: Value) -> Result<()> {
        let value = val.dump()?;
        self.0 = Rc::new(UnsafeCell::new(value));
        Ok(())
    }

    /// Assign a value into a prop
    ///
    /// # Errors
    ///
    /// Will return `Err` if cannot dump the `val`
    pub fn assign(&mut self, val: Value) -> Result<()> {
        let value = val.dump::<T>()?;
        *std::ops::DerefMut::deref_mut(self) = value;

        Ok(())
    }
}

impl<T: 'static + Serialize> Prop<T> {
    /// Dump the value of a prop
    ///
    /// # Errors
    ///
    /// Will return `Err` if cannot load self into a `Value`
    pub fn dump(&self) -> Result<Value> {
        Value::load(&**self)
    }
}

/// Generic prop
pub struct GenericProp(Rc<dyn Any>);

impl GenericProp {
    /// Verify if two generic props are the same
    pub fn is_same(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_props() {
        let output_prop = Prop::new(12);
        let gen = output_prop.as_generic();

        let mut input_prop = Prop::new(1);
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
