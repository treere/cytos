use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug)]
pub enum Data {
    None,
    U8(u8),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Path {
    node: u64,
    field: u64,
}

impl Path {
    pub fn new(node: u64, field: u64) -> Self {
        Path { node, field }
    }
}

struct Processor {
    name: u64,
    fun: Box<dyn Transformer>,
}

impl Processor {
    fn new(name: u64, fun: impl Transformer + 'static) -> Self {
        Self {
            name,
            fun: Box::new(fun),
        }
    }

    fn inputs(&self) -> Vec<Path> {
        self.fun
            .inputs()
            .iter()
            .map(|x| Path::new(self.name, *x))
            .collect()
    }

    fn outputs(&self) -> Vec<Path> {
        self.fun
            .outputs()
            .iter()
            .map(|x| Path::new(self.name, *x))
            .collect()
    }

    fn process(
        &self,
        inputs: &HashMap<u64, Rc<RefCell<Data>>>,
        outputs: &mut HashMap<u64, Rc<RefCell<Data>>>,
    ) -> Result<(), ()> {
        self.fun.process(inputs, outputs)
    }
}

pub struct Orchestrator {
    nodes: Vec<Processor>,

    outputs: HashMap<u64, HashMap<u64, Rc<RefCell<Data>>>>,
    inputs: HashMap<u64, HashMap<u64, Rc<RefCell<Data>>>>,
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),

            outputs: HashMap::new(),
            inputs: HashMap::new(),
        }
    }

    pub fn add(mut self, name: u64, processor: impl Transformer + 'static) -> Result<Self, ()> {
        if !self.nodes.iter().any(|n| n.name == name) {
            self.outputs.insert(
                name,
                HashMap::from_iter(
                    processor
                        .outputs()
                        .iter()
                        .map(|n| (*n, Rc::new(RefCell::new(Data::None)))),
                ),
            );

            self.nodes.push(Processor::new(name, processor));

            Ok(self)
        } else {
            Err(())
        }
    }

    pub fn connect(mut self, src: Path, dst: Path) -> Result<Self, ()> {
        if let Some(inp) = self.nodes.iter().find(|n| n.name == dst.node) {
            if !inp.inputs().contains(&dst) {
                return Err(());
            }
            inp
        } else {
            return Err(());
        };

        if let Some(outp) = self.nodes.iter_mut().find(|n| n.name == src.node) {
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
            .and_then(|node| node.get(&src.field))
            .ok_or(())?;

        self.inputs
            .entry(dst.node)
            .or_default()
            .insert(dst.field, output.clone());

        Ok(self)
    }

    pub fn step(&mut self) -> Result<(), ()> {
        for node in self.nodes.iter_mut() {
            node.process(
                self.inputs.get(&node.name).unwrap_or(&HashMap::new()),
                self.outputs
                    .get_mut(&node.name)
                    .unwrap_or(&mut HashMap::new()),
            )?;
        }

        Ok(())
    }
}

pub trait Transformer {
    fn inputs(&self) -> &[u64];
    fn outputs(&self) -> &[u64];

    fn process(
        &self,
        inputs: &HashMap<u64, Rc<RefCell<Data>>>,
        outputs: &mut HashMap<u64, Rc<RefCell<Data>>>,
    ) -> Result<(), ()>;
}
