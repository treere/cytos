use serde::Deserialize;

use crate::{architecture::Graph as G, transformer::Loader};

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
