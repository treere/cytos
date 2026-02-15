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
    /// The directions of this parameter (input/output).
    pub directions: Vec<ParamDirection>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_direction_equality() {
        assert_eq!(ParamDirection::Input, ParamDirection::Input);
        assert_eq!(ParamDirection::Output, ParamDirection::Output);
        assert_ne!(ParamDirection::Input, ParamDirection::Output);
    }

    #[test]
    fn test_param_direction_clone() {
        let input = ParamDirection::Input;
        let cloned = input.clone();
        assert_eq!(input, cloned);
    }

    #[test]
    fn test_param_info_creation() {
        let param_info = ParamInfo {
            id: ParamId(0),
            name: "test_param".to_string(),
            description: "A test parameter".to_string(),
            directions: vec![ParamDirection::Input],
            type_name: "Prop<i32>".to_string(),
        };

        assert_eq!(param_info.id, ParamId(0));
        assert_eq!(param_info.name, "test_param");
        assert_eq!(param_info.description, "A test parameter");
        assert_eq!(param_info.directions, vec![ParamDirection::Input]);
        assert_eq!(param_info.type_name, "Prop<i32>");
    }

    #[test]
    fn test_param_info_with_multiple_directions() {
        let param_info = ParamInfo {
            id: ParamId(1),
            name: "bidirectional".to_string(),
            description: "A bidirectional parameter".to_string(),
            directions: vec![ParamDirection::Input, ParamDirection::Output],
            type_name: "Prop<i32>".to_string(),
        };

        assert_eq!(param_info.directions.len(), 2);
        assert!(param_info.directions.contains(&ParamDirection::Input));
        assert!(param_info.directions.contains(&ParamDirection::Output));
    }

    #[test]
    fn test_node_metadata_creation() {
        let params = vec![
            ParamInfo {
                id: ParamId(0),
                name: "input".to_string(),
                description: "Input value".to_string(),
                directions: vec![ParamDirection::Input],
                type_name: "Prop<i32>".to_string(),
            },
            ParamInfo {
                id: ParamId(1),
                name: "output".to_string(),
                description: "Output value".to_string(),
                directions: vec![ParamDirection::Output],
                type_name: "Prop<i32>".to_string(),
            },
        ];

        let metadata = NodeMetadata {
            name: "TestNode".to_string(),
            description: "A test node for testing".to_string(),
            params,
        };

        assert_eq!(metadata.name, "TestNode");
        assert_eq!(metadata.description, "A test node for testing");
        assert_eq!(metadata.params.len(), 2);
    }

    #[test]
    fn test_node_metadata_serialization() {
        let metadata = NodeMetadata {
            name: "TestNode".to_string(),
            description: "Test".to_string(),
            params: vec![ParamInfo {
                id: ParamId(0),
                name: "param".to_string(),
                description: "A parameter".to_string(),
                directions: vec![ParamDirection::Input],
                type_name: "Prop<i32>".to_string(),
            }],
        };

        let serialized = serde_json::to_string(&metadata).expect("should serialize");
        let deserialized: NodeMetadata =
            serde_json::from_str(&serialized).expect("should deserialize");

        assert_eq!(deserialized.name, metadata.name);
        assert_eq!(deserialized.description, metadata.description);
        assert_eq!(deserialized.params.len(), metadata.params.len());
    }

    #[test]
    fn test_param_direction_serialization() {
        let input = ParamDirection::Input;
        let output = ParamDirection::Output;

        let input_serialized = serde_json::to_string(&input).expect("should serialize");
        let output_serialized = serde_json::to_string(&output).expect("should serialize");

        let input_deserialized: ParamDirection =
            serde_json::from_str(&input_serialized).expect("should deserialize");
        let output_deserialized: ParamDirection =
            serde_json::from_str(&output_serialized).expect("should deserialize");

        assert_eq!(input, input_deserialized);
        assert_eq!(output, output_deserialized);
    }

    #[test]
    fn test_empty_params() {
        let metadata = NodeMetadata {
            name: "EmptyNode".to_string(),
            description: "Node with no params".to_string(),
            params: vec![],
        };

        assert!(metadata.params.is_empty());
    }

    #[test]
    fn test_metadata_provider_trait() {
        // Test that we can implement the trait
        struct TestProvider;

        impl MetadataProvider for TestProvider {
            fn metadata() -> NodeMetadata {
                NodeMetadata {
                    name: "TestProvider".to_string(),
                    description: "Test provider for testing".to_string(),
                    params: vec![],
                }
            }
        }

        let metadata = TestProvider::metadata();
        assert_eq!(metadata.name, "TestProvider");
    }

    #[test]
    fn test_node_metadata_debug() {
        let metadata = NodeMetadata {
            name: "DebugNode".to_string(),
            description: "For debugging".to_string(),
            params: vec![],
        };

        let debug_str = format!("{:?}", metadata);
        assert!(debug_str.contains("DebugNode"));
        assert!(debug_str.contains("For debugging"));
    }

    #[test]
    fn test_param_info_clone() {
        let param_info = ParamInfo {
            id: ParamId(5),
            name: "clone_test".to_string(),
            description: "Testing clone".to_string(),
            directions: vec![ParamDirection::Input],
            type_name: "Prop<String>".to_string(),
        };

        let cloned = param_info.clone();
        assert_eq!(param_info.id, cloned.id);
        assert_eq!(param_info.name, cloned.name);
        assert_eq!(param_info.description, cloned.description);
        assert_eq!(param_info.directions, cloned.directions);
        assert_eq!(param_info.type_name, cloned.type_name);
    }
}
