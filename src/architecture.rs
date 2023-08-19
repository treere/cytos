use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use crate::data::Data;

pub type NodeId = u32;
pub type ParamId = u32;

#[derive(Debug, PartialEq, Eq)]
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

    fn process(&mut self, inputs: Params, outputs: Outputs) -> Result<(), ()> {
        self.fun.process(inputs, outputs)
    }
}

struct Communication {
    outputs: Vec<(NodeId, ParamData)>,
    inputs: Vec<(NodeId, ParamData)>,
}

impl Communication {
    fn new() -> Self {
        Self {
            outputs: Vec::new(),
            inputs: Vec::new(),
        }
    }

    fn add_output(&mut self, id: NodeId, processor: &impl Transformer) {
        self.outputs.push((
            id,
            processor
                .outputs()
                .iter()
                .map(|n| (*n, Rc::new(RefCell::new(Data::None))))
                .collect(),
        ));

        self.inputs.push((id, Vec::new()));
    }

    fn connect(&mut self, src: Path, dst: Path) -> Result<(), ()> {
        let output = self
            .outputs
            .iter()
            .find(|(o, _)| o == &src.node)
            .and_then(|(_, node)| node.iter().find(|(o, _)| o == &src.param))
            .map(|(_, p)| p)
            .ok_or(())?;

        self.inputs
            .iter_mut()
            .find(|(o, _)| o == &dst.node)
            .map(|(_, node)| node.push((dst.param, output.clone())))
            .ok_or(())
    }

    fn get_arguments(&mut self, id: NodeId) -> (Params, Outputs) {
        let (_, inputs) = self.inputs.iter().find(|(o, _)| *o == id).unwrap();
        let (_, outputs) = self.outputs.iter_mut().find(|(o, _)| *o == id).unwrap();
        (Params { map: inputs }, Outputs { map: outputs })
    }

    fn get_outputs(&self, id: NodeId) -> Params {
        self.outputs
            .iter()
            .find(|(o, _)| *o == id)
            .map(|(_, p)| Params { map: p })
            .unwrap()
    }
}

type ParamData = Vec<(ParamId, Rc<RefCell<Data>>)>;

pub struct Params<'a> {
    map: &'a ParamData,
}

impl<'a> Params<'a> {
    pub fn get(&self, val: &ParamId) -> impl Deref<Target = Data> + 'a {
        self.map
            .iter()
            .find(|(o, _)| o == val)
            .map(|(_, p)| p)
            .unwrap()
            .borrow()
    }
}

pub struct Outputs<'a> {
    map: &'a mut ParamData,
}

impl<'a> Outputs<'a> {
    pub fn get_mut<'b>(&'b mut self, val: &'b ParamId) -> impl DerefMut<Target = Data> + 'b {
        self.map
            .iter_mut()
            .find(|(o, _)| o == val)
            .map(|(_, p)| (**p).borrow_mut())
            .unwrap()
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
        self.nodes
            .iter()
            .find(|n| n.id == dst.node)
            .and_then(|inp| inp.inputs().contains(&dst).then_some(()))
            .ok_or(())?;

        self.communication.connect(src, dst)?;

        Ok(self)
    }

    pub fn step(&mut self) -> Result<(), ()> {
        for node in self.nodes.iter_mut() {
            let (inputs, outputs) = self.communication.get_arguments(node.id);
            node.process(inputs, outputs)?;
        }

        Ok(())
    }

    pub fn value(&mut self, node: NodeId, param: ParamId) -> impl Deref<Target = Data> + '_ {
        self.communication.get_outputs(node).get(&param)
    }
}

pub trait Transformer {
    fn inputs(&self) -> &[ParamId];
    fn outputs(&self) -> &[ParamId];

    fn process(&mut self, inputs: Params, outputs: Outputs) -> Result<(), ()>;
}
