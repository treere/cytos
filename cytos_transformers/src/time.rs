//! Time management transformer nodes for Cytos.
//!
//! This module provides nodes for time-related operations:
//! - Timer: Measures elapsed time and calculates FPS (frames per second)
//! - Sleep: Pauses execution for a specified duration
//! - `RateLimiter`: Controls execution frequency by limiting steps per second
//!
//! These nodes are useful for profiling, pacing pipelines, and controlling
//! processing rates in real-time applications.

use cytos::loader::DynamicLoadingRegistryWrapper;
use cytos::{Prop, Stepper};
use cytos_derive::CytosNode;

use std::time::{Duration, Instant};
use std::{thread, time};

/// Node that measures elapsed time and calculates frames per second (FPS).
///
/// This node tracks time intervals and computes the processing rate.
/// It outputs the elapsed duration and calculated FPS every N steps,
/// where N is configurable via the `every` input.
#[derive(CytosNode)]
struct Timer {
    /// The elapsed time since last reset
    #[cytos(output)]
    output: Prop<Duration>,

    /// The calculated frames per second
    #[cytos(output)]
    fps: Prop<f64>,

    /// How often (in steps) to update the output values
    #[cytos(input)]
    every: Prop<u64>,

    instant: Instant,
    count: u64,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            output: Prop::new(Duration::ZERO),
            instant: Instant::now(),
            fps: Prop::default(),
            count: 0,
            every: Prop::new(30),
        }
    }
}

impl Stepper for Timer {
    fn step(&mut self) -> cytos::Result<()> {
        if self.count == 0 {
            *self.output = self.instant.elapsed();

            *self.fps = (*self.every as f64) / (*self.output).as_secs_f64();

            self.instant = Instant::now();
            self.count = *self.every;
        }
        self.count -= 1;

        Ok(())
    }
}

/// Node that pauses execution for a specified duration.
///
/// On each step, sleeps for the number of milliseconds specified by the
/// `millis` input. Useful for pacing pipelines or adding delays.
#[derive(CytosNode, Default)]
struct Sleep {
    /// The number of milliseconds to sleep on each step
    #[cytos(input)]
    millis: Prop<u64>,
}

impl Stepper for Sleep {
    fn step(&mut self) -> cytos::Result<()> {
        let ten_millis = time::Duration::from_millis(*self.millis);
        thread::sleep(ten_millis);

        Ok(())
    }
}

/// Node that limits execution frequency to a target rate.
///
/// Ensures that steps occur no more frequently than the specified frequency
/// in hertz (steps per second). If a step would occur too soon after the
/// previous one, the node sleeps until the target time has elapsed.
///
/// This is useful for controlling processing rates in real-time applications
/// or when interfacing with hardware that has specific timing requirements.
#[derive(CytosNode)]
struct RateLimiter {
    /// The target frequency in hertz (steps per second)
    #[cytos(input)]
    hz: Prop<f64>,

    last_step: Option<Instant>,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self {
            hz: Prop::new(60.0),
            last_step: None,
        }
    }
}

impl Stepper for RateLimiter {
    fn step(&mut self) -> cytos::Result<()> {
        let now = Instant::now();
        if let Some(last) = self.last_step {
            let elapsed = now.duration_since(last);
            let target_period = Duration::from_secs_f64(1.0 / *self.hz);
            if elapsed < target_period {
                let sleep_duration = target_period.checked_sub(elapsed).unwrap();
                thread::sleep(sleep_duration);
            }
        }
        self.last_step = Some(Instant::now());
        Ok(())
    }
}

pub fn load_registry(registry: &mut DynamicLoadingRegistryWrapper) {
    registry
        .add("Sleep", Sleep::default)
        .add("Timer", Timer::default)
        .add("RateLimiter", RateLimiter::default);
}
