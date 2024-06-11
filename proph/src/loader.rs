use crate::architecture::{
    graph::{Graph, Processor},
    Result, Transformer, Value,
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
    pub fn from_json(file: &str) -> Result<Self> {
        serde_json::from_str(file).or(Err("cannot load file"))
    }

    pub fn build(self, loader: &Registry) -> Result<Graph> {
        let mut graph = Graph::default();
        for node in self.nodes.into_iter() {
            let processor = loader.load(node.name.as_str(), node.typ.as_str())?;
            graph = graph.insert(processor)?;

            for (prop, value) in node.props.into_iter() {
                graph.load((&node.name, &prop), value)?;
            }
        }

        for Link {
            src: (s0, s1),
            dst: (d0, d1),
        } in self.links.into_iter()
        {
            graph.connect((&s0, &s1), (&d0, &d1))?;
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
    props: HashMap<String, Value>,
}

/// Link between nodes
#[derive(Deserialize, Debug)]
struct Link {
    /// Source node param
    src: (String, String),

    /// Destination node param
    dst: (String, String),
}

type Factory = Box<dyn Fn() -> Box<dyn Transformer> + Send>;

/// Registry of transformers
#[derive(Default)]
pub struct Registry {
    /// Factories
    factories: HashMap<String, Factory>,
}

impl Registry {
    /// Add a factory
    pub fn add<K: Transformer + 'static>(
        mut self,
        name: impl AsRef<str>,
        factory: impl (Fn() -> K) + 'static + Send,
    ) -> Self {
        self.factories
            .entry(name.as_ref().to_owned())
            .or_insert(Box::new(move || Box::new(factory())));
        self
    }

    /// Load a Processor
    pub fn load(&self, name: &str, typ: &str) -> Result<Processor> {
        let factory = self.factories.get(typ).ok_or("missing type")?;
        Ok(Processor::new(name.to_owned(), factory()))
    }
}
