use proph::architecture::{Done, InputProp, OutputProp, Stepper};
use proph_derive::TransFn;

use crate::decoder::Image;

#[derive(TransFn, Default)]
pub struct Mean {
    input: InputProp<Image>,
    output: OutputProp<f64>,
}

impl Stepper for Mean {
    fn initialize(&mut self) -> Done {
        Ok(())
    }

    fn step(&mut self) -> Done {
        let img = &self.input.get().data;
        let sum = img.as_bytes().iter().fold(0u64, |a, b| (*b) as u64 + a) as f64;

        *self.output.set() = sum / (img.width() * img.height()) as f64;
        Ok(())
    }
}
