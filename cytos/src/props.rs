//! Properties module for managing node parameters with interior mutability.
//!
//! This module provides the [`Prop<T>`] type, which is the primary way nodes store and
//! exchange data. Props use interior mutability via [`Rc<UnsafeCell<T>>`] to allow
//! shared mutable access without requiring mutable references.
//!
//! # Load vs Assign
//!
//! The [`GenericPropInterface`] trait provides two distinct methods for setting values:
//!
//! - **[`load`](GenericPropInterface::load)**: Replaces the entire property value, **breaking all links**.
//!   Use this when you want to set a static configuration value that shouldn't change
//!   dynamically. This removes any connections to other props.
//!
//! - **[`assign`](GenericPropInterface::assign)**: Updates the property value while **preserving links**.
//!   Use this when the prop is actively linked to other nodes and you want to update
//!   its value as part of normal dataflow. The prop will continue to receive updates
//!   from linked sources.
//!
//! # The Ownable Trait
//!
//! The [`Ownable`] trait enables types to be converted to a `Send + Sync` representation.
//! This is essential for transferring ownership of props between threads when using
//! [`GenericOwnedProp`] for inter-graph communication. Types that implement `Ownable`
//! can be safely sent across thread boundaries while preserving their data.
//!
//! # Example Usage
//!
//! ```rust,ignore
//! use cytos::Prop;
//!
//! // Create a property holding an i32
//! let mut prop = Prop::new(42);
//!
//! // Access the value (uses Deref)
//! assert_eq!(*prop, 42);
//!
//! // Modify the value (uses DerefMut)
//! *prop = 100;
//!
//! // Link to another prop
//! let output_prop = Prop::new(200);
//! let generic = output_prop.as_generic();
//! prop.link_value(generic).expect("type mismatch");
//!
//! // Now prop shares the same underlying data as output_prop
//! assert_eq!(*prop, 200);
//! ```

use serde::{Serialize, de::DeserializeOwned};
use std::{any::Any, cell::UnsafeCell, rc::Rc};

use super::{Result, Value};

/// Converts a type to and from a `Send + Sync` representation.
///
/// This trait is essential for types used in [`GenericOwnedProp`], which enables
/// transferring property ownership between threads. The `Value` associated type
/// must implement `Send + Sync + 'static` to ensure thread safety.
///
/// Implementations are provided for primitive types, strings, common collections,
/// tuples, and the `imageio::Image` type.
///
/// # Example
///
/// For a simple wrapper type:
/// ```rust,ignore
/// struct MyData(String);
///
/// impl Ownable for MyData {
///     type Value = String;
///
///     fn to_ownable(&self) -> Self::Value {
///         self.0.clone()
///     }
///
///     fn from_owned(v: &Self::Value) -> Self {
///         MyData(v.clone())
///     }
/// }
/// ```
pub trait Ownable {
    /// The `Send + Sync` type that this type can be converted to/from.
    type Value: Send + Sync + 'static;

    /// Converts this type to its `Send + Sync` representation.
    fn to_ownable(&self) -> Self::Value;

    /// Converts from the `Send + Sync` representation back to this type.
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

/// A generic owned property for transferring values between threads.
///
/// This type wraps any `Send + Sync + 'static` type and enables safe transfer
/// of property values across thread boundaries. It is used when sending data
/// between graphs running in different threads via [`SystemLink`] connections.
///
/// To create a `GenericOwnedProp`, use [`GenericPropInterface::as_owned`] on a prop.
/// To restore the value, use [`GenericPropInterface::load_owned`] or
/// [`GenericPropInterface::assign_owned`].
///
/// [`SystemLink`]: crate::repr::SystemLink
pub struct GenericOwnedProp(Box<dyn Any + Send + Sync + 'static>);

impl std::fmt::Debug for GenericOwnedProp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GenericOwnedProp")
    }
}

/// Interface for managing property values and links in the graph system.
///
/// This trait provides a common interface for properties that can be linked
/// to other properties, loaded with configuration values, assigned runtime values,
/// and serialized for inter-thread communication.
///
/// # Load vs Assign
///
/// The key distinction between [`load`](Self::load) and [`assign`](Self::assign):
///
/// - **Load**: Replaces the entire property value and **breaks all existing links**.
///   Use this for static configuration that shouldn't be dynamically updated.
///   After loading, the prop will only hold the loaded value, not receive updates
///   from other nodes.
///
/// - **Assign**: Updates the property value while **preserving links**.
///   Use this during normal graph execution when the prop is linked to upstream
///   nodes. The prop will continue to receive updates from linked sources after
///   the assignment.
pub trait GenericPropInterface {
    /// Links this property to another generic property.
    ///
    /// After linking, this property will share the same underlying data as the
    /// source property. Any changes to either property will be visible to both.
    ///
    /// # Arguments
    ///
    /// * `val` - The generic property to link to.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the type is invalid or the link cannot be established.
    fn link(&mut self, val: GenericProp) -> Result<()>;

    /// Loads a configuration value, replacing the property and breaking links.
    ///
    /// This method creates a new property with the given value, effectively
    /// severing any existing links. Use this when setting static configuration
    /// values that should not be dynamically updated during graph execution.
    ///
    /// # Arguments
    ///
    /// * `val` - The value to load (serialized as JSON).
    ///
    /// # Errors
    ///
    /// Returns an error if the value cannot be deserialized or loaded.
    fn load(&mut self, val: Value) -> Result<()>;

    /// Assigns a runtime value while preserving existing links.
    ///
    /// This method updates the property's current value but maintains any
    /// existing links to other properties. Use this during graph execution
    /// when the property is part of the dataflow and should continue receiving
    /// updates from linked sources.
    ///
    /// # Arguments
    ///
    /// * `val` - The value to assign (serialized as JSON).
    ///
    /// # Errors
    ///
    /// Returns an error if the value cannot be deserialized or assigned.
    fn assign(&mut self, val: Value) -> Result<()>;
    ///
    /// Dumps the current value of a parameter.
    ///
    /// # Errors
    ///
    /// Returns an error if the value cannot be dumped.
    fn dump(&self) -> Result<Value>;
    ///
    /// Loads an owned property for a parameter.
    ///
    /// # Arguments
    ///
    /// * `val` - The owned property to load.
    ///
    /// # Errors
    ///
    /// Returns an error if the property cannot be loaded.
    fn load_owned(&mut self, val: GenericOwnedProp) -> Result<()>;
    ///
    /// Assigns an owned property to a parameter.
    ///
    /// # Arguments
    ///
    /// * `val` - The owned property to assign.
    ///
    /// # Errors
    ///
    /// Returns an error if the property cannot be assigned.
    fn assign_owned(&mut self, val: GenericOwnedProp) -> Result<()>;

    /// Dumps the current owned property of a parameter.
    ///
    /// # Errors
    ///
    /// Returns an error if the property cannot be dumped.
    fn as_owned(&self) -> GenericOwnedProp;
    ///
    /// Gets an output property by parameter name.
    ///
    /// # Arguments
    ///
    /// * `val` - The parameter ID of the output.
    ///
    /// # Returns
    ///
    /// The output property if it exists, `None` otherwise.
    fn as_generic(&self) -> GenericProp;
}

/// A property that holds a value of type `T` with interior mutability.
///
/// `Prop<T>` is the fundamental unit of data storage and exchange in Cytos.
/// It uses [`Rc<UnsafeCell<T>>`] to enable shared mutable access without
/// requiring mutable references, allowing multiple nodes to share and modify
/// the same data.
///
/// Props implement [`std::ops::Deref`] and [`std::ops::DerefMut`], allowing direct access to the
/// underlying value using the `*` operator.
///
/// # Thread Safety
///
/// Props themselves are not `Send` or `Sync` because they use `Rc` and `UnsafeCell`.
/// For inter-thread communication, use [`GenericOwnedProp`] which converts the
/// value to a thread-safe representation via the [`Ownable`] trait.
///
/// # Linking
///
/// Props can be linked together using [`link`](GenericPropInterface::link) or
/// [`link_value`](Self::link_value). When linked, both props share the same
/// underlying data, so changes to one are immediately visible in the other.
///
/// # Example
///
/// ```rust,ignore
/// use cytos::Prop;
///
/// // Create a new property
/// let prop = Prop::new(42);
///
/// // Access the value
/// assert_eq!(*prop, 42);
///
/// // Create another property and link them
/// let mut prop2 = Prop::new(0);
/// prop2.link(prop.as_generic()).expect("type mismatch");
///
/// // Now they share the same data
/// *prop2 = 100;
/// assert_eq!(*prop, 100); // prop also sees the change
/// ```
#[derive(Default)]
pub struct Prop<T>(Rc<UnsafeCell<T>>);

impl<T: 'static> Prop<T> {
    /// Creates a new property with the given value.
    ///
    /// # Arguments
    ///
    /// * `val` - The initial value for the property.
    pub fn new(val: T) -> Self {
        Self(Rc::new(UnsafeCell::new(val)))
    }

    /// Links this property to another generic property.
    ///
    /// After linking, both properties share the same underlying data.
    /// Any changes to either property will be visible to both.
    ///
    /// # Type Safety
    ///
    /// This method performs a runtime type check to ensure the generic
    /// property contains a value of type `T`. If the types don't match,
    /// an error is returned.
    ///
    /// # Arguments
    ///
    /// * `val` - The generic property to link to.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the generic property's inner type doesn't match `T`.
    pub fn link_value(&mut self, val: GenericProp) -> Result<()> {
        match val.0.downcast::<UnsafeCell<T>>() {
            Ok(v) => {
                self.0 = v;
                Ok(())
            }
            _ => Err("invalid type".into()),
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

impl<T: DeserializeOwned + Serialize + Ownable + 'static> GenericPropInterface for Prop<T> {
    fn link(&mut self, val: GenericProp) -> Result<()> {
        self.link_value(val)
    }

    fn load(&mut self, val: Value) -> Result<()> {
        let value = val.dump()?;
        self.0 = Rc::new(UnsafeCell::new(value));
        Ok(())
    }

    fn assign(&mut self, val: Value) -> Result<()> {
        let value = val.dump::<T>()?;
        **self = value;

        Ok(())
    }

    fn dump(&self) -> Result<Value> {
        Value::load(&**self)
    }

    fn load_owned(&mut self, val: GenericOwnedProp) -> Result<()> {
        match val.0.downcast::<T::Value>() {
            Ok(v) => {
                self.0 = Rc::new(UnsafeCell::new(Ownable::from_owned(&*v)));
                Ok(())
            }
            _ => Err("invalid type".into()),
        }
    }

    fn assign_owned(&mut self, val: GenericOwnedProp) -> Result<()> {
        val.0.downcast::<T::Value>().map_or_else(
            |_| Err("invalid type".into()),
            |v| {
                **self = Ownable::from_owned(&*v);

                Ok(())
            },
        )
    }

    fn as_owned(&self) -> GenericOwnedProp {
        GenericOwnedProp(Box::new(self.to_ownable()))
    }

    fn as_generic(&self) -> GenericProp {
        GenericProp(self.0.clone())
    }
}

/// A type-erased property that can represent any `Prop<T>`.
///
/// `GenericProp` allows storing and passing properties without knowing their
/// concrete type at compile time. It wraps an `Rc<dyn Any>` and is used for
/// linking properties of different types together in the graph.
///
/// # Usage
///
/// Obtain a `GenericProp` from a typed prop using [`as_generic`](GenericPropInterface::as_generic),
/// then link it to another prop using [`link`](GenericPropInterface::link).
///
/// # Type Safety
///
/// While `GenericProp` erases the type, type checking is performed when linking
/// via [`link_value`](Prop::link_value), which will fail if the types don't match.
pub struct GenericProp(Rc<dyn Any>);

impl GenericProp {
    /// Checks if two generic props point to the same underlying data.
    ///
    /// This performs a pointer equality check on the internal `Rc`, returning `true`
    /// if both props share the exact same allocation (i.e., they are linked).
    ///
    /// # Arguments
    ///
    /// * `other` - The other generic prop to compare with.
    ///
    /// # Returns
    ///
    /// `true` if both props share the same underlying data, `false` otherwise.
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
        let generic = output_prop.as_generic();

        let mut input_prop = Prop::new(1);
        input_prop.link_value(generic).expect("cannot link");

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

        let generic = prop.as_owned();

        std::thread::spawn(|| {
            let mut thread_prop = Prop::new(2);
            thread_prop.load_owned(generic).expect("cannot link");

            assert_eq!(1, *thread_prop);
        })
        .join()
        .expect("cannot join");
    }
}
