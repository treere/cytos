use std::time::Instant;

pub fn time_execution<T: FnMut() -> ()>(mut f: T) -> f64 {
    let now = Instant::now();

    f();

    let elapsed_time = now.elapsed();
    elapsed_time.as_secs_f64()
}
