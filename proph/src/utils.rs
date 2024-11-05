//! Util fuctions
//!
//! Util functions to help in the developent

use std::time::Instant;

/// Executes a function and returns the execution time
pub fn execution_time<T: FnMut()>(mut f: T) -> f64 {
    let now = Instant::now();

    f();

    let elapsed_time = now.elapsed();
    elapsed_time.as_secs_f64()
}
