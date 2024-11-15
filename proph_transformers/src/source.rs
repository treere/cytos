use proph::architecture::{InputProp, OutputProp, Result, Stepper};
use proph_derive::TransFn;
use rscam::Camera;

use crate::types::Frame;

#[derive(TransFn)]
pub struct Rscam {
    filename: InputProp<String>,
    interval: InputProp<(u32, u32)>,
    resolution: InputProp<(u32, u32)>,

    frame: OutputProp<Frame>,

    camera: Option<Camera>,
}

impl Default for Rscam {
    fn default() -> Self {
        Self {
            filename: InputProp::new("/dev/video0".to_owned()),
            interval: InputProp::new((1, 30)),
            resolution: InputProp::new((1280, 720)),
            frame: OutputProp::new(Frame::default()),
            camera: None,
        }
    }
}

impl Stepper for Rscam {
    fn step(&mut self) -> Result<()> {
        if let Some(camera) = self.camera.as_ref() {
            let frame = camera.capture().or(Err("cannot capture"))?;

            *self.frame = frame.into();

            Ok(())
        } else {
            unreachable!()
        }
    }

    fn initialize(&mut self) -> Result<()> {
        let mut camera =
            rscam::new(&self.filename).map_err(|x| format!("cannot open camera: {}", x))?;

        camera
            .start(&rscam::Config {
                interval: *self.interval,
                resolution: *self.resolution,
                format: b"MJPG",
                nbuffers: 4,
                ..Default::default()
            })
            .map_err(|x| format!("cannot start camera: {}", x))?;

        self.camera = Some(camera);
        Ok(())
    }

    fn terminate(&mut self) -> Result<()> {
        if let Some(mut camera) = self.camera.take() {
            camera.stop().or(Err("cannot stop".into()))
        } else {
            Ok(())
        }
    }
}
