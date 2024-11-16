use proph::architecture::{InputProp, OutputProp, Result, Stepper};
use proph_derive::ProphNode;

use crate::decoder::Image;

#[derive(ProphNode, Default)]
pub struct Mean {
    input: InputProp<Image>,
    output: OutputProp<f64>,
}

impl Stepper for Mean {
    fn step(&mut self) -> Result<()> {
        let sum = self
            .input
            .image
            .pixels()
            .fold(0u64, |a, b| u64::from(b[0]) + a) as f64;

        *self.output = sum / f64::from(self.input.image.width() * self.input.image.height());
        Ok(())
    }
}
