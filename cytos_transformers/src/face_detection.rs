use cytos::{props::Ownable, Prop, Stepper};
use cytos_derive::CytosNode;
use rustface::{Detector, ImageData};
use serde::{Deserialize, Serialize};

use crate::decoder::Image;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rectangle {
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
pub struct FaceDetection {
    #[input]
    image: Prop<Image>,

    #[output]
    facesinfo: Prop<Vec<Rectangle>>,

    detector: Option<Box<dyn Detector>>,
}

impl Stepper for FaceDetection {
    fn step(&mut self) -> cytos::Result<()> {
        if let Some(detector) = &mut self.detector {
            let width = self.image.image.width();
            let height = self.image.image.height();
            let bytes = &*self.image.image;
            let mut image = ImageData::new(bytes, width, height);
            *self.facesinfo = detector
                .detect(&mut image)
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
        let detector = rustface::create_detector("/path/to/model")
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
