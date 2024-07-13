//! Struct to manage graph architecture.

pub mod graph;
pub mod props;
pub mod repr;
pub mod runner;
mod system;
mod traits;
pub mod value;

use std::fmt::Display;

use crate::utils::{string_to_u64, u64_to_string};

pub use self::props::{InputProp, OutputProp};
pub use self::traits::{Stepper, Transformer};
use serde::{Deserialize, Serialize};
pub use system::System;
pub use value::Value;

pub type Result<T> = std::result::Result<T, &'static str>;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
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
