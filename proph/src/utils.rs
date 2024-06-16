//! Util fuctions

use std::time::Instant;

use serde::{de::DeserializeOwned, Serialize};

use crate::architecture::{NodeId, ParamId, Result, Value};

/// Returns the seconds needed to execute the given function
pub fn execution_time<T: FnMut()>(mut f: T) -> f64 {
    let now = Instant::now();

    f();

    let elapsed_time = now.elapsed();
    elapsed_time.as_secs_f64()
}

pub fn string_to_nodeid(s: impl AsRef<str>) -> Result<u64> {
    u64::from_str_radix(s.as_ref(), 36).or(Err("invalid string"))
}

pub fn string_to_paramid(s: impl AsRef<str>) -> Result<u64> {
    u64::from_str_radix(s.as_ref(), 36).or(Err("invalid string"))
}

pub fn nodeid_to_string(val: u64) -> String {
    format_radix(val, 36)
}

pub fn paramid_to_string(val: u64) -> String {
    format_radix(val, 36)
}

fn format_radix(mut x: u64, radix: u64) -> String {
    let mut result = vec![];

    loop {
        let m = x % radix;
        x /= radix;

        // will panic if you use a bad radix (< 2 or > 36).
        result.push(std::char::from_digit(m as u32, radix as u32).unwrap());
        if x == 0 {
            break;
        }
    }
    result.into_iter().rev().collect()
}

pub fn load_value_from_string(s: String) -> Result<Value> {
    serde_json::to_value(s).or(Err("Cannot decode"))
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

pub fn convert_val_to_nodeid_string(val: Value) -> Result<Vec<String>> {
    let nodes: Vec<NodeId> = serde_json::from_value(val).or(Err("cannot convert to NodeId"))?;
    Ok(nodes.into_iter().map(nodeid_to_string).collect())
}

pub fn convert_val_to_paramid_string(val: Value) -> Result<Vec<String>> {
    let nodes: Vec<ParamId> = serde_json::from_value(val).or(Err("cannot convert to ParamId"))?;
    Ok(nodes.into_iter().map(paramid_to_string).collect())
}
