use proph::architecture::props::Ownable;
use serde::{ser::SerializeSeq, Deserialize, Serialize};

enum FrameKind {
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
            FrameKind::Raw(vec) => {
                let mut s = serializer.serialize_seq(Some(vec.len()))?;
                for i in vec.iter() {
                    s.serialize_element(i)?;
                }
                s.end()
            }
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

impl From<rscam::Frame> for Frame {
    fn from(value: rscam::Frame) -> Self {
        Self {
            frame: FrameKind::Rscam(value),
        }
    }
}

impl Ownable for Frame {
    type Value = Vec<u8>;

    fn to_ownable(&self) -> Self::Value {
        match &self.frame {
            FrameKind::Rscam(frame) => frame.iter().cloned().collect(),
            FrameKind::Raw(vec) => vec.clone(),
        }
    }

    fn from_owned(v: &Self::Value) -> Self {
        Self {
            frame: FrameKind::Raw(v.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use proph::architecture::Value;

    use super::*;

    #[test]
    fn test_dump_empty_frame() {
        let f = Frame::default();
        let p = Value::load(&f).expect("cannot load");

        let p: Frame = p.dump().expect("cannot dump");
        assert_eq!(0, p.as_u8().len());
    }

    #[test]
    fn test_dump_dummy_frame() {
        let f = Frame {
            frame: FrameKind::Raw(vec![1, 2, 3]),
        };
        let p = Value::load(&f).expect("cannot load");

        let p: Frame = p.dump().expect("cannot dump");
        assert_eq!(3, p.as_u8().len());
    }
}
