pub use cytos::{loader::DynamicLoadingRegistryWrapper, props::Ownable, Prop, Stepper};
use cytos_derive::CytosNode;
use image::GenericImageView;
use rustface::{Detector, ImageData};
use serde::{Deserialize, Serialize};

pub mod yolov8;

use crate::imageio::Image;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Rectangle {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

impl Ownable for Rectangle {
    type Value = Rectangle;

    fn to_ownable(&self) -> Self::Value {
        self.clone()
    }

    fn from_owned(v: &Self::Value) -> Self {
        v.clone()
    }
}

#[derive(CytosNode, Default)]
struct FaceDetection {
    #[input]
    image: Prop<Image>,

    #[input]
    model: Prop<String>,

    #[output]
    facesinfo: Prop<Vec<Rectangle>>,

    detector: Option<Box<dyn Detector>>,
}

impl Stepper for FaceDetection {
    fn step(&mut self) -> cytos::Result<()> {
        if let Some(detector) = &mut self.detector {
            let width = self.image.image.width();
            let height = self.image.image.height();
            let bytes = &*self.image.image.to_luma8();
            let image = ImageData::new(bytes, width, height);
            *self.facesinfo = detector
                .detect(&image)
                .into_iter()
                .map(|info| Rectangle {
                    x: info.bbox().x(),
                    y: info.bbox().y(),
                    width: info.bbox().height(),
                    height: info.bbox().width(),
                })
                .collect();

            Ok(())
        } else {
            Err("missing detector".into())
        }
    }

    fn initialize(&mut self) -> cytos::Result<()> {
        let detector = rustface::create_detector(&self.model)
            .map(|mut detector| {
                detector.set_min_face_size(20);
                detector.set_score_thresh(2.0);
                detector.set_pyramid_scale_factor(0.8);
                detector.set_slide_window_step(4, 4);
                detector
            })
            .ok();

        self.detector = detector;

        Ok(())
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Buffer(pub ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 4]>>);

impl Ownable for Buffer {
    type Value = Buffer;

    fn to_ownable(&self) -> Self::Value {
        self.clone()
    }

    fn from_owned(v: &Self::Value) -> Self {
        v.clone()
    }
}

#[derive(CytosNode, Default)]
struct Image2Buffer {
    #[input]
    input: Prop<Image>,

    #[output]
    output: Prop<Buffer>,
}

impl Stepper for Image2Buffer {
    fn step(&mut self) -> cytos::Result<()> {
        let input = &self.input.image;
        let w = input.width();
        let h = input.height();

        let mut buffer = ndarray::Array::zeros((1, 3, w as usize, h as usize));
        for pixel in input.pixels() {
            let x = pixel.0 as _;
            let y = pixel.1 as _;
            let [r, g, b, _] = pixel.2 .0;
            buffer[[0, 0, y, x]] = (r as f32) / 255.;
            buffer[[0, 1, y, x]] = (g as f32) / 255.;
            buffer[[0, 2, y, x]] = (b as f32) / 255.;
        }
        *self.output = Buffer(buffer);
        Ok(())
    }
}

pub fn load_registry(registry: &mut DynamicLoadingRegistryWrapper) {
    yolov8::load_registry(registry);
    registry.add("FaceDetection", FaceDetection::default);
    registry.add("Image2Buffer", Image2Buffer::default);
}
