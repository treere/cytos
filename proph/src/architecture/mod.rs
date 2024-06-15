//! Struct to manage graph architecture.

pub mod graph;
pub mod props;
pub mod runner;
mod traits;

pub use self::props::{Dumper, InputProp, OutputProp};
pub use self::traits::{Stepper, Transformer};

pub type NodeId = u64;
pub type ParamId = u64;

pub type Result<T> = std::result::Result<T, &'static str>;
pub type Value = serde_json::Value;

