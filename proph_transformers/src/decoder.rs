use image::DynamicImage;
use proph::architecture::{InputProp, OutputProp, Result, Stepper};
use proph_derive::TransFn;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]

pub struct Image {
    #[serde(skip)]
    pub data: DynamicImage,
}

#[derive(TransFn, Default)]
pub struct ImageDecoder {
    frame: InputProp<Vec<u8>>,

    decoded: OutputProp<Image>,
}

impl Stepper for ImageDecoder {
    fn step(&mut self) -> Result<()> {
        if let Ok(image) = image::load_from_memory(&self.frame[..]) {
            *self.decoded = Image { data: image };
            Ok(())
        } else {
            Err("cannot decode image")
        }
    }
}
