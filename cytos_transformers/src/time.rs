use cytos::loader::DynamicLoadingRegistryWrapper;
use cytos::{Prop, Stepper};
use cytos_derive::CytosNode;

use std::time::{Duration, Instant};
use std::{thread, time};

#[derive(CytosNode)]
struct Timer {
    #[output]
    output: Prop<Duration>,

    #[input]
    every: Prop<u64>,

    timer: Instant,
    count: u64,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            output: Prop::new(Duration::ZERO),
            timer: Instant::now(),
            count: 0,
            every: Prop::new(30),
        }
    }
}

impl Stepper for Timer {
    fn step(&mut self) -> cytos::Result<()> {
        if self.count == 0 {
            *self.output = self.timer.elapsed();

            self.timer = Instant::now();
            self.count = *self.every;
        }
        self.count -= 1;

        Ok(())
    }
}

#[derive(CytosNode, Default)]
struct Sleep {
    #[input]
    millis: Prop<u64>,
}

impl Stepper for Sleep {
    fn step(&mut self) -> cytos::Result<()> {
        let ten_millis = time::Duration::from_millis(*self.millis);
        thread::sleep(ten_millis);

        Ok(())
    }
}

pub fn load_registry(registry: &mut DynamicLoadingRegistryWrapper) {
    registry
        .add("Sleep", Sleep::default)
        .add("Timer", Timer::default);
}
