//! Struct to manage graph architecture.

pub mod graph;
pub mod props;
pub mod runner;
mod traits;

pub use self::props::{InputProp, OutputProp};
pub use self::traits::{Stepper, Transformer};

pub type NodeId = String;
pub type ParamId = String;

pub type Result<T> = std::result::Result<T, &'static str>;
pub type Done = Result<()>;
pub type Value = serde_json::Value;
