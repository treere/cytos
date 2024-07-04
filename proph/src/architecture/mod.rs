//! Struct to manage graph architecture.

pub mod graph;
pub mod props;
pub mod runner;
mod traits;
pub mod value;

pub use self::props::{InputProp, OutputProp};
pub use self::traits::{Stepper, Transformer};
pub use value::Value;

pub type NodeId = u64;
pub type ParamId = u64;

pub type Result<T> = std::result::Result<T, &'static str>;
