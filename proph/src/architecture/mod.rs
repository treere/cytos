//! Struct to manage graph architecture.

pub mod graph;
pub mod props;
pub mod runner;
mod traits;

pub use self::props::{Dumper, InputProp, OutputProp};
pub use self::traits::{Stepper, Transformer};

pub type NodeId = String;
pub type ParamId = String;

pub type Result<T> = std::result::Result<T, &'static str>;
pub type Value = serde_json::Value;

pub fn load_value_from_string(s: String) -> Result<Value> {
    serde_json::to_value(s).map_err(|_| "Cannot decode")
}
