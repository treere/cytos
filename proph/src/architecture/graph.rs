use std::{cmp::Ordering, collections::HashMap, ops::Deref};

use super::{props::are_linked, NodeId, Path, Stepper, Transformer};

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
    fn initialize(&mut self) -> Result<(), &'static str> {
        self.transformer.initialize()
    }

    fn step(&mut self) -> Result<(), &'static str> {
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
    pub fn insert(mut self, processor: Processor) -> Result<Self, &'static str> {
        if self.nodes.iter().all(|x| x.id != processor.id) {
            self.nodes.push(processor);

            Ok(self)
        } else {
            Err("already exist")
        }
    }

    /// Connects a output data to an input one.
    pub fn connect(mut self, src: Path, dst: Path) -> Result<Self, &'static str> {
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

        self.order_nodes();

        Ok(self)
    }

    /// Initialize the nodes
    pub fn initialize(&mut self) -> Result<(), &'static str> {
        for node in self.nodes.iter_mut() {
            node.initialize()?;
        }
        Ok(())
    }

    /// Compute one step of processing
    pub fn step(&mut self) -> Result<(), &'static str> {
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

    /// Reorder nodes so there cannot a required node after.
    fn order_nodes(&mut self) {
        let links = self.find_links();

        let mut p = HashMap::new();
        for (s, d) in links.iter() {
            p.entry(d.0.clone()).or_insert(Vec::new()).push(s.0.clone())
        }
        self.nodes.sort_unstable_by(|s, d| {
            if p.get(&d.id).map(|v| v.contains(&s.id)).unwrap_or(false) {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        });
        self.nodes.reverse();
    }

    /// Find links between nodes
    fn find_links(&self) -> Vec<(Path, Path)> {
        let inputs: Vec<_> = self
            .nodes
            .iter()
            .flat_map(|n| {
                n.transformer
                    .input_names()
                    .iter()
                    .map(|p| {
                        (
                            (n.id.clone(), p.clone()),
                            n.transformer.input(p.clone()).unwrap(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        let outputs: Vec<_> = self
            .nodes
            .iter()
            .flat_map(|n| {
                n.transformer
                    .output_names()
                    .iter()
                    .map(|p| {
                        (
                            (n.id.clone(), p.clone()),
                            n.transformer.output(p.clone()).unwrap(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        let mut v = vec![];
        for i in inputs.iter() {
            for j in outputs.iter() {
                if are_linked(&i.1, &j.1) {
                    v.push((j.0.clone(), i.0.clone()));
                }
            }
        }
        v
    }
}
