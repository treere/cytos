use std::io::Cursor;

use cytos::{loader::DynamicLoadingRegistryWrapper, props::Ownable, Prop, Result, Stepper};
use cytos_derive::CytosNode;
use image::ImageFormat;
use serde::{Deserialize, Serialize};

use crate::types::Frame;

#[derive(Serialize, Deserialize)]
pub struct SerdeImage {
    buffer: Vec<u8>,
}

impl TryFrom<SerdeImage> for Image {
    fn try_from(value: SerdeImage) -> std::result::Result<Self, Self::Error> {
        let image = image::load_from_memory(&value.buffer).map_err(|_| "cannot load")?;
        Ok(Self { image })
    }

    type Error = &'static str;
}

impl From<Image> for SerdeImage {
    fn from(value: Image) -> Self {
        let mut buffer = Vec::new();
        value
            .image
            .write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)
            .expect("cannot convert image to serde image");
        Self { buffer }
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(try_from = "SerdeImage", into = "SerdeImage")]
pub struct Image {
    pub image: image::DynamicImage,
}

impl Ownable for Image {
    type Value = Self;

    fn to_ownable(&self) -> Self::Value {
        self.clone()
    }

    fn from_owned(v: &Self::Value) -> Self {
        v.clone()
    }
}

#[derive(CytosNode, Default)]
struct ImageDecoder {
    #[input]
    frame: Prop<Frame>,

    #[output]
    decoded: Prop<Image>,
}

impl Stepper for ImageDecoder {
    fn step(&mut self) -> Result<()> {
        match image::load_from_memory(self.frame.as_u8()) { Ok(image) => {
            *self.decoded = Image { image };
            Ok(())
        } _ => {
            Err("cannot decode image".into())
        }}
    }
}

#[derive(CytosNode, Default)]
struct ImageSave {
    #[input]
    input: Prop<Image>,

    #[input]
    filename: Prop<String>,
}

impl Stepper for ImageSave {
    fn step(&mut self) -> Result<()> {
        self.input
            .image
            .save(&*self.filename)
            .map_err(|x| x.to_string())?;
        Ok(())
    }
}

pub fn load_registry(registry: &mut DynamicLoadingRegistryWrapper) {
    registry
        .add("ImageDecoder", ImageDecoder::default)
        .add("ImageSave", ImageSave::default);
}
