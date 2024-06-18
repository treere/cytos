use serde::{de::DeserializeOwned, Serialize};

use crate::architecture::Result;

pub type Value = serde_json::Value;

pub fn load_value_from_string(s: String) -> Result<Value> {
    serde_json::from_str(&s).or(Err("Cannot decode"))
}

pub fn dump_to_value<T: Serialize>(val: &T) -> Result<Value> {
    serde_json::to_value(val).or(Err("cannot dump value"))
}

pub fn dump_to_string(data: &Value) -> Result<String> {
    serde_json::to_string(data).or(Err("cannot dump to string"))
}

pub fn load_value<T: DeserializeOwned>(val: Value) -> Result<T> {
    serde_json::from_value(val).or(Err("cannot load value"))
}
