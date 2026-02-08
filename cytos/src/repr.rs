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

    /// Continue processing the fraph
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
