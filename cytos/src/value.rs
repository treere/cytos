use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::Result;

/// Generic value
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct Value(serde_json::Value);

impl Value {
    /// Load into a value
    ///
    /// # Errors
    /// If cannot load
    pub fn load<T: Serialize>(val: &T) -> Result<Self> {
        serde_json::to_value(val)
            .map(Self)
            .or(Err("cannot load value".into()))
    }

    /// Dump a value
    ///
    /// # Errors
    /// If cannot dump
    pub fn dump<T: DeserializeOwned>(self) -> Result<T> {
        serde_json::from_value(self.0).or(Err("cannot dump value".into()))
    }
}
