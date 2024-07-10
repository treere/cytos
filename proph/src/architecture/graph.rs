use super::{GraphId, NodeId, ParamId, Result, Stepper, Transformer, Value};

/// A wrapper around a [`Transformer`] keeping trace of the node id.
pub struct Processor {
    /// Wrapped transformer.
    transformer: Box<dyn Transformer>,
}

impl Processor {
    /// Create a new Processor.
    pub fn new(transformer: Box<dyn Transformer>) -> Self {
        Self { transformer }
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
    nodes: Vec<(NodeId, Processor)>,

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
    pub fn insert(mut self, id: NodeId, processor: Processor) -> Result<Self> {
        if self.nodes.iter().all(|x| x.0 != id) {
            self.nodes.push((id, processor));

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
            .find(|p| p.0 == src_node_id)
            .ok_or("cannot find source")?
            .1
            .transformer
            .output(src_param_id)
            .ok_or("cannot find param")?;

        self.nodes
            .iter_mut()
            .find(|p| p.0 == dst_node_id)
            .ok_or("cannot find dest")?
            .1
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
            .find(|p| p.0 == node_id)
            .ok_or("cannot find node")?
            .1
            .transformer
            .load(param_id, value)?;

        Ok(())
    }

    pub fn dumper_for(&self, (node_id, param_id): (NodeId, ParamId)) -> Result<Value> {
        self.nodes
            .iter()
            .find(|p| p.0 == node_id)
            .ok_or("cannot find node")?
            .1
            .transformer
            .dump(param_id)
    }

    /// Initialize the nodes
    pub fn initialize(&mut self) -> Result<()> {
        for node in &mut self.nodes {
            node.1.initialize()?;
        }
        Ok(())
    }

    /// Terminate the nodes
    pub fn terminate(&mut self) -> Result<()> {
        for node in &mut self.nodes {
            node.1.terminate()?;
        }
        Ok(())
    }

    /// Compute one step of processing
    pub fn step(&mut self) -> Result<()> {
        for node in &mut self.nodes {
            node.1.step()?;
        }

        Ok(())
    }

    /// List nodes
    pub fn list_nodes(&self) -> Vec<NodeId> {
        self.nodes.iter().map(|x| x.0).collect()
    }

    /// List node inputs
    pub fn list_node_inputs(&self, node: NodeId) -> Result<Vec<ParamId>> {
        self.nodes
            .iter()
            .find(|n| n.0 == node)
            .map(|n| n.1.transformer.input_names())
            .ok_or("missing node")
    }

    /// List node outputs
    pub fn list_node_outputs(&self, node: NodeId) -> Result<Vec<ParamId>> {
        self.nodes
            .iter()
            .find(|n| n.0 == node)
            .map(|n| n.1.transformer.output_names())
            .ok_or("missing node")
    }
}
