use image::DynamicImage;
use proph::architecture::{Done, InputProp, OutputProp, Stepper};
use proph_derive::TransFn;

#[derive(TransFn, Default)]
pub struct ImageDecoder {
    frame: InputProp<Vec<u8>>,

    decoded: OutputProp<DynamicImage>,
}

impl Stepper for ImageDecoder {
    fn initialize(&mut self) -> Done {
        Ok(())
    }

    fn step(&mut self) -> Done {
        if let Ok(image) = image::load_from_memory(&self.frame.get()[..]) {
            *self.decoded.set() = image;
            Ok(())
        } else {
            Err("cannot decode image")
        }
    }
}
