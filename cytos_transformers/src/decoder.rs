use cytos::{props::Ownable, Prop, Result, Stepper};
use cytos_derive::CytosNode;
use serde::{Deserialize, Serialize};

use crate::types::Frame;

#[derive(Serialize, Deserialize)]
pub struct SerdeImage {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl TryFrom<SerdeImage> for Image {
    fn try_from(value: SerdeImage) -> std::result::Result<Self, Self::Error> {
        let image = image::GrayImage::from_vec(value.width, value.height, value.data)
            .ok_or("cannot convert")?;
        Ok(Image { image })
    }

    type Error = &'static str;
}

impl From<Image> for SerdeImage {
    fn from(value: Image) -> Self {
        SerdeImage {
            width: value.image.width(),
            height: value.image.height(),
            data: value.image.into_vec(),
        }
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(try_from = "SerdeImage", into = "SerdeImage")]
pub struct Image {
    pub image: image::GrayImage,
}

impl Ownable for Image {
    type Value = Image;

    fn to_ownable(&self) -> Self::Value {
        self.clone()
    }

    fn from_owned(v: &Self::Value) -> Self {
        v.clone()
    }
}

#[derive(CytosNode, Default)]
pub struct ImageDecoder {
    #[input]
    frame: Prop<Frame>,

    #[output]
    decoded: Prop<Image>,
}

impl Stepper for ImageDecoder {
    fn step(&mut self) -> Result<()> {
        if let Ok(image) = image::load_from_memory(self.frame.as_u8()) {
            *self.decoded = Image {
                image: image.to_luma8(),
            };
            Ok(())
        } else {
            Err("cannot decode image".into())
        }
    }
}
