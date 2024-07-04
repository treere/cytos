use crate::{
    architecture::{
        graph::{Graph, Processor},
        Result, Transformer, Value,
    },
    utils::{string_to_graphid, string_to_nodeid, string_to_paramid},
};

use libloading::{Library, Symbol};
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};

/// Graph representatio to be loaded
#[derive(Deserialize, Debug)]
pub struct GraphRepr {
    /// Graph name
    name: String,

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
        let mut graph = Graph::new(string_to_graphid(&self.name)?);
        for node in self.nodes {
            let processor = loader.load(node.name.as_str(), node.typ.as_str())?;
            graph = graph.insert(processor)?;
            let nodeid = string_to_nodeid(&node.name)?;

            for (prop, value) in node.props {
                let propid = string_to_paramid(&prop)?;
                graph.load((nodeid, propid), value)?;
            }
        }

        for Link {
            src: (s0, s1),
            dst: (d0, d1),
        } in self.links
        {
            let s0 = string_to_nodeid(&s0)?;
            let s1 = string_to_paramid(&s1)?;
            let d0 = string_to_nodeid(&d0)?;
            let d1 = string_to_paramid(&d1)?;

            graph.connect((s0, s1), (d0, d1))?;
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

    /// Libs
    libs: Vec<Arc<Library>>,
}

impl Registry {
    /// Add a factory
    pub fn add<K: Transformer + 'static>(
        &mut self,
        name: impl AsRef<str>,
        factory: impl (Fn() -> K) + 'static + Send,
    ) -> &mut Self {
        self.factories
            .entry(name.as_ref().to_owned())
            .or_insert(Box::new(move || Box::new(factory())));
        self
    }

    /// Load a Processor
    pub fn load(&self, name: &str, typ: &str) -> Result<Processor> {
        let factory = self.factories.get(typ).ok_or("missing type")?;
        let name = string_to_nodeid(name)?;
        Ok(Processor::new(name, factory()))
    }

    pub fn list_factories(&self) -> impl Iterator<Item = &String> {
        self.factories.keys()
    }

    pub fn load_library(&mut self, file: &str) -> Result<()> {
        let lib = unsafe {
            Library::new(libloading::library_filename(file)).or(Err("cannot load library"))?
        };
        let lib = Arc::new(lib);

        let load_registry: Symbol<fn(&mut Registry) -> ()> = unsafe {
            lib.get(b"load_registry")
                .or(Err("missing load_registry function"))?
        };

        load_registry(self);
        self.libs.push(lib);
        Ok(())
    }
}
