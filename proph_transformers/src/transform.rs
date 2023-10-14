use image::DynamicImage;
use proph::architecture::{Done, OutputProp, Stepper};
use proph_derive::TransFn;

#[derive(TransFn, Default)]
pub struct GrayScale {
    input: OutputProp<DynamicImage>,
    output: OutputProp<DynamicImage>,
}

impl Stepper for GrayScale {
    fn initialize(&mut self) -> Done {
        Ok(())
    }

    fn step(&mut self) -> Done {
        *self.output.set() = image::imageops::grayscale(self.input.get()).into();
        Ok(())
    }
}
