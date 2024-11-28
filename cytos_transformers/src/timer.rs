use std::time::{Duration, Instant};

use cytos::{Prop, Stepper};
use cytos_derive::CytosNode;

#[derive(CytosNode)]
pub struct Timer {
    #[output]
    output: Prop<Duration>,

    timer: Instant,
    count: u64,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            output: Prop::new(Duration::ZERO),
            timer: Instant::now(),
            count: 0,
        }
    }
}

impl Stepper for Timer {
    fn step(&mut self) -> cytos::Result<()> {
        if self.count == 0 {
            *self.output = self.timer.elapsed();

            self.timer = Instant::now();
            self.count = 1000;
        }
        self.count -= 1;

        Ok(())
    }
}
