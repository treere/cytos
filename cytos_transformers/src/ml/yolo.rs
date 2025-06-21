use cytos::{Prop, Stepper, loader::DynamicLoadingRegistryWrapper};
use cytos_derive::CytosNode;
use image::{GenericImageView, imageops::FilterType};
use ndarray::{Array, Axis, s};
use ort::{
    inputs,
    session::{Session, SessionOutputs},
    value::TensorRef,
};
use serde::{Deserialize, Serialize};

const YOLO: &[u8] = include_bytes!("../../models/yolov8m.onnx");

use crate::imageio::Image;

use super::Buffer;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct BoundingBox {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
}

fn intersection(box1: &BoundingBox, box2: &BoundingBox) -> f32 {
    (box1.x2.min(box2.x2) - box1.x1.max(box2.x1)) * (box1.y2.min(box2.y2) - box1.y1.max(box2.y1))
}

fn union(box1: &BoundingBox, box2: &BoundingBox) -> f32 {
    (box1.x2 - box1.x1).mul_add(box1.y2 - box1.y1, (box2.x2 - box2.x1) * (box2.y2 - box2.y1))
        - intersection(box1, box2)
}

#[rustfmt::skip]
const YOLO_CLASS_LABELS: [&str; 80] = [
    "person", "bicycle", "car", "motorcycle", "airplane", "bus", "train", "truck", "boat", "traffic light",
	"fire hydrant", "stop sign", "parking meter", "bench", "bird", "cat", "dog", "horse", "sheep", "cow", "elephant",
	"bear", "zebra", "giraffe", "backpack", "umbrella", "handbag", "tie", "suitcase", "frisbee", "skis", "snowboard",
	"sports ball", "kite", "baseball bat", "baseball glove", "skateboard", "surfboard", "tennis racket", "bottle",
	"wine glass", "cup", "fork", "knife", "spoon", "bowl", "banana", "apple", "sandwich", "orange", "broccoli",
	"carrot", "hot dog", "pizza", "donut", "cake", "chair", "couch", "potted plant", "bed", "dining table", "toilet",
	"tv", "laptop", "mouse", "remote", "keyboard", "cell phone", "microwave", "oven", "toaster", "sink", "refrigerator",
	"book", "clock", "vase", "scissors", "teddy bear", "hair drier", "toothbrush"
];

#[derive(CytosNode, Default)]
struct Yolo {
    #[input]
    input: Prop<Image>,

    #[output]
    resized: Prop<Image>,

    #[output]
    buffer: Prop<Buffer>,

    #[output]
    results: Prop<Vec<(BoundingBox, &'static str, f32)>>,

    model: Option<Session>,
}

impl Stepper for Yolo {
    #[allow(clippy::many_single_char_names)]
    fn step(&mut self) -> cytos::Result<()> {
        let model = self.model.as_mut().unwrap();
        let original_img = &self.input.image;
        let img_width = original_img.width();
        let img_height = original_img.height();
        *self.resized = Image {
            image: original_img.resize_exact(640, 640, FilterType::CatmullRom),
        };
        let mut input = Array::zeros((1, 3, 640, 640));
        for pixel in self.resized.image.pixels() {
            let x = pixel.0 as _;
            let y = pixel.1 as _;
            let [r, g, b, _] = pixel.2.0;
            input[[0, 0, y, x]] = f32::from(r) / 255.;
            input[[0, 1, y, x]] = f32::from(g) / 255.;
            input[[0, 2, y, x]] = f32::from(b) / 255.;
        }

        let images = TensorRef::from_array_view(&input)?;
        let outputs: SessionOutputs = model.run(inputs! {"images" => images})?;
        *self.buffer = Buffer(input);
        let output = outputs["output0"]
            .try_extract_array::<f32>()?
            .t()
            .into_owned();

        let mut boxes = Vec::new();
        let output = output.slice(s![.., .., 0]);
        for row in output.axis_iter(Axis(0)) {
            let row: Vec<_> = row.iter().copied().collect();
            let (class_id, prob) = row
                .iter()
                // skip bounding box coordinates
                .skip(4)
                .enumerate()
                .map(|(index, value)| (index, *value))
                .reduce(|accum, row| if row.1 > accum.1 { row } else { accum })
                .unwrap();
            if prob < 0.5 {
                continue;
            }
            let label = YOLO_CLASS_LABELS[class_id];
            let xc = row[0] / 640. * (img_width as f32);
            let yc = row[1] / 640. * (img_height as f32);
            let w = row[2] / 640. * (img_width as f32);
            let h = row[3] / 640. * (img_height as f32);
            boxes.push((
                BoundingBox {
                    x1: xc - w / 2.,
                    y1: yc - h / 2.,
                    x2: xc + w / 2.,
                    y2: yc + h / 2.,
                },
                label,
                prob,
            ));
        }

        boxes.sort_by(|box1, box2| box2.2.total_cmp(&box1.2));
        self.results.clear();

        while !boxes.is_empty() {
            self.results.push(boxes[0]);
            boxes = boxes
                .iter()
                .filter(|box1| {
                    intersection(&boxes[0].0, &box1.0) / union(&boxes[0].0, &box1.0) < 0.7
                })
                .copied()
                .collect();
        }
        Ok(())
    }

    fn initialize(&mut self) -> cytos::Result<()> {
        self.model = Some(Session::builder()?.commit_from_memory(YOLO)?);

        Ok(())
    }

    fn terminate(&mut self) -> cytos::Result<()> {
        self.model = None;
        Ok(())
    }
}

#[derive(CytosNode, Default)]
struct YoloRunner {
    #[input]
    input: Prop<Buffer>,

    #[output]
    output: Prop<Vec<Vec<f32>>>,

    model: Option<Session>,
}

impl Stepper for YoloRunner {
    fn step(&mut self) -> cytos::Result<()> {
        let model = self.model.as_mut().unwrap();
        let input = &(*self.input).0;
        let images = TensorRef::from_array_view(input)?;
        let outputs: SessionOutputs = model.run(inputs! { "images" => images})?;

        let output = outputs["output0"]
            .try_extract_array::<f32>()?
            .t()
            .into_owned();

        let output = output.slice(s![.., .., 0]);
        *self.output = output
            .axis_iter(Axis(0))
            .map(|row| row.iter().copied().collect::<Vec<_>>())
            .collect::<Vec<_>>();

        Ok(())
    }

    fn initialize(&mut self) -> cytos::Result<()> {
        self.model = Some(Session::builder()?.commit_from_memory(YOLO)?);

        Ok(())
    }

    fn terminate(&mut self) -> cytos::Result<()> {
        self.model = None;
        Ok(())
    }
}

#[derive(CytosNode, Default)]
struct YoloDecoder {
    #[input]
    original: Prop<Image>,

    #[input]
    detections: Prop<Vec<Vec<f32>>>,

    #[input]
    threshold: Prop<f32>,

    #[output]
    results: Prop<Vec<(BoundingBox, &'static str, f32)>>,
}

impl Stepper for YoloDecoder {
    fn step(&mut self) -> cytos::Result<()> {
        let original_img = &self.original.image;
        let img_width = original_img.width();
        let img_height = original_img.height();

        let mut boxes = Vec::new();

        for row in &(*self.detections) {
            let (class_id, prob) = row
                .iter()
                // skip bounding box coordinates
                .skip(4)
                .enumerate()
                .map(|(index, value)| (index, *value))
                .reduce(|accum, row| if row.1 > accum.1 { row } else { accum })
                .unwrap();
            if prob < *self.threshold {
                continue;
            }
            let label = YOLO_CLASS_LABELS[class_id];
            let xc = row[0] / 640. * (img_width as f32);
            let yc = row[1] / 640. * (img_height as f32);
            let w = row[2] / 640. * (img_width as f32);
            let h = row[3] / 640. * (img_height as f32);
            boxes.push((
                BoundingBox {
                    x1: xc - w / 2.,
                    y1: yc - h / 2.,
                    x2: xc + w / 2.,
                    y2: yc + h / 2.,
                },
                label,
                prob,
            ));
        }

        boxes.sort_by(|box1, box2| box2.2.total_cmp(&box1.2));
        self.results.clear();

        while !boxes.is_empty() {
            self.results.push(boxes[0]);
            boxes = boxes
                .iter()
                .filter(|box1| {
                    intersection(&boxes[0].0, &box1.0) / union(&boxes[0].0, &box1.0) < 0.7
                })
                .copied()
                .collect();
        }
        Ok(())
    }
}

pub fn load_registry(registry: &mut DynamicLoadingRegistryWrapper) {
    registry.add("Yolo", Yolo::default);
    registry.add("YoloRunner", YoloRunner::default);
    registry.add("YoloDecoder", YoloDecoder::default);
}
