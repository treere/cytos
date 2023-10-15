use std::ops::Deref;

use super::{Done, NodeId, Path, Result, Stepper, Transformer, Value};

/// A wrapper around a [`Transformer`] keeping trace of the node id.
pub struct Processor {
    /// Node identifier.
    id: NodeId,

    /// Wrapped transformer.
    transformer: Box<dyn Transformer>,
}

impl Processor {
    /// Create a new Processor.
    pub fn new(id: NodeId, transformer: Box<dyn Transformer>) -> Self {
        Self { id, transformer }
    }
}

impl Stepper for Processor {
    fn initialize(&mut self) -> Done {
        self.transformer.initialize()
    }

    fn step(&mut self) -> Done {
        self.transformer.step()
    }
}

#[derive(Default)]
/// Graph.
pub struct Graph {
    /// Processors
    nodes: Vec<Processor>,
}

impl Graph {
    /// Add a processor with a given id to the graph.
    pub fn insert(mut self, processor: Processor) -> Result<Self> {
        if self.nodes.iter().all(|x| x.id != processor.id) {
            self.nodes.push(processor);

            Ok(self)
        } else {
            Err("already exist")
        }
    }

    /// Connects a output data to an input one.
    pub fn connect(mut self, src: Path, dst: Path) -> Result<Self> {
        let output = self
            .nodes
            .iter()
            .find(|p| p.id == src.0)
            .ok_or("cannot find source")
            .and_then(|s| s.transformer.output(src.1).ok_or("cannot find param"))?;

        self.nodes
            .iter_mut()
            .find(|p| p.id == dst.0)
            .ok_or("cannot find dest")
            .and_then(|d| d.transformer.link(dst.1, output))?;

        Ok(self)
    }

    pub fn load(mut self, src: Path, value: Value) -> Result<Self> {
        self.nodes
            .iter_mut()
            .find(|p| p.id == src.0)
            .ok_or("cannot find node")
            .and_then(|d| d.transformer.load(src.1, value))?;

        Ok(self)
    }

    pub fn dump(&self, src: Path) -> Result<String> {
        self.nodes
            .iter()
            .find(|p| p.id == src.0)
            .ok_or("cannot find node")
            .and_then(|d| d.transformer.dump(src.1))
    }

    /// Initialize the nodes
    pub fn initialize(&mut self) -> Done {
        for node in self.nodes.iter_mut() {
            node.initialize()?;
        }
        Ok(())
    }

    /// Compute one step of processing
    pub fn step(&mut self) -> Done {
        for node in self.nodes.iter_mut() {
            node.step()?;
        }

        Ok(())
    }

    pub fn param_value(&self, node: NodeId) -> Option<&dyn Transformer> {
        self.nodes
            .iter()
            .find(|x| x.id == node)
            .map(|p| p.transformer.deref())
    }
}
