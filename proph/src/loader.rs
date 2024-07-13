use crate::architecture::{
    graph::{Graph, Processor},
    repr::{GraphRepr, Link, LinkSource, ProcessorRepr},
    GraphId, NodeId, ParamId, Result, Transformer,
};

use libloading::{Library, Symbol};

use std::{collections::HashMap, sync::Arc};

impl GraphRepr {
    pub fn id(&self) -> Result<GraphId> {
        GraphId::try_from(self.name.as_str())
    }

    pub fn build(self, loader: &Registry) -> Result<Graph> {
        let mut graph = Graph::default();
        for node in self.nodes {
            let (id, processor) = node.build(loader)?;
            graph = graph.insert(id, processor)?;
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
                LinkSource::ExternalLinkSource(_g0, _s0, _s1) => {
                    // let g0 = GraphId::try_from(&g0)?;
                    // let s0 = NodeId::try_from(&s0)?;
                    // let s1 = ParamId::try_from(&s1)?;

                    // graph.external_link((g0, s0, s1), (d0, d1))?;
                    todo!();
                }
            }
        }
        Ok(graph)
    }
}

impl ProcessorRepr {
    fn build(self, loader: &Registry) -> Result<(NodeId, Processor)> {
        let mut transformer = loader.load(self.typ.as_str())?;
        let nodeid = NodeId::try_from(self.name.as_str())?;
        for (prop, value) in self.props {
            let propid = ParamId::try_from(&prop)?;
            transformer.load(propid, value)?;
        }

        Ok((nodeid, Processor::new(transformer)))
    }
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
