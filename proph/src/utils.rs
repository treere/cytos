//! Util fuctions

use std::time::Instant;

use crate::architecture::{GraphId, NodeId, ParamId, Result, Value};

/// Returns the seconds needed to execute the given function
pub fn execution_time<T: FnMut()>(mut f: T) -> f64 {
    let now = Instant::now();

    f();

    let elapsed_time = now.elapsed();
    elapsed_time.as_secs_f64()
}

pub fn string_to_u64(s: impl AsRef<str>) -> Result<u64> {
    u64::from_str_radix(s.as_ref(), 36).or(Err("invalid string"))
}

pub fn u64_to_string(val: u64) -> String {
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

pub fn convert_val_to_graphid_string(val: Value) -> Result<String> {
    let graph: GraphId = val.convert()?;
    Ok(format!("{graph}"))
}

pub fn convert_val_to_nodeid_string(val: Value) -> Result<Vec<String>> {
    let nodes: Vec<NodeId> = val.convert()?;
    Ok(nodes.into_iter().map(|n| format!("{n}")).collect())
}

pub fn convert_val_to_paramid_string(val: Value) -> Result<Vec<String>> {
    let nodes: Vec<ParamId> = val.convert()?;
    Ok(nodes.into_iter().map(|n| format!("{n}")).collect())
}
