use proph::architecture::{OutputProp, Stepper};
use proph_derive::TransFn;
use rscam::Camera;

#[derive(TransFn, Default)]
pub struct Rscam {
    camera: Option<Camera>,

    frame: OutputProp<(Vec<u8>, (u32, u32), [u8; 4])>,
}

impl Stepper for Rscam {
    fn step(&mut self) -> Result<(), &'static str> {
        if let Some(camera) = self.camera.as_ref() {
            let frame = camera.capture().map_err(|_| "cannot capture")?;

            *self.frame.set() = (
                Vec::from(&frame[..]),
                frame.resolution.clone(),
                frame.format,
            );

            Ok(())
        } else {
            unreachable!()
        }
    }

    fn initialize(&mut self) -> Result<(), &'static str> {
        let mut camera = rscam::new("/dev/video0").unwrap();

        camera
            .start(&rscam::Config {
                interval: (1, 30), // 30 fps.
                resolution: (1280, 720),
                format: b"MJPG",
                ..Default::default()
            })
            .unwrap();

        self.camera = Some(camera);
        Ok(())
    }
}
