use proph::architecture::{Done, InputProp, OutputProp, Stepper};
use proph_derive::TransFn;

use crate::decoder::Image;

#[derive(TransFn, Default)]
pub struct GrayScale {
    input: InputProp<Image>,
    output: OutputProp<Image>,
}

impl Stepper for GrayScale {
    fn step(&mut self) -> Done {
        let p = image::imageops::grayscale(&self.input.get().data).into();
        *self.output.set() = Image { data: p };
        Ok(())
    }
}
