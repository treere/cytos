//! Data source transformer nodes for Cytos.
//!
//! This module provides nodes for reading data from various sources:
//! - File reading (loading file contents as frames)
//! - Camera capture (Linux via rscam library)
//!
//! These nodes are typically used at the beginning of processing pipelines
//! to ingest data for further processing.

use std::io::Read;

use cytos::{ChangeCheckProp, Prop, Result, Stepper, loader::DynamicLoadingRegistryWrapper};
use cytos_derive::CytosNode;
use rscam::Camera;

use crate::types::Frame;

/// Node that reads a file and outputs its contents as a frame.
///
/// On initialization, reads the entire file specified by the `filename` input
/// and stores it as a `Frame`. On termination, clears the frame.
#[derive(CytosNode, Default)]
struct File {
    /// The path to the file to read
    #[cytos(input)]
    filename: Prop<String>,

    /// The file contents as a frame
    #[cytos(output)]
    frame: Prop<Frame>,
}

impl Stepper for File {
    fn step(&mut self) -> Result<()> {
        Ok(())
    }

    fn initialize(&mut self) -> Result<()> {
        let mut f = std::fs::File::open(&*self.filename)?;
        let mut v = Vec::new();

        f.read_to_end(&mut v)?;
        *self.frame = v.into();
        Ok(())
    }

    fn terminate(&mut self) -> Result<()> {
        *self.frame = Frame::default();
        Ok(())
    }
}

/// Node that captures frames from a camera using the rscam library (Linux only).
///
/// On initialization, opens the camera device and starts capture with the
/// specified parameters. On each step, captures a frame and outputs it.
/// Supports MJPEG format with configurable resolution and frame interval.
///
/// Configuration changes (device path, resolution, interval) are detected and
/// trigger camera re-initialization automatically.
///
/// # Platform Support
///
/// This node is only available on Linux systems with `Video4Linux2` (V4L2) support.
#[derive(CytosNode)]
struct Rscam {
    /// The camera device path (e.g., "/dev/video0")
    #[cytos(input)]
    filename: ChangeCheckProp<String>,
    /// The capture interval as (numerator, denominator) seconds
    #[cytos(input)]
    interval: ChangeCheckProp<(u32, u32)>,
    /// The capture resolution as (width, height) in pixels
    #[cytos(input)]
    resolution: ChangeCheckProp<(u32, u32)>,

    /// The captured frame
    #[cytos(output)]
    frame: Prop<Frame>,

    camera: Option<Camera>,
}

impl Default for Rscam {
    fn default() -> Self {
        Self {
            filename: ChangeCheckProp::new("/dev/video0".to_owned()),
            interval: ChangeCheckProp::new((1, 30)),
            resolution: ChangeCheckProp::new((1280, 720)),
            frame: Prop::new(Frame::default()),
            camera: None,
        }
    }
}

impl Rscam {
    fn init_camera(&mut self) -> Result<()> {
        // Drop existing camera first
        if let Some(camera) = self.camera.take() {
            drop(camera);
        }

        let mut camera =
            rscam::new(&self.filename).map_err(|x| format!("cannot open camera: {x}"))?;

        camera
            .start(&rscam::Config {
                interval: *self.interval,
                resolution: *self.resolution,
                format: b"MJPG",
                nbuffers: 4,
                ..Default::default()
            })
            .map_err(|x| format!("cannot start camera: {x}"))?;

        self.camera = Some(camera);

        // Clear changed flags after successful initialization
        self.filename.clear_changed();
        self.interval.clear_changed();
        self.resolution.clear_changed();

        Ok(())
    }
}

impl Stepper for Rscam {
    fn step(&mut self) -> Result<()> {
        // Reinitialize camera if configuration changed
        if self.filename.is_changed() || self.interval.is_changed() || self.resolution.is_changed()
        {
            self.init_camera()?;
        }

        if let Some(camera) = self.camera.as_ref() {
            let frame = camera.capture().or(Err("cannot capture"))?;

            *self.frame = frame.into();

            Ok(())
        } else {
            unreachable!()
        }
    }

    fn initialize(&mut self) -> Result<()> {
        self.init_camera()
    }

    fn terminate(&mut self) -> Result<()> {
        *self.frame = Frame::default();
        let camera = self.camera.take();
        drop(camera);
        Ok(())
    }
}

pub fn load_registry(registry: &mut DynamicLoadingRegistryWrapper) {
    registry
        .add("RscamSource", Rscam::default)
        .add("FileSource", File::default);
}
