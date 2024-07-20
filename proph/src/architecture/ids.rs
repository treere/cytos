use serde::{Deserialize, Serialize};
use std::fmt::Display;
use super::Result;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Hash)]
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
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
