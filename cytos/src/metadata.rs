use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::ParamId;

/// Metadata describing a node type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub name: String,
    pub description: String,
    pub input_ids: Vec<ParamId>,
    pub output_ids: Vec<ParamId>,
    pub params: HashMap<ParamId, ParamInfo>,
}

/// Information about a single parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamInfo {
    pub name: String,
    pub description: String,
    pub direction: ParamDirection,
    pub type_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParamDirection {
    Input,
    Output,
}

/// Trait for types that can provide metadata
pub trait MetadataProvider {
    fn metadata() -> NodeMetadata;
}
