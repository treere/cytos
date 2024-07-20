//! Struct to manage graph architecture.

pub mod graph;
pub mod props;
pub mod repr;
pub mod runner;
mod traits;
pub mod value;
mod ids;


pub use self::props::{InputProp, OutputProp};
pub use self::traits::{Stepper, Transformer};
pub use runner::System;
pub use value::Value;
pub use ids::{GraphId, NodeId, ParamId};

pub type Result<T> = std::result::Result<T, &'static str>;

