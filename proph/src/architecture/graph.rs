use super::{Done, NodeId, ParamId, Result, Stepper, Transformer, Value};

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

    fn terminate(&mut self) -> Done {
        self.transformer.terminate()
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
    pub fn connect(&mut self, src: (&NodeId, &ParamId), dst: (&NodeId, &ParamId)) -> Done {
        let output = self
            .nodes
            .iter()
            .find(|p| p.id == *src.0)
            .ok_or("cannot find source")?
            .transformer
            .output(src.1)
            .ok_or("cannot find param")?;

        self.nodes
            .iter_mut()
            .find(|p| p.id == *dst.0)
            .ok_or("cannot find dest")?
            .transformer
            .link(dst.1, output)?;

        Ok(())
    }

    pub fn load(&mut self, src: (&NodeId, &ParamId), value: Value) -> Done {
        self.nodes
            .iter_mut()
            .find(|p| p.id == *src.0)
            .ok_or("cannot find node")?
            .transformer
            .load(src.1, value)?;

        Ok(())
    }

    pub fn dump(&self, src: (&NodeId, &ParamId)) -> Result<Value> {
        self.nodes
            .iter()
            .find(|p| p.id == *src.0)
            .ok_or("cannot find node")?
            .transformer
            .dump(src.1)
    }

    /// Initialize the nodes
    pub fn initialize(&mut self) -> Done {
        for node in self.nodes.iter_mut() {
            node.initialize()?;
        }
        Ok(())
    }

    /// Terminate the nodes
    pub fn terminate(&mut self) -> Done {
        for node in self.nodes.iter_mut() {
            node.terminate()?;
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

    /// List nodes
    pub fn list_nodes(&self) -> Vec<String> {
        self.nodes.iter().map(|x| x.id.clone()).collect()
    }

    /// List node inputs
    pub fn list_node_inputs(&self, node: &NodeId) -> Result<Vec<String>> {
        self.nodes
            .iter()
            .find(|n| n.id == *node)
            .map(|n| n.transformer.input_names())
            .ok_or("missing node")
    }

    /// List node outputs
    pub fn list_node_outputs(&self, node: &NodeId) -> Result<Vec<String>> {
        self.nodes
            .iter()
            .find(|n| n.id == *node)
            .map(|n| n.transformer.output_names())
            .ok_or("missing node")
    }
}

unsafe impl Send for Graph {}
