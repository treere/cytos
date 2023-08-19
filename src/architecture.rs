use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub type NodeId = u64;
pub type ParamId = u64;

#[derive(Debug)]
pub enum Data {
    None,
    U8(u8),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Path {
    node: NodeId,
    param: ParamId,
}

impl Path {
    pub fn new(node: NodeId, param: ParamId) -> Self {
        Path { node, param }
    }
}

struct Processor {
    id: NodeId,
    fun: Box<dyn Transformer>,
}

impl Processor {
    fn new(id: NodeId, fun: impl Transformer + 'static) -> Self {
        Self {
            id,
            fun: Box::new(fun),
        }
    }

    fn inputs(&self) -> Vec<Path> {
        self.fun
            .inputs()
            .iter()
            .map(|x| Path::new(self.id, *x))
            .collect()
    }

    fn outputs(&self) -> Vec<Path> {
        self.fun
            .outputs()
            .iter()
            .map(|x| Path::new(self.id, *x))
            .collect()
    }

    fn process(
        &self,
        inputs: &HashMap<NodeId, Rc<RefCell<Data>>>,
        outputs: &mut HashMap<NodeId, Rc<RefCell<Data>>>,
    ) -> Result<(), ()> {
        self.fun.process(inputs, outputs)
    }
}

pub struct Orchestrator {
    nodes: Vec<Processor>,
    outputs: HashMap<NodeId, HashMap<ParamId, Rc<RefCell<Data>>>>,
    inputs: HashMap<NodeId, HashMap<ParamId, Rc<RefCell<Data>>>>,
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            outputs: HashMap::new(),
            inputs: HashMap::new(),
        }
    }

    pub fn add(mut self, id: NodeId, processor: impl Transformer + 'static) -> Result<Self, ()> {
        if !self.nodes.iter().any(|n| n.id == id) {
            self.outputs.insert(
                id,
                HashMap::from_iter(
                    processor
                        .outputs()
                        .iter()
                        .map(|n| (*n, Rc::new(RefCell::new(Data::None)))),
                ),
            );

            self.nodes.push(Processor::new(id, processor));

            Ok(self)
        } else {
            Err(())
        }
    }

    pub fn connect(mut self, src: Path, dst: Path) -> Result<Self, ()> {
        if let Some(inp) = self.nodes.iter().find(|n| n.id == dst.node) {
            if !inp.inputs().contains(&dst) {
                return Err(());
            }
            inp
        } else {
            return Err(());
        };

        if let Some(outp) = self.nodes.iter_mut().find(|n| n.id == src.node) {
            if !outp.outputs().contains(&src) {
                return Err(());
            }
            outp
        } else {
            return Err(());
        };

        let output = self
            .outputs
            .get(&src.node)
            .and_then(|node| node.get(&src.param))
            .ok_or(())?;

        self.inputs
            .entry(dst.node)
            .or_default()
            .insert(dst.param, output.clone());

        Ok(self)
    }

    pub fn step(&mut self) -> Result<(), ()> {
        for node in self.nodes.iter_mut() {
            node.process(
                self.inputs.get(&node.id).unwrap_or(&HashMap::new()),
                self.outputs
                    .get_mut(&node.id)
                    .unwrap_or(&mut HashMap::new()),
            )?;
        }

        Ok(())
    }
}

pub trait Transformer {
    fn inputs(&self) -> &[ParamId];
    fn outputs(&self) -> &[ParamId];

    fn process(
        &self,
        inputs: &HashMap<ParamId, Rc<RefCell<Data>>>,
        outputs: &mut HashMap<ParamId, Rc<RefCell<Data>>>,
    ) -> Result<(), ()>;
}
