use std::time::{Duration, Instant};

use proph::architecture::{OutputProp, Stepper};
use proph_derive::ProphNode;

#[derive(ProphNode)]
pub struct Timer {
    output: OutputProp<Duration>,

    timer: Instant,
    count: u64,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            output: OutputProp::new(Duration::ZERO),
            timer: Instant::now(),
            count: 0,
        }
    }
}

impl Stepper for Timer {
    fn step(&mut self) -> proph::architecture::Result<()> {
        if self.count == 0 {
            *self.output = self.timer.elapsed();

            self.timer = Instant::now();
            self.count = 1000;
        }
        self.count -= 1;

        Ok(())
    }
}
