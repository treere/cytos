use super::{GraphId, NodeId, ParamId, Result, Stepper, Transformer, Value};

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
    fn initialize(&mut self) -> Result<()> {
        self.transformer.initialize()
    }

    fn step(&mut self) -> Result<()> {
        self.transformer.step()
    }

    fn terminate(&mut self) -> Result<()> {
        self.transformer.terminate()
    }
}

/// Graph.
pub struct Graph {
    /// Name
    pub id: GraphId,

    /// Processors
    nodes: Vec<Processor>,

    /// External links
    external: Vec<((GraphId, NodeId, ParamId), (NodeId, ParamId))>,
}

impl Graph {
    /// Create a graph
    pub fn new(id: GraphId) -> Self {
        Self {
            id,
            nodes: Vec::default(),
            external: Vec::default(),
        }
    }

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
    pub fn internal_link(
        &mut self,
        (src_node_id, src_param_id): (NodeId, ParamId),
        (dst_node_id, dst_param_id): (NodeId, ParamId),
    ) -> Result<()> {
        let output = self
            .nodes
            .iter()
            .find(|p| p.id == src_node_id)
            .ok_or("cannot find source")?
            .transformer
            .output(src_param_id)
            .ok_or("cannot find param")?;

        self.nodes
            .iter_mut()
            .find(|p| p.id == dst_node_id)
            .ok_or("cannot find dest")?
            .transformer
            .link(dst_param_id, output)?;

        Ok(())
    }

    pub fn external_link(
        &mut self,
        src: (GraphId, NodeId, ParamId),
        dst: (NodeId, ParamId),
    ) -> Result<()> {
        self.external.push((src, dst));
        Ok(())
    }

    pub fn load(&mut self, (node_id, param_id): (NodeId, ParamId), value: Value) -> Result<()> {
        self.nodes
            .iter_mut()
            .find(|p| p.id == node_id)
            .ok_or("cannot find node")?
            .transformer
            .load(param_id, value)?;

        Ok(())
    }

    pub fn dumper_for(&self, (node_id, param_id): (NodeId, ParamId)) -> Result<Value> {
        self.nodes
            .iter()
            .find(|p| p.id == node_id)
            .ok_or("cannot find node")?
            .transformer
            .dump(param_id)
    }

    /// Initialize the nodes
    pub fn initialize(&mut self) -> Result<()> {
        for node in &mut self.nodes {
            node.initialize()?;
        }
        Ok(())
    }

    /// Terminate the nodes
    pub fn terminate(&mut self) -> Result<()> {
        for node in &mut self.nodes {
            node.terminate()?;
        }
        Ok(())
    }

    /// Compute one step of processing
    pub fn step(&mut self) -> Result<()> {
        for node in &mut self.nodes {
            node.step()?;
        }

        Ok(())
    }

    /// List nodes
    pub fn list_nodes(&self) -> Vec<NodeId> {
        self.nodes.iter().map(|x| x.id).collect()
    }

    /// List node inputs
    pub fn list_node_inputs(&self, node: NodeId) -> Result<Vec<ParamId>> {
        self.nodes
            .iter()
            .find(|n| n.id == node)
            .map(|n| n.transformer.input_names())
            .ok_or("missing node")
    }

    /// List node outputs
    pub fn list_node_outputs(&self, node: NodeId) -> Result<Vec<ParamId>> {
        self.nodes
            .iter()
            .find(|n| n.id == node)
            .map(|n| n.transformer.output_names())
            .ok_or("missing node")
    }
}
