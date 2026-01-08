#![deny(clippy::perf)]
#![deny(clippy::style)]
#![deny(clippy::complexity)]
#![deny(clippy::suspicious)]
#![deny(clippy::correctness)]
#![deny(clippy::nursery)]
#![deny(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]

//! # Cytos: Lightweight Processing Pipelines
//!
//! Cytos is a Rust library that provides a lightweight foundation for building processing pipelines.
//! It's designed to have minimal overhead, making it suitable for high-performance applications.
//! Cytos allows you to create multiple graphs that can run concurrently in different threads,
//! enabling efficient and scalable processing.
//!
//! ## Key Features
//!
//! * **Low-Overhead Architecture**: Cytos is designed to have minimal overhead, making it suitable for high-performance applications.
//! * **Multi-Graph Support**: Cytos allows you to create multiple graphs that can run concurrently in different threads.
//! * **Customizable Nodes and Edges**: Define your own node and edge types to suit your specific use case.
//! * **Custom Processing Logic**: Implement custom processing logic for each node and edge.

pub mod graph;
mod ids;
pub mod loader;
mod metadata;
mod node;
pub mod props;
mod queue;
pub mod repr;
pub mod system;
mod transformer;
pub mod value;

use std::error::Error;

pub use self::props::{GenericOwnedProp, Prop};
pub use self::transformer::{Stepper, Transformer};
pub use ids::{GraphId, NodeId, ParamId, id_number_to_string, id_string_to_number};
pub use metadata::{MetadataProvider, NodeMetadata, ParamDirection, ParamInfo};
pub use system::System;
pub use value::Value;

/// Result type
pub type Result<T> = std::result::Result<T, Box<dyn Error + 'static>>;

#[cfg(test)]
pub mod test;
