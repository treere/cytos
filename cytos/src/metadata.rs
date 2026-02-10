//! Metadata module providing type information for nodes and their parameters.
//!
//! This module defines structures for describing node types, their inputs, outputs,
//! and parameters. This metadata is used for introspection and runtime inspection
//! of graph nodes.

use serde::{Deserialize, Serialize};

use crate::ParamId;

/// Metadata describing a node type.
///
/// This struct contains all information needed to describe a node type,
/// including its name, description, input/output parameter IDs, and
/// detailed information about each parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    /// The name of the node type.
    pub name: String,
    /// A description of what the node does.
    pub description: String,
    /// A map of parameter IDs to their detailed information.
    pub params: Vec<ParamInfo>,
}

/// Information about a single parameter.
///
/// This struct describes a parameter's name, purpose, direction (input/output),
/// and its type name.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamInfo {
    /// Param identifier
    pub id: ParamId,
    /// The human-readable name of the parameter.
    pub name: String,
    /// A description of the parameter's purpose.
    pub description: String,
    /// Whether this is an input or output parameter.
    pub direction: ParamDirection,
    /// The Rust type name of the parameter (e.g., "`Prop<i32>`").
    pub type_name: String,
}

/// The direction of a parameter.
///
/// Parameters can be either inputs (data flowing into the node)
/// or outputs (data flowing out of the node).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParamDirection {
    /// An input parameter that receives data from upstream nodes or external sources.
    Input,
    /// An output parameter that produces data for downstream nodes.
    Output,
}

/// Trait for types that can provide metadata.
///
/// Implementors of this trait define the metadata for a node type,
/// which is used by the registry and graph system for introspection.
pub trait MetadataProvider {
    /// Returns the metadata for this node type.
    fn metadata() -> NodeMetadata;
}
