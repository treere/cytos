use image::{DynamicImage, ImageBuffer};
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
        if let Ok(image) = image::load_from_memory(&self.frame.get()[..]) {
            *self.decoded.set() = Image { data: image };
            Ok(())
        } else {
            Err("cannot decode image")
        }
    }
}

#[derive(TransFn, Default)]
pub struct ZuneImageDecoder {
    frame: InputProp<Vec<u8>>,

    decoded: OutputProp<Image>,
}

impl Stepper for ZuneImageDecoder {
    fn step(&mut self) -> Result<()> {
        let mut decoder = zune_jpeg::JpegDecoder::new(&self.frame.get()[..]);

        decoder.decode_headers().unwrap();
        let image_info = decoder.info().unwrap();
        let image: ImageBuffer<image::Rgb<u8>, Vec<u8>> = ImageBuffer::from_vec(
            image_info.width.into(),
            image_info.height.into(),
            decoder.decode().or(Err("cannot decode"))?,
        )
        .unwrap();

        *self.decoded.set() = Image {
            data: DynamicImage::ImageRgb8(image),
        };

        Ok(())
    }
}
