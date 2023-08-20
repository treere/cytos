use std::{
    cell::RefCell,
    cmp::Ordering,
    collections::HashMap,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use crate::{containers::VecMap, data::Data};

pub type NodeId = u32;
pub type ParamId = u32;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Path {
    node: NodeId,
    param: ParamId,
}

impl From<(NodeId, ParamId)> for Path {
    fn from((node, param): (NodeId, ParamId)) -> Self {
        Path::new(node, param)
    }
}

impl Path {
    fn new(node: NodeId, param: ParamId) -> Self {
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

    fn process(&mut self, inputs: Params, outputs: Outputs) -> Result<(), ()> {
        self.fun.process(inputs, outputs)
    }
}

type ParamData = VecMap<ParamId, Rc<RefCell<Data>>>;

#[derive(Debug)]
struct Communication {
    outputs: VecMap<NodeId, ParamData>,
    inputs: VecMap<NodeId, ParamData>,
}

impl Communication {
    fn new() -> Self {
        Self {
            outputs: VecMap::new(),
            inputs: VecMap::new(),
        }
    }

    fn add_processor(
        &mut self,
        id: NodeId,
        processor: &(impl InputConfiguration + OutputConfiguration),
    ) {
        self.outputs.insert(
            id,
            VecMap::from_iterator(
                processor
                    .outputs()
                    .iter()
                    .map(|n| (*n, Rc::new(RefCell::new(processor.outputs_default(*n))))),
            ),
        );

        self.inputs.insert(
            id,
            VecMap::from_iterator(
                processor
                    .inputs()
                    .iter()
                    .map(|n| (*n, Rc::new(RefCell::new(processor.inputs_default(*n))))),
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
            .and_then(|node| node.get_mut(&dst.param))
            .map(|param| *param = output.clone())
            .ok_or(())
    }

    fn get_arguments(&mut self, id: NodeId) -> Option<(Params, Outputs)> {
        let inputs = self.inputs.get(&id)?;
        let outputs = self.outputs.get_mut(&id)?;
        Some((Params { map: inputs }, Outputs { map: outputs }))
    }

    fn get_outputs(&self, id: NodeId) -> Option<Params> {
        self.outputs.get(&id).map(|p| Params { map: p })
    }
}

#[derive(Debug)]
pub struct Params<'a> {
    map: &'a ParamData,
}

impl<'a> Params<'a> {
    pub fn get(&self, val: &ParamId) -> Option<impl Deref<Target = Data> + 'a> {
        self.map.get(val).map(|x| x.borrow())
    }
}

#[derive(Debug)]
pub struct Outputs<'a> {
    map: &'a mut ParamData,
}

impl<'a> Outputs<'a> {
    pub fn get_mut<'b>(
        &'b mut self,
        val: &'b ParamId,
    ) -> Option<impl DerefMut<Target = Data> + 'b> {
        self.map.get_mut(val).map(|p| (**p).borrow_mut())
    }
}

pub struct Orchestrator {
    nodes: Vec<Processor>,
    links: Vec<(Path, Path)>,
    communication: Communication,
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            links: Vec::new(),
            communication: Communication::new(),
        }
    }

    pub fn add(
        mut self,
        id: NodeId,
        processor: impl Transformer + InputConfiguration + OutputConfiguration + 'static,
    ) -> Result<Self, ()> {
        if self.communication.get_outputs(id).is_none() {
            self.communication.add_processor(id, &processor);
            self.nodes.push(Processor::new(id, processor));

            Ok(self)
        } else {
            Err(())
        }
    }

    pub fn connect(mut self, src: impl Into<Path>, dst: impl Into<Path>) -> Result<Self, ()> {
        let src = src.into();
        let dst = dst.into();
        self.communication.connect(src.clone(), dst.clone())?;

        self.links.push((src, dst));

        self.order_nodes();

        Ok(self)
    }

    fn order_nodes(&mut self) {
        let mut p = HashMap::new();
        for (s, d) in self.links.iter() {
            p.entry(d.node).or_insert(Vec::new()).push(s.node)
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

    pub fn step(&mut self) -> Result<(), ()> {
        for node in self.nodes.iter_mut() {
            let (inputs, outputs) = self.communication.get_arguments(node.id).ok_or(())?;
            node.process(inputs, outputs)?;
        }

        Ok(())
    }

    pub fn value(
        &mut self,
        node: NodeId,
        param: ParamId,
    ) -> Option<impl Deref<Target = Data> + '_> {
        self.communication.get_outputs(node)?.get(&param)
    }
}

pub trait Transformer {
    fn process(&mut self, inputs: Params, outputs: Outputs) -> Result<(), ()>;
}

pub trait InputConfiguration {
    fn inputs(&self) -> &[ParamId];
    fn inputs_default(&self, val: ParamId) -> Data;
}

pub trait OutputConfiguration {
    fn outputs(&self) -> &[ParamId];
    fn outputs_default(&self, val: ParamId) -> Data;
}

#[cfg(test)]
mod tests {
    use crate::{
        architecture::{NodeId, Orchestrator, ParamId},
        transformer::{
            AddConfigConfigInput, AddValue, IncrementalGenerator, IncrementalGeneratorConfigOutput,
        },
    };

    pub const SOURCE1: NodeId = 7;
    pub const SOURCE2: NodeId = 8;
    pub const SOURCE: NodeId = 1;
    pub const DOUBLER: NodeId = 9;
    pub const PIPPO: NodeId = 255;
    pub const PLUTO: ParamId = 255;

    #[test]
    fn test_add_success() {
        assert!(Orchestrator::new()
            .add(SOURCE1, IncrementalGenerator::new())
            .expect("cannot insert")
            .add(SOURCE2, IncrementalGenerator::new())
            .is_ok())
    }

    #[test]
    fn test_add_same_name() {
        assert!(Orchestrator::new()
            .add(SOURCE, IncrementalGenerator::new())
            .expect("cannot insert")
            .add(SOURCE, IncrementalGenerator::new())
            .is_err())
    }

    #[test]
    fn test_connect_success() {
        assert!(Orchestrator::new()
            .add(SOURCE, IncrementalGenerator::new())
            .expect("cannot add source")
            .add(DOUBLER, AddValue::new())
            .expect("cannot add doubler")
            .connect(
                (SOURCE, IncrementalGeneratorConfigOutput::OUTPUT),
                (DOUBLER, AddConfigConfigInput::INPUT)
            )
            .is_ok())
    }

    #[test]
    fn test_connect_missing_destination_source() {
        assert!(Orchestrator::new()
            .add(SOURCE, IncrementalGenerator::new())
            .expect("cannot add source")
            .add(DOUBLER, AddValue::new())
            .expect("cannot add doubler")
            .connect(
                (SOURCE, IncrementalGeneratorConfigOutput::OUTPUT),
                (PIPPO, PLUTO)
            )
            .is_err())
    }

    #[test]
    fn test_connect_missing_destination_value() {
        assert!(Orchestrator::new()
            .add(SOURCE, IncrementalGenerator::new())
            .expect("cannot add source")
            .add(DOUBLER, AddValue::new())
            .expect("cannot add doubler")
            .connect(
                (SOURCE, IncrementalGeneratorConfigOutput::OUTPUT),
                (DOUBLER, PLUTO)
            )
            .is_err())
    }

    #[test]
    fn test_connect_missing_source_source() {
        assert!(Orchestrator::new()
            .add(SOURCE, IncrementalGenerator::new())
            .expect("cannot add source")
            .add(DOUBLER, AddValue::new())
            .expect("cannot add doubler")
            .connect((PIPPO, PLUTO), (DOUBLER, AddConfigConfigInput::INPUT))
            .is_err())
    }

    #[test]
    fn test_connect_missing_source_value() {
        assert!(Orchestrator::new()
            .add(SOURCE, IncrementalGenerator::new())
            .expect("cannot add source")
            .add(DOUBLER, AddValue::new())
            .expect("cannot add doubler")
            .connect((SOURCE, PLUTO), (DOUBLER, AddConfigConfigInput::INPUT))
            .is_err())
    }
}
