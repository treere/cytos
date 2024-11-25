//! Architecture module

pub mod graph;
mod ids;
mod node;
pub mod props;
pub mod system;
mod transfomer;
pub mod value;

use std::error::Error;

pub use self::props::{GenericOwnedProp, Prop};
pub use self::transfomer::{Stepper, Transformer};
pub use ids::{GraphId, NodeId, ParamId};
pub use system::System;
pub use value::Value;

/// Result type
pub type Result<T> = std::result::Result<T, Box<dyn Error + 'static>>;
