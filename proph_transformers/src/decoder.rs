use image::DynamicImage;
use proph::architecture::{InputProp, OutputProp, Stepper};
use proph_derive::TransFn;
use rscam::Frame;

#[derive(TransFn, Default)]
pub struct ImageDecoder {
    frame: InputProp<Option<Frame>>,

    decoded: OutputProp<DynamicImage>,
}

impl Stepper for ImageDecoder {
    fn initialize(&mut self) -> Result<(), &'static str> {
        Ok(())
    }

    fn step(&mut self) -> Result<(), &'static str> {
        if let Some(image) = self.frame.get() {
            if let Ok(image) = image::load_from_memory(image) {
                *self.decoded.set() = image;
                Ok(())
            } else {
                Err("cannot decode image")
            }
        } else {
            Err("missing image")
        }
    }
}
