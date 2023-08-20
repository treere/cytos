use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use crate::{data::Data, map::Map};

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
    inputs: Vec<Path>,
}

impl Processor {
    fn new(
        id: NodeId,
        fun: impl Transformer + InputConfiguration + OutputConfiguration + 'static,
    ) -> Self {
        Self {
            id,
            inputs: fun.inputs().iter().map(|x| Path::new(id, *x)).collect(),
            fun: Box::new(fun),
        }
    }

    fn inputs(&self) -> &Vec<Path> {
        &self.inputs
    }

    fn process(&mut self, inputs: Params, outputs: Outputs) -> Result<(), ()> {
        self.fun.process(inputs, outputs)
    }
}

type ParamData = Map<ParamId, Rc<RefCell<Data>>>;

struct Communication {
    outputs: Map<NodeId, ParamData>,
    inputs: Map<NodeId, ParamData>,
}

impl Communication {
    fn new() -> Self {
        Self {
            outputs: Map::new(),
            inputs: Map::new(),
        }
    }

    fn add_output(
        &mut self,
        id: NodeId,
        processor: &(impl Transformer + InputConfiguration + OutputConfiguration),
    ) {
        self.outputs.insert(
            id,
            Map::from_iterator(
                processor
                    .outputs_default()
                    .into_iter()
                    .map(|(n, data)| (n, Rc::new(RefCell::new(data)))),
            ),
        );

        self.inputs.insert(
            id,
            Map::from_iterator(
                processor
                    .inputs_default()
                    .into_iter()
                    .map(|(n, data)| (n, Rc::new(RefCell::new(data)))),
            ),
        )
    }

    fn connect(&mut self, src: Path, dst: Path) -> Result<(), ()> {
        let output = self
            .outputs
            .get(&src.node)
            .and_then(|node| node.get(&src.param))
            .ok_or(())?;

        self.inputs
            .get_mut(&dst.node)
            .map(|node| node.insert(dst.param, output.clone()))
            .ok_or(())
    }

    fn get_arguments(&mut self, id: NodeId) -> (Params, Outputs) {
        let inputs = self.inputs.get(&id).unwrap();
        let outputs = self.outputs.get_mut(&id).unwrap();
        (Params { map: inputs }, Outputs { map: outputs })
    }

    fn get_outputs(&self, id: NodeId) -> Params {
        self.outputs.get(&id).map(|p| Params { map: p }).unwrap()
    }
}

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
        self.map.get_mut(val).map(|p| (**p).borrow_mut()).unwrap()
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

    pub fn add(
        mut self,
        id: NodeId,
        processor: impl Transformer + InputConfiguration + OutputConfiguration + 'static,
    ) -> Result<Self, ()> {
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
    fn process(&mut self, inputs: Params, outputs: Outputs) -> Result<(), ()>;
}

pub trait InputConfiguration {
    fn inputs(&self) -> &[ParamId];
    fn inputs_default(&self) -> Vec<(ParamId, Data)>;
}

pub trait OutputConfiguration {
    fn outputs(&self) -> &[ParamId];
    fn outputs_default(&self) -> Vec<(ParamId, Data)>;
}
