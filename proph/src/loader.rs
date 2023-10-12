use crate::architecture::Graph;
use crate::architecture::{Processor, Transformer};

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct GraphRepr {
    nodes: Vec<Node>,

    links: Vec<Link>,
}

impl GraphRepr {
    pub fn load(file: &str, loader: &Loader) -> Result<Graph, &'static str> {
        let repr: Self = serde_json::from_str(file).map_err(|_| "cannot load file")?;

        let mut graph = Graph::default();
        for node in repr.nodes.iter() {
            graph = graph.insert(loader.load(node.name.as_str(), node.typ.as_str())?)?;
        }

        for link in repr.links.into_iter() {
            graph = graph.connect(link.src, link.dst)?;
        }
        Ok(graph)
    }
}

#[derive(Deserialize, Debug)]
struct Node {
    name: String,

    #[serde(rename = "type")]
    typ: String,
}

#[derive(Deserialize, Debug)]
struct Link {
    src: (String, String),
    dst: (String, String),
}

#[derive(Default)]
pub struct Loader {
    transformers: HashMap<String, Box<dyn Fn() -> Box<dyn Transformer>>>,
}

impl Loader {
    pub fn add<K: Transformer + 'static>(
        mut self,
        name: impl AsRef<str>,
        f: impl (Fn() -> K) + 'static,
    ) -> Self {
        self.transformers
            .entry(name.as_ref().to_owned())
            .or_insert(Box::new(move || Box::new(f())));
        self
    }

    pub fn load(&self, name: &str, typ: &str) -> Result<Processor, &'static str> {
        let factory = self.transformers.get(typ).ok_or("missing type")?;
        Ok(Processor::load(name.to_owned(), factory()))
    }
}
