use proph::architecture::{InputProp, OutputProp, Result, Stepper};
use proph_derive::TransFn;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

#[derive(TransFn, Default)]
pub struct ImageDecoder {
    frame: InputProp<Vec<u8>>,

    decoded: OutputProp<Image>,
}

impl Stepper for ImageDecoder {
    fn step(&mut self) -> Result<()> {
        if let Ok(image) = image::load_from_memory(&self.frame[..]) {
            let gray = image.to_luma8();
            let width = gray.width();
            let height = gray.height();
            let data = gray.into_vec();
            *self.decoded = Image {
                width,
                height,
                data,
            };
            Ok(())
        } else {
            Err("cannot decode image".into())
        }
    }
}
