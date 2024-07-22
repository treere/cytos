use super::Result;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

macro_rules! serde_u64 {
    ($struct_name:ident) => {
        impl Serialize for $struct_name {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let value = u64_to_string(self.0);
                serializer.serialize_str(&value)
            }
        }

        impl<'de> Deserialize<'de> for $struct_name {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                let value = string_to_u64(&value).unwrap();
                Ok($struct_name(value))
            }
        }
    };
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct GraphId(pub u64);

impl From<u64> for GraphId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl TryFrom<&str> for GraphId {
    type Error = &'static str;

    fn try_from(value: &str) -> std::prelude::v1::Result<Self, Self::Error> {
        string_to_u64(value).map(Self)
    }
}

impl TryFrom<&String> for GraphId {
    type Error = &'static str;

    fn try_from(value: &String) -> std::prelude::v1::Result<Self, Self::Error> {
        string_to_u64(value).map(Self)
    }
}

impl Display for GraphId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", u64_to_string(self.0))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct NodeId(pub u64);

impl From<u64> for NodeId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl TryFrom<&str> for NodeId {
    type Error = &'static str;

    fn try_from(value: &str) -> std::prelude::v1::Result<Self, Self::Error> {
        string_to_u64(value).map(Self)
    }
}

impl TryFrom<&String> for NodeId {
    type Error = &'static str;

    fn try_from(value: &String) -> std::prelude::v1::Result<Self, Self::Error> {
        string_to_u64(value).map(Self)
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", u64_to_string(self.0))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct ParamId(pub u64);

impl From<u64> for ParamId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl TryFrom<&str> for ParamId {
    type Error = &'static str;

    fn try_from(value: &str) -> std::prelude::v1::Result<Self, Self::Error> {
        string_to_u64(value).map(Self)
    }
}

impl TryFrom<&String> for ParamId {
    type Error = &'static str;

    fn try_from(value: &String) -> std::prelude::v1::Result<Self, Self::Error> {
        string_to_u64(value).map(Self)
    }
}

impl Display for ParamId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", u64_to_string(self.0))
    }
}

fn string_to_u64(s: impl AsRef<str>) -> Result<u64> {
    u64::from_str_radix(s.as_ref(), 36).or(Err("invalid string"))
}

fn u64_to_string(val: u64) -> String {
    format_radix(val, 36)
}

fn format_radix(mut x: u64, radix: u64) -> String {
    let mut result = vec![];

    loop {
        let m = x % radix;
        x /= radix;

        // will panic if you use a bad radix (< 2 or > 36).
        result.push(std::char::from_digit(m as u32, radix as u32).unwrap());
        if x == 0 {
            break;
        }
    }
    result.into_iter().rev().collect()
}

serde_u64!(GraphId);
serde_u64!(NodeId);
serde_u64!(ParamId);
