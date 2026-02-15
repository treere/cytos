//! Representation module for serializable graph and system definitions.
//!
//! This module provides serializable types for describing graphs and systems
//! in JSON format. These representations can be loaded from files or strings
//! and converted into executable [`Graph`] or [`System`] instances.
//!
//! # Serialization
//!
//! All types in this module implement [`serde::Deserialize`], allowing them to
//! be loaded from JSON files. The main entry point is [`GraphRepr`] for individual
//! graphs and [`SystemRepr`] for systems containing multiple graphs.
//!
//! # Key Types
//!
//! - [`GraphRepr`]: A serializable graph definition with nodes and links
//! - [`SystemRepr`]: A serializable system definition with multiple graphs
//! - [`SystemLink`]: Links between nodes in different graphs
//! - [`LinkKind`]: Behavior for inter-graph links (`Wait` or `Continue`)
//!
//! [`Graph`]: crate::graph::Graph
//! [`System`]: crate::system::System

use std::collections::HashMap;

use crate::{GraphId, NodeId, ParamId, Value};
use serde::Deserialize;

/// A deserializable representation of a node.
#[derive(Deserialize, Debug)]
pub struct NodeRepr {
    /// Type of the node
    #[serde(rename = "type")]
    pub typ: String,

    /// Properties
    #[serde(default)]
    pub props: HashMap<ParamId, Value>,
}

/// A representation of a graph to be loaded and executed.
#[derive(Deserialize, Debug)]
pub struct GraphRepr {
    /// List of links between nodes
    #[serde(default)]
    pub links: Vec<GraphLink>,

    /// Map of nodes with it's id
    pub nodes: Vec<InternalNodeRepr>,
}

/// A link between two nodes in a graph, connecting an output parameter to an input parameter.
#[derive(Deserialize, Debug)]
pub struct GraphLink {
    /// Source node param
    pub src: (NodeId, ParamId),

    /// Destination node param
    pub dst: (NodeId, ParamId),
}

/// A node representation within a graph, including its name and error handling behavior.
#[derive(Deserialize, Debug)]
pub struct InternalNodeRepr {
    /// Name
    pub name: NodeId,

    /// Node repr
    #[serde(flatten)]
    pub node: NodeRepr,

    /// On error expect behaviour
    #[serde(default)]
    pub on_error: OnError,
}

/// Graph behaviour on node failure
#[derive(Default, Debug, Deserialize)]
pub enum OnError {
    /// Skip this processing
    Skip,

    /// Continue processing the graph
    Continue,

    /// Forward the error
    #[default]
    Fail,
}

/// `SystemRepr`
///
/// Deserializable System Representation
#[derive(Deserialize, Debug)]
pub struct SystemRepr {
    /// Graphs by name
    #[serde(default)]
    pub graphs: HashMap<GraphId, GraphRepr>,

    #[serde(default)]
    /// Request between graphs
    pub requests: Vec<SystemLink>,
}

impl SystemRepr {
    /// Create a `SystemRepr` loading a file
    ///
    /// # Errors
    ///
    /// Will return `Err` if `data` is not a valid `SystemRepr`
    pub fn from_json(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }
}

/// `SystemLink` between params of different graphs
#[derive(Deserialize, Debug, Clone)]
pub struct SystemLink {
    /// Source node
    pub src: (GraphId, NodeId, ParamId),

    /// Destination node
    pub dst: (GraphId, NodeId, ParamId),

    #[serde(default = "LinkKind::wait")]
    pub kind: LinkKind,
}

/// Represents the type of link between system nodes
#[derive(Deserialize, Debug, Clone, Ord, Eq, PartialEq, PartialOrd)]
pub enum LinkKind {
    /// Waits the value
    Wait,
    /// Continue without waiting the value
    Continue,
}

impl LinkKind {
    const fn wait() -> Self {
        Self::Wait
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GraphId, NodeId, ParamId, Value};
    use std::collections::HashMap;

    #[test]
    fn test_node_repr_deserialization() {
        let json = r#"{
            "type": "TestNode",
            "props": {
                "0": 42
            }
        }"#;

        let result: Result<NodeRepr, _> = serde_json::from_str(json);
        assert!(result.is_ok());

        let node_repr = result.unwrap();
        assert_eq!(node_repr.typ, "TestNode");
        assert!(node_repr.props.contains_key(&ParamId(0)));
    }

    #[test]
    fn test_node_repr_with_empty_props() {
        let json = r#"{
            "type": "EmptyNode"
        }"#;

        let result: Result<NodeRepr, _> = serde_json::from_str(json);
        assert!(result.is_ok());

        let node_repr = result.unwrap();
        assert_eq!(node_repr.typ, "EmptyNode");
        assert!(node_repr.props.is_empty());
    }

    #[test]
    fn test_graph_repr_deserialization() {
        let json = r#"{
            "links": [],
            "nodes": [
                {
                    "name": "0",
                    "type": "TestNode"
                }
            ]
        }"#;

        let result: Result<GraphRepr, _> = serde_json::from_str(json);
        assert!(result.is_ok());

        let graph_repr = result.unwrap();
        assert!(graph_repr.links.is_empty());
        assert_eq!(graph_repr.nodes.len(), 1);
        assert_eq!(graph_repr.nodes[0].name, NodeId(0));
    }

    #[test]
    fn test_graph_repr_with_link() {
        let json = r#"{
            "links": [
                {
                    "src": ["0", "1"],
                    "dst": ["2", "3"]
                }
            ],
            "nodes": [
                {
                    "name": "0",
                    "type": "Node1"
                },
                {
                    "name": "2",
                    "type": "Node2"
                }
            ]
        }"#;

        let result: Result<GraphRepr, _> = serde_json::from_str(json);
        assert!(result.is_ok());

        let graph_repr = result.unwrap();
        assert_eq!(graph_repr.links.len(), 1);
        assert_eq!(graph_repr.links[0].src, (NodeId(0), ParamId(1)));
        assert_eq!(graph_repr.links[0].dst, (NodeId(2), ParamId(3)));
    }

    #[test]
    fn test_graph_link_creation() {
        let link = GraphLink {
            src: (NodeId(1), ParamId(0)),
            dst: (NodeId(2), ParamId(1)),
        };

        assert_eq!(link.src.0, NodeId(1));
        assert_eq!(link.src.1, ParamId(0));
        assert_eq!(link.dst.0, NodeId(2));
        assert_eq!(link.dst.1, ParamId(1));
    }

    #[test]
    fn test_internal_node_repr() {
        let node_repr = NodeRepr {
            typ: "TestNode".to_string(),
            props: HashMap::new(),
        };

        let internal = InternalNodeRepr {
            name: NodeId(42),
            node: node_repr,
            on_error: OnError::Continue,
        };

        assert_eq!(internal.name, NodeId(42));
        assert_eq!(internal.node.typ, "TestNode");
        assert!(matches!(internal.on_error, OnError::Continue));
    }

    #[test]
    fn test_on_error_variants() {
        let skip = OnError::Skip;
        let continue_ = OnError::Continue;
        let fail = OnError::Fail;

        // Test that all variants can be created
        assert!(matches!(skip, OnError::Skip));
        assert!(matches!(continue_, OnError::Continue));
        assert!(matches!(fail, OnError::Fail));
    }

    #[test]
    fn test_on_error_default() {
        let default: OnError = Default::default();
        assert!(matches!(default, OnError::Fail));
    }

    #[test]
    fn test_system_repr_empty() {
        let system_repr = SystemRepr {
            graphs: HashMap::new(),
            requests: vec![],
        };

        assert!(system_repr.graphs.is_empty());
        assert!(system_repr.requests.is_empty());
    }

    #[test]
    fn test_system_repr_with_graph() {
        let graph_repr = GraphRepr {
            links: vec![],
            nodes: vec![InternalNodeRepr {
                name: NodeId(0),
                node: NodeRepr {
                    typ: "TestNode".to_string(),
                    props: HashMap::new(),
                },
                on_error: OnError::Fail,
            }],
        };

        let mut system_repr = SystemRepr {
            graphs: HashMap::new(),
            requests: vec![],
        };
        system_repr.graphs.insert(GraphId(1), graph_repr);

        assert_eq!(system_repr.graphs.len(), 1);
        assert!(system_repr.graphs.contains_key(&GraphId(1)));
    }

    #[test]
    fn test_system_repr_from_json() {
        let json = r#"{
            "graphs": {
                "abc": {
                    "nodes": [],
                    "links": []
                }
            },
            "requests": []
        }"#;

        let result = SystemRepr::from_json(json);
        assert!(result.is_ok());

        let system_repr = result.unwrap();
        assert_eq!(system_repr.graphs.len(), 1);
    }

    #[test]
    fn test_system_repr_from_json_invalid() {
        let json = r#"{ invalid json }"#;
        let result = SystemRepr::from_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_system_link_creation() {
        let link = SystemLink {
            src: (GraphId(1), NodeId(2), ParamId(3)),
            dst: (GraphId(4), NodeId(5), ParamId(6)),
            kind: LinkKind::Wait,
        };

        assert_eq!(link.src, (GraphId(1), NodeId(2), ParamId(3)));
        assert_eq!(link.dst, (GraphId(4), NodeId(5), ParamId(6)));
        assert!(matches!(link.kind, LinkKind::Wait));
    }

    #[test]
    fn test_link_kind_variants() {
        let wait = LinkKind::Wait;
        let continue_ = LinkKind::Continue;

        assert!(matches!(wait, LinkKind::Wait));
        assert!(matches!(continue_, LinkKind::Continue));
    }

    #[test]
    fn test_link_kind_equality() {
        assert_eq!(LinkKind::Wait, LinkKind::Wait);
        assert_eq!(LinkKind::Continue, LinkKind::Continue);
        assert_ne!(LinkKind::Wait, LinkKind::Continue);
    }

    #[test]
    fn test_link_kind_ordering() {
        assert!(LinkKind::Wait < LinkKind::Continue);
        assert!(LinkKind::Continue > LinkKind::Wait);
    }

    #[test]
    fn test_link_kind_clone() {
        let kind = LinkKind::Wait;
        let cloned = kind.clone();
        assert_eq!(kind, cloned);
    }

    #[test]
    fn test_link_kind_default() {
        let default = LinkKind::wait();
        assert!(matches!(default, LinkKind::Wait));
    }

    #[test]
    fn test_graph_repr_debug() {
        let graph_repr = GraphRepr {
            links: vec![],
            nodes: vec![],
        };

        let debug_str = format!("{:?}", graph_repr);
        assert!(debug_str.contains("GraphRepr"));
    }

    #[test]
    fn test_node_repr_debug() {
        let node_repr = NodeRepr {
            typ: "Test".to_string(),
            props: HashMap::new(),
        };

        let debug_str = format!("{:?}", node_repr);
        assert!(debug_str.contains("Test"));
    }

    #[test]
    fn test_system_link_with_props() {
        let mut props = HashMap::new();
        props.insert(ParamId(0), Value::load(&42i32).unwrap());
        props.insert(ParamId(1), Value::load(&"test").unwrap());

        let node_repr = NodeRepr {
            typ: "NodeWithProps".to_string(),
            props,
        };

        assert_eq!(node_repr.props.len(), 2);
        assert!(node_repr.props.contains_key(&ParamId(0)));
        assert!(node_repr.props.contains_key(&ParamId(1)));
    }

    #[test]
    fn test_system_link_deserialization() {
        let json = r#"{
            "src": ["1", "2", "3"],
            "dst": ["4", "5", "6"],
            "kind": "Wait"
        }"#;

        let result: Result<SystemLink, _> = serde_json::from_str(json);
        assert!(result.is_ok());

        let link = result.unwrap();
        assert_eq!(link.src.0, GraphId(1));
        assert_eq!(link.src.1, NodeId(2));
        assert_eq!(link.src.2, ParamId(3));
        assert!(matches!(link.kind, LinkKind::Wait));
    }

    #[test]
    fn test_system_link_default_kind() {
        let json = r#"{
            "src": ["1", "2", "3"],
            "dst": ["4", "5", "6"]
        }"#;

        let result: Result<SystemLink, _> = serde_json::from_str(json);
        assert!(result.is_ok());

        let link = result.unwrap();
        // Default kind should be Wait
        assert!(matches!(link.kind, LinkKind::Wait));
    }

    #[test]
    fn test_complex_system_repr() {
        let json = r#"{
            "graphs": {
                "0": {
                    "nodes": [
                        {
                            "name": "node1",
                            "type": "TestNode1",
                            "on_error": "Skip"
                        },
                        {
                            "name": "node2",
                            "type": "TestNode2",
                            "on_error": "Continue"
                        }
                    ],
                    "links": [
                        {
                            "src": ["node1", "0"],
                            "dst": ["node2", "0"]
                        }
                    ]
                }
            },
            "requests": [
                {
                    "src": ["0", "node1", "0"],
                    "dst": ["1", "node3", "0"],
                    "kind": "Continue"
                }
            ]
        }"#;

        let result = SystemRepr::from_json(json);
        assert!(result.is_ok());

        let system_repr = result.unwrap();
        assert_eq!(system_repr.graphs.len(), 1);
        assert_eq!(system_repr.requests.len(), 1);
    }
}
