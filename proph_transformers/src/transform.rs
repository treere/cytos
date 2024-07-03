use proph::architecture::{InputProp, OutputProp, Result, Stepper};
use proph_derive::TransFn;

use crate::decoder::Image;

#[derive(TransFn, Default)]
pub struct GrayScale {
    input: InputProp<Image>,
    output: OutputProp<Image>,
}

impl Stepper for GrayScale {
    fn step(&mut self) -> Result<()> {
        let p = image::imageops::grayscale(&self.input.data).into();
        *self.output = Image { data: p };
        Ok(())
    }
}
