//! Struct to manage graph architecture.

pub mod graph;
mod ids;
mod node;
pub mod props;
pub mod system;
mod traits;
pub mod value;

pub use self::props::{InputProp, OutputProp};
pub use self::traits::{Stepper, Transformer};
pub use ids::{GraphId, NodeId, ParamId};
pub use system::System;
pub use value::Value;

pub type Result<T> = std::result::Result<T, &'static str>;
