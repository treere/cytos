use crate::architecture::Graph as G;
use crate::architecture::{Processor, Transformer};

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Graph {
    pub nodes: Vec<Node>,

    pub links: Vec<Link>,
}

impl Graph {
    pub fn load(file: &str, loader: &Loader) -> Result<G, &'static str> {
        let l: Self = serde_json::from_str(file).map_err(|_| "cannot load file")?;

        let mut g = G::new();
        for node in l.nodes.iter() {
            g = g.insert(loader.load(node.name.as_str(), node.typ.as_str())?)?;
        }

        for link in l.links.into_iter() {
            g = g.connect(link.src, link.dst)?;
        }
        Ok(g)
    }
}

#[derive(Deserialize, Debug)]
pub struct Node {
    pub name: String,

    #[serde(rename = "type")]
    pub typ: String,
}

#[derive(Deserialize, Debug)]
pub struct Link {
    pub src: (String, String),
    pub dst: (String, String),
}

pub struct Loader {
    transformers: HashMap<String, Box<dyn Fn() -> Box<dyn Transformer>>>,
}

impl Loader {
    pub fn new() -> Self {
        Self {
            transformers: HashMap::new(),
        }
    }
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

impl Default for Loader {
    fn default() -> Self {
        Self::new()
    }
}
