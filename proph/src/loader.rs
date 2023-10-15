use crate::architecture::{
    graph::{Graph, Processor},
    Result, Transformer,
};

use serde::Deserialize;
use std::collections::HashMap;

/// Graph representatio to be loaded
#[derive(Deserialize, Debug)]
pub struct GraphRepr {
    /// List of nodes
    nodes: Vec<Node>,

    /// List of links between nodes
    links: Vec<Link>,
}

impl GraphRepr {
    pub fn load(file: &str, loader: &Registry) -> Result<Graph> {
        let repr: Self = serde_json::from_str(file).map_err(|_| "cannot load file")?;

        let mut graph = Graph::default();
        for node in repr.nodes.into_iter() {
            let processor = loader.load(node.name.as_str(), node.typ.as_str())?;
            graph = graph.insert(processor)?;

            for (prop, value) in node.props.into_iter() {
                graph = graph.load((node.name.clone(), prop), &value)?;
            }
        }

        for link in repr.links.into_iter() {
            graph = graph.connect(link.src, link.dst)?;
        }
        Ok(graph)
    }
}

#[derive(Deserialize, Debug)]
struct Node {
    /// Name of the node
    name: String,

    /// Type of the node
    #[serde(rename = "type")]
    typ: String,

    /// Properties
    #[serde(default)]
    props: HashMap<String, String>,
}

/// Link between nodes
#[derive(Deserialize, Debug)]
struct Link {
    /// Source node param
    src: (String, String),

    /// Destination node param
    dst: (String, String),
}

/// Registry of transformers
#[derive(Default)]
pub struct Registry {
    /// Factories
    factories: HashMap<String, Box<dyn Fn() -> Box<dyn Transformer>>>,
}

impl Registry {
    /// Add a factory
    pub fn add<K: Transformer + 'static>(
        mut self,
        name: impl AsRef<str>,
        factory: impl (Fn() -> K) + 'static,
    ) -> Self {
        self.factories
            .entry(name.as_ref().to_owned())
            .or_insert(Box::new(move || Box::new(factory())));
        self
    }

    /// Load a Processor
    fn load(&self, name: &str, typ: &str) -> Result<Processor> {
        let factory = self.factories.get(typ).ok_or("missing type")?;
        Ok(Processor::new(name.to_owned(), factory()))
    }
}
