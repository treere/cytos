# Cytos Transformers

A collection of transformer nodes for the Cytos dataflow framework. This crate provides various nodes for image processing, logic operations, machine learning, I/O, and more.

## Features

- **Image Processing**: Decode/save images, apply filters (blur, resize, crop), compute statistics
- **Logic Operations**: Boolean gates (AND, OR, XOR, NOT), comparisons (LT, EQ, etc.)
- **Machine Learning**: Face detection, YOLO object detection
- **Data Sources**: File reading, camera capture (Linux via rscam)
- **Time Management**: Timers, rate limiters, sleep nodes
- **I/O**: Printing values, sending data via HTTP

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
cytos_transformers = { path = "../cytos_transformers" }
```

Then load the nodes into your Cytos registry:

```rust
use cytos::loader::DynamicLoadingRegistryWrapper;
use cytos_transformers::load_registry;

let mut registry = DynamicLoadingRegistryWrapper::new();
// Load all transformer nodes
unsafe { load_registry(&mut registry); }
```

## Available Nodes

### Image Operations
- `ImageDecoder`: Decode frames to images
- `ImageSave`: Save images to files
- `Blur`, `FastBlur`, `Brighten`: Image adjustments
- `Resize`, `Crop`: Image transformations
- `Filter3x3`: Custom convolution filters
- `ImageMean`: Compute average pixel value

### Logic and Comparisons
- `And`, `Or`, `Xor`, `Not`: Boolean operations
- `Lt`, `Lte`, `Gt`, `Gte`, `Eq`: Comparisons for various types

### Machine Learning
- `FaceDetection`: Detect faces in images
- `Yolo`: Object detection using YOLOv8
- `YoloRunner`, `YoloDecoder`: YOLO pipeline components

### Data Sources
- `FileSource`: Read files as frames
- `RscamSource`: Capture from cameras

### Time and Control
- `Timer`: Measure elapsed time and FPS
- `Sleep`: Pause execution
- `RateLimiter`: Control execution frequency

### Utilities
- `Print*`: Print values of various types
- `WebSender*`: Send data via HTTP POST
- `AddValue`, `IncrementalGenerator`: Basic arithmetic

## Building

Ensure you have the required dependencies:

- Image processing: `image` crate
- Camera support: `rscam` (Linux only)
- ML models: `ort` for ONNX, `rustface` for face detection

Some models need to be downloaded - check the `models/` directory.

## License

This project is part of the Cytos framework.</content>
<parameter name="filePath">/home/treere/Documents/programming/rust/cytos/cytos_transformers/README.md