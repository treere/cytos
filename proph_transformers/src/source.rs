use proph::architecture::{InputProp, OutputProp, Result, Stepper};
use proph_derive::TransFn;
use rscam::Camera;
use serde::{ser::SerializeSeq, Deserialize, Serialize};

pub enum FrameKind {
    Rscam(rscam::Frame),
    Raw(Vec<u8>),
}

impl serde::Serialize for FrameKind {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            FrameKind::Rscam(frame) => {
                let mut s = serializer.serialize_seq(Some(frame.len()))?;
                for i in frame.iter() {
                    s.serialize_element(i)?;
                }
                s.end()
            }
            FrameKind::Raw(_vec) => todo!(),
        }
    }
}

struct FrameKindVisitor;

impl<'de> serde::de::Visitor<'de> for FrameKindVisitor {
    type Value = FrameKind;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("A FrameKind")
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut v = vec![];
        while let Some(s) = seq.next_element()? {
            v.push(s);
        }
        Ok(FrameKind::Raw(v))
    }
}

impl<'de> serde::Deserialize<'de> for FrameKind {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(FrameKindVisitor)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Frame {
    frame: FrameKind,
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            frame: FrameKind::Raw(Vec::default()),
        }
    }
}

impl Frame {
    pub fn as_u8(&self) -> &[u8] {
        match self.frame {
            FrameKind::Rscam(ref frame) => frame,
            FrameKind::Raw(ref vec) => &vec[..],
        }
    }
}

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

            *self.frame = Frame {
                frame: FrameKind::Rscam(frame),
            };

            Ok(())
        } else {
            unreachable!()
        }
    }

    fn initialize(&mut self) -> Result<()> {
        let mut camera = rscam::new(&self.filename).unwrap();

        camera
            .start(&rscam::Config {
                interval: *self.interval,
                resolution: *self.resolution,
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
            camera.stop().or(Err("cannot stop".into()))
        } else {
            Ok(())
        }
    }
}
