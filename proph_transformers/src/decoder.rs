use image::{DynamicImage, ImageBuffer};
use proph::architecture::{Done, InputProp, OutputProp, Stepper};
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
    fn step(&mut self) -> Done {
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
    fn step(&mut self) -> Done {
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

#[derive(TransFn, Default)]
pub struct TurboImageDecoder {
    frame: InputProp<Vec<u8>>,

    decoded: OutputProp<Image>,
}

impl Stepper for TurboImageDecoder {
    fn step(&mut self) -> Done {
        let image: image::RgbImage =
            turbojpeg::decompress_image(&self.frame.get()[..]).or(Err("cannot deocde"))?;

        *self.decoded.set() = Image { data: image.into() };

        Ok(())
    }
}
