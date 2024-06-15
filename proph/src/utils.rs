//! Util fuctions

use std::time::Instant;

/// Returns the seconds needed to execute the given function
pub fn execution_time<T: FnMut()>(mut f: T) -> f64 {
    let now = Instant::now();

    f();

    let elapsed_time = now.elapsed();
    elapsed_time.as_secs_f64()
}

pub fn string_to_nodeid(s: impl AsRef<str>) -> Result<u64, &'static str> {
    u64::from_str_radix(s.as_ref(), 36).map_err(|_| "invalid string")
}

pub fn string_to_paramid(s: impl AsRef<str>) -> Result<u64, &'static str> {
    u64::from_str_radix(s.as_ref(), 36).map_err(|_| "invalid string")
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
        x = x / radix;

        // will panic if you use a bad radix (< 2 or > 36).
        result.push(std::char::from_digit(m as u32, radix as u32).unwrap());
        if x == 0 {
            break;
        }
    }
    result.into_iter().rev().collect()
}
