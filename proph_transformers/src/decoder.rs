use proph::architecture::{props::Ownable, InputProp, OutputProp, Result, Stepper};
use proph_derive::TransFn;
use serde::{Deserialize, Serialize};

use crate::types::Frame;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl Ownable for Image {
    type Value = u8;

    fn to_ownable(&self) -> Self::Value {
        todo!()
    }

    fn from_owned(_v: &Self::Value) -> Self {
        todo!()
    }
}

#[derive(TransFn, Default)]
pub struct ImageDecoder {
    frame: InputProp<Frame>,

    decoded: OutputProp<Image>,
}

impl Stepper for ImageDecoder {
    fn step(&mut self) -> Result<()> {
        if let Ok(image) = image::load_from_memory(self.frame.as_u8()) {
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
