use proph::architecture::{InputProp, OutputProp, Result, Stepper};
use proph_derive::TransFn;
use rscam::Camera;

#[derive(TransFn)]
pub struct Rscam {
    filename: InputProp<String>,
    interval: InputProp<(u32, u32)>,
    resolution: InputProp<(u32, u32)>,

    frame: OutputProp<Vec<u8>>,

    camera: Option<Camera>,
}

impl Default for Rscam {
    fn default() -> Self {
        Self {
            filename: InputProp::new("/dev/video0".to_owned()),
            interval: InputProp::new((1, 30)),
            resolution: InputProp::new((1280, 720)),
            frame: OutputProp::new(Vec::default()),
            camera: None,
        }
    }
}

impl Stepper for Rscam {
    fn step(&mut self) -> Result<()> {
        if let Some(camera) = self.camera.as_ref() {
            let frame = camera.capture().or(Err("cannot capture"))?;

            *self.frame.set() = Vec::from_iter(frame.iter().cloned());

            Ok(())
        } else {
            unreachable!()
        }
    }

    fn initialize(&mut self) -> Result<()> {
        let mut camera = rscam::new(self.filename.get()).unwrap();

        camera
            .start(&rscam::Config {
                interval: *self.interval.get(),
                resolution: *self.resolution.get(),
                format: b"MJPG",
                nbuffers: 4,
                ..Default::default()
            })
            .unwrap();

        self.camera = Some(camera);
        Ok(())
    }

    fn terminate(&mut self) -> Result<()> {
        if let Some(mut camera) = self.camera.take() {
            camera.stop().map_err(|_| "cannot stop")
        } else {
            Ok(())
        }
    }
}
