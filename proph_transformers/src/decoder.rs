use image::DynamicImage;
use proph::architecture::{InputProp, OutputProp, Stepper};
use proph_derive::TransFn;

#[derive(TransFn, Default)]
pub struct ImageDecoder {
    frame: InputProp<(Vec<u8>, (u32, u32), [u8; 4])>,

    decoded: OutputProp<DynamicImage>,
}

impl Stepper for ImageDecoder {
    fn initialize(&mut self) -> Result<(), &'static str> {
        Ok(())
    }

    fn step(&mut self) -> Result<(), &'static str> {
        if let Ok(image) = image::load_from_memory(&self.frame.get().0) {
            *self.decoded.set() = image;
            Ok(())
        } else {
            Err("cannot decode image")
        }
    }
}
