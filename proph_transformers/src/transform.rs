use image::DynamicImage;
use proph::architecture::{OutputProp, Stepper};
use proph_derive::TransFn;

#[derive(TransFn, Default)]
pub struct GrayScale {
    input: OutputProp<DynamicImage>,
    output: OutputProp<DynamicImage>,
}

impl Stepper for GrayScale {
    fn initialize(&mut self) -> Result<(), &'static str> {
        Ok(())
    }

    fn step(&mut self) -> Result<(), &'static str> {
        *self.output.set() = image::imageops::grayscale(self.input.get()).into();
        Ok(())
    }
}
