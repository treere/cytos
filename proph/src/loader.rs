use crate::architecture::{
    graph::{Graph, Processor},
    GraphId, NodeId, ParamId, Result, Transformer, Value,
};

use libloading::{Library, Symbol};
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};

#[derive(Deserialize, Debug)]
pub struct SystemRepr {
    graphs: Vec<GraphRepr>,
}

impl SystemRepr {
    pub fn from_json(file: &str) -> Result<Self> {
        serde_json::from_str(file).or(Err("cannot load file"))
    }

    pub fn build(self, loader: &Registry) -> Result<Vec<Graph>> {
        self.graphs
            .into_iter()
            .map(|x| x.build(&loader))
            .collect::<Result<Vec<_>>>()
    }
}

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
        let mut graph = Graph::new(GraphId::try_from(self.name.as_str())?);
        for node in self.nodes {
            let processor = node.build(loader)?;
            graph = graph.insert(processor)?;
        }

        for Link { src, dst: (d0, d1) } in self.links {
            let d0 = NodeId::try_from(&d0)?;
            let d1 = ParamId::try_from(&d1)?;

            match src {
                LinkSource::InternalLinkSource(s0, s1) => {
                    let s0 = NodeId::try_from(&s0)?;
                    let s1 = ParamId::try_from(&s1)?;

                    graph.internal_link((s0, s1), (d0, d1))?;
                }
                LinkSource::ExternalLinkSource(g0, s0, s1) => {
                    let g0 = GraphId::try_from(&g0)?;
                    let s0 = NodeId::try_from(&s0)?;
                    let s1 = ParamId::try_from(&s1)?;

                    println!("{g0} {s0} {s1}");
                    todo!("I need a way to take the command of the other graph");
                }
            }
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

impl Node {
    fn build(self, loader: &Registry) -> Result<Processor> {
        let mut transformer = loader.load(self.typ.as_str())?;
        let nodeid = NodeId::try_from(self.name.as_str())?;
        for (prop, value) in self.props {
            let propid = ParamId::try_from(&prop)?;
            transformer.load(propid, value)?;
        }

        Ok(Processor::new(nodeid, transformer))
    }
}

#[derive(Deserialize, Debug)]
enum LinkSource {
    InternalLinkSource(String, String),
    ExternalLinkSource(String, String, String),
}

/// Link between nodes
#[derive(Deserialize, Debug)]
struct Link {
    /// Source node param
    src: LinkSource,

    /// Destination node param
    dst: (String, String),
}

type Factory = Arc<dyn Fn() -> Box<dyn Transformer> + Send + Sync>;

/// Registry of transformers
#[derive(Default, Clone)]
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
        factory: impl (Fn() -> K) + 'static + Send + Sync,
    ) -> &mut Self {
        self.factories
            .entry(name.as_ref().to_owned())
            .or_insert(Arc::new(move || Box::new(factory())));
        self
    }

    /// Load a Processor
    pub fn load(&self, typ: &str) -> Result<Box<dyn Transformer>> {
        let factory = self.factories.get(typ).ok_or("missing type")?;
        Ok(factory())
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
