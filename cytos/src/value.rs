use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::Result;

/// A generic value wrapper around JSON values for flexible serialization and deserialization.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct Value(serde_json::Value);

impl Value {
    /// Load a serializable value into a `Value`.
    ///
    /// # Errors
    ///
    /// Returns an error if the value cannot be serialized to JSON.
    pub fn load<T: Serialize>(val: &T) -> Result<Self> {
        serde_json::to_value(val)
            .map(Self)
            .or(Err("cannot load value".into()))
    }

    /// Deserialize the `Value` into a specific type.
    ///
    /// # Errors
    ///
    /// Returns an error if the value cannot be deserialized into the target type.
    pub fn dump<T: DeserializeOwned>(self) -> Result<T> {
        serde_json::from_value(self.0).or(Err("cannot dump value".into()))
    }
}
