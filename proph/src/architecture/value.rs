use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::architecture::Result;

#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct Value(serde_json::Value);

impl Value {
    pub fn load<T: Serialize>(val: &T) -> Result<Self> {
        serde_json::to_value(val)
            .map(Self)
            .or(Err("cannot dump value"))
    }

    pub fn dump<T: DeserializeOwned>(self) -> Result<T> {
        serde_json::from_value(self.0).or(Err("cannot load value"))
    }
}
