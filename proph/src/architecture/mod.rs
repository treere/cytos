//! Struct to manage graph architecture.

pub mod graph;
pub mod props;
mod traits;

pub use self::props::{InputProp, OutputProp};
pub use self::traits::{Stepper, Transformer};

pub type NodeId = String;
pub type ParamId = String;
type Path = (NodeId, ParamId);

pub type Result<T> = std::result::Result<T, &'static str>;
pub type Done = Result<()>;
