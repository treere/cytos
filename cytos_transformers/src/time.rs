use cytos::loader::DynamicLoadingRegistryWrapper;
use cytos::{Prop, Stepper};
use cytos_derive::CytosNode;

use std::time::{Duration, Instant};
use std::{thread, time};

#[derive(CytosNode)]
struct Timer {
    #[cytos(output)]
    output: Prop<Duration>,

    #[cytos(output)]
    fps: Prop<f64>,

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

#[derive(CytosNode, Default)]
struct Sleep {
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

#[derive(CytosNode)]
struct RateLimiter {
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
