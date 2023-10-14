use proph::architecture::{Done, OutputProp, Stepper};
use proph_derive::TransFn;
use rscam::Camera;

#[derive(TransFn, Default)]
pub struct Rscam {
    camera: Option<Camera>,

    frame: OutputProp<Vec<u8>>,
}

impl Stepper for Rscam {
    fn step(&mut self) -> Done {
        if let Some(camera) = self.camera.as_ref() {
            let frame = camera.capture().map_err(|_| "cannot capture")?;

            *self.frame.set() = Vec::from_iter(frame.iter().cloned());

            Ok(())
        } else {
            unreachable!()
        }
    }

    fn initialize(&mut self) -> Done {
        let mut camera = rscam::new("/dev/video0").unwrap();

        camera
            .start(&rscam::Config {
                interval: (1, 30), // 30 fps.
                resolution: (1280, 720),
                format: b"MJPG",
                nbuffers: 4,
                ..Default::default()
            })
            .unwrap();

        self.camera = Some(camera);
        Ok(())
    }
}
