/// Image input/output transformer nodes for Cytos.
///
/// This module provides nodes for loading images from frames and saving images to files.
/// It includes serialization support for images via PNG encoding.
use std::io::Cursor;

use cytos::{Prop, Result, Stepper, loader::DynamicLoadingRegistryWrapper, props::Ownable};
use cytos_derive::CytosNode;
use image::ImageFormat;
use serde::{Deserialize, Serialize};

use crate::types::Frame;

/// Serializable representation of an image using PNG encoding.
/// Used for serializing images in Cytos dataflows.
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

/// Wrapper around `image::DynamicImage` that implements `Ownable` and serialization.
/// Supports serialization via PNG encoding for use in Cytos dataflows.
#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(try_from = "SerdeImage", into = "SerdeImage")]
pub struct Image {
    /// The underlying dynamic image.
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

/// Node that decodes a frame into an image.
/// Takes a `Frame` input and outputs a decoded `Image`.
#[derive(CytosNode, Default)]
struct ImageDecoder {
    #[cytos(input)]
    frame: Prop<Frame>,

    #[cytos(output)]
    decoded: Prop<Image>,
}

impl Stepper for ImageDecoder {
    fn step(&mut self) -> Result<()> {
        match image::load_from_memory(self.frame.as_u8()) {
            Ok(image) => {
                *self.decoded = Image { image };
                Ok(())
            }
            _ => Err("cannot decode image".into()),
        }
    }
}

/// Node that saves an image to a file.
/// Takes an `Image` and a filename, saves the image to disk.
#[derive(CytosNode, Default)]
struct ImageSave {
    #[cytos(input)]
    input: Prop<Image>,

    #[cytos(input)]
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

/// Registers the image I/O nodes into the Cytos registry.
pub fn load_registry(registry: &mut DynamicLoadingRegistryWrapper) {
    registry
        .add("ImageDecoder", ImageDecoder::default)
        .add("ImageSave", ImageSave::default);
}
