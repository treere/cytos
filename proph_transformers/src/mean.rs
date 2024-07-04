use proph::architecture::{InputProp, OutputProp, Result, Stepper};
use proph_derive::TransFn;

use crate::decoder::Image;

#[derive(TransFn, Default)]
pub struct Mean {
    input: InputProp<Image>,
    output: OutputProp<f64>,
}

impl Stepper for Mean {
    fn step(&mut self) -> Result<()> {
        let img = &self.input.data;
        let sum = img.as_bytes().iter().fold(0u64, |a, b| u64::from(*b) + a) as f64;

        *self.output = sum / f64::from(img.width() * img.height());
        Ok(())
    }
}
