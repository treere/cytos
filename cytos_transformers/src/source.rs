use std::io::Read;

use cytos::{Prop, Result, Stepper};
use cytos_derive::CytosNode;
use rscam::Camera;

use crate::types::Frame;

#[derive(CytosNode, Default)]
pub struct File {
    #[input]
    filename: Prop<String>,

    #[output]
    frame: Prop<Frame>,
}

impl Stepper for File {
    fn step(&mut self) -> Result<()> {
        Ok(())
    }

    fn initialize(&mut self) -> Result<()> {
        let mut f = std::fs::File::open(&*self.filename)?;
        let mut v = Vec::new();

        f.read_exact(&mut v)?;
        *self.frame = v.into();
        Ok(())
    }

    fn terminate(&mut self) -> Result<()> {
        *self.frame = Frame::default();
        Ok(())
    }
}

#[derive(CytosNode)]
pub struct Rscam {
    #[input]
    filename: Prop<String>,
    #[input]
    interval: Prop<(u32, u32)>,
    #[input]
    resolution: Prop<(u32, u32)>,

    #[output]
    frame: Prop<Frame>,

    camera: Option<Camera>,
}

impl Default for Rscam {
    fn default() -> Self {
        Self {
            filename: Prop::new("/dev/video0".to_owned()),
            interval: Prop::new((1, 30)),
            resolution: Prop::new((1280, 720)),
            frame: Prop::new(Frame::default()),
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
        *self.frame = Frame::default();
        let camera = self.camera.take();
        drop(camera);
        Ok(())
    }
}
