use std::{
    cell::RefCell,
    collections::HashMap,
    ops::{Deref, DerefMut},
    rc::Rc,
};

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

    fn process(&self, inputs: Params, outputs: Outputs) -> Result<(), ()> {
        self.fun.process(inputs, outputs)
    }
}

struct Communication {
    outputs: HashMap<NodeId, ParamData>,
    inputs: HashMap<NodeId, ParamData>,
}

impl Communication {
    fn new() -> Self {
        Self {
            outputs: HashMap::new(),
            inputs: HashMap::new(),
        }
    }

    fn add_output(&mut self, id: NodeId, processor: &impl Transformer) {
        self.outputs.insert(
            id,
            HashMap::from_iter(
                processor
                    .outputs()
                    .iter()
                    .map(|n| (*n, Rc::new(RefCell::new(Data::None)))),
            ),
        );

        self.inputs.insert(id, HashMap::new());
    }

    fn connect(&mut self, src: Path, dst: Path) -> Result<(), ()> {
        let output = self
            .outputs
            .get(&src.node)
            .and_then(|node| node.get(&src.param))
            .ok_or(())?;

        self.inputs
            .entry(dst.node)
            .or_default()
            .insert(dst.param, output.clone());

        Ok(())
    }

    fn get(&mut self, id: NodeId) -> (Params, Outputs) {
        let inputs = self.inputs.get(&id).unwrap();
        let outputs = self.outputs.get_mut(&id).unwrap();
        (Params { map: inputs }, Outputs { map: outputs })
    }
}

type ParamData = HashMap<ParamId, Rc<RefCell<Data>>>;

pub struct Params<'a> {
    map: &'a ParamData,
}

impl<'a> Params<'a> {
    pub fn get(&self, val: &ParamId) -> impl Deref<Target = Data> + 'a {
        self.map.get(val).unwrap().borrow()
    }
}

pub struct Outputs<'a> {
    map: &'a mut ParamData,
}

impl<'a> Outputs<'a> {
    pub fn get_mut<'b>(&'b mut self, val: &'b ParamId) -> impl DerefMut<Target = Data> + 'b {
        let p = self.map.get_mut(val).unwrap();
        (**p).borrow_mut()
    }
}

pub struct Orchestrator {
    nodes: Vec<Processor>,
    communication: Communication,
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            communication: Communication::new(),
        }
    }

    pub fn add(mut self, id: NodeId, processor: impl Transformer + 'static) -> Result<Self, ()> {
        if !self.nodes.iter().any(|n| n.id == id) {
            self.communication.add_output(id, &processor);
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

        self.communication.connect(src, dst)?;

        Ok(self)
    }

    pub fn step(&mut self) -> Result<(), ()> {
        for node in self.nodes.iter_mut() {
            let (inputs, outputs) = self.communication.get(node.id);
            node.process(inputs, outputs)?;
        }

        Ok(())
    }
}

pub trait Transformer {
    fn inputs(&self) -> &[ParamId];
    fn outputs(&self) -> &[ParamId];

    fn process(&self, inputs: Params, outputs: Outputs) -> Result<(), ()>;
}
