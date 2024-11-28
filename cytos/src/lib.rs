mod ids;
mod node;
mod transfomer;
pub mod graph;
pub mod loader;
pub mod props;
pub mod repr;
pub mod system;
pub mod value;

use std::error::Error;

pub use self::props::{GenericOwnedProp, Prop};
pub use self::transfomer::{Stepper, Transformer};
pub use ids::{id_number_to_string, id_string_to_number, GraphId, NodeId, ParamId};
pub use system::System;
pub use value::Value;

/// Result type
pub type Result<T> = std::result::Result<T, Box<dyn Error + 'static>>;

#[cfg(test)]
pub mod test;
