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
