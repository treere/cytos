use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::architecture::Result;

#[derive(Serialize, Deserialize, Debug)]
pub struct Value(serde_json::Value);

impl Value {
    pub fn from_string(s: String) -> Result<Self> {
        serde_json::from_str(&s).map(Self).or(Err("Cannot decode"))
    }

    pub fn from_t<T: Serialize>(val: &T) -> Result<Self> {
        serde_json::to_value(val)
            .map(Self)
            .or(Err("cannot dump value"))
    }

    pub fn to_string(&self) -> Result<String> {
        serde_json::to_string(&self.0).or(Err("cannot dump to string"))
    }

    pub fn convert<T: DeserializeOwned>(self) -> Result<T> {
        serde_json::from_value(self.0).or(Err("cannot load value"))
    }
}
