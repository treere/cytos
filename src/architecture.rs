//! Struct to manage graph architecture.

use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    cmp::Ordering,
    collections::HashMap,
    rc::Rc,
};

use crate::containers::VecMap;

pub type NodeId = u32;
pub type ParamId = u32;

/// Identify a param inside a node.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Path {
    node: NodeId,
    param: ParamId,
}

impl Path {
    fn new(node: NodeId, param: ParamId) -> Self {
        Path { node, param }
    }
}

impl From<(NodeId, ParamId)> for Path {
    fn from((node, param): (NodeId, ParamId)) -> Self {
        Path::new(node, param)
    }
}

/// A wrapper around a [`Transformer`] keeping trace of the node id.
struct Processor {
    /// Node identifier.
    id: NodeId,

    /// Wrapped transformer.
    fun: Box<dyn Transformer>,
}

impl Processor {
    /// Create a new Processor.
    fn new(id: NodeId, fun: impl Transformer + 'static) -> Self {
        Self {
            id,
            fun: Box::new(fun),
        }
    }

    /// Process the data reading data from [`Params`] and write the output to [`Results`].
    fn process(&mut self, inputs: Params, outputs: Results) -> Result<(), &'static str> {
        self.fun.process(inputs, outputs)
    }
}

/// Shared param data.
pub type SharedData = Rc<RefCell<dyn Any + 'static>>;

pub fn new_shared<T: 'static>(v: T) -> SharedData {
    Rc::new(RefCell::new(v))
}

/// Data shared between nodes.
type SharedNodeData = VecMap<ParamId, SharedData>;

struct Communication {
    /// Output values per node.
    outputs: VecMap<NodeId, SharedNodeData>,

    /// Input parameters per node.
    inputs: VecMap<NodeId, SharedNodeData>,
}

impl Communication {
    /// Creates a new instance.
    fn new() -> Self {
        Self {
            outputs: VecMap::new(),
            inputs: VecMap::new(),
        }
    }

    /// Add a processor to communication.
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
                    .map(|n| (*n, processor.output_default(*n))),
            ),
        );

        self.inputs.insert(
            id,
            VecMap::from_iterator(
                processor
                    .inputs()
                    .iter()
                    .map(|n| (*n, processor.input_default(*n))),
            ),
        )
    }

    /// Connect an output to in an input.
    fn connect(&mut self, src: Path, dst: Path) -> Result<(), &'static str> {
        let output = self
            .outputs
            .get(&src.node)
            .and_then(|node| node.get(&src.param))
            .ok_or("cannot find output node")?;

        self.inputs
            .get_mut(&dst.node)
            .and_then(|node| node.get_mut(&dst.param))
            .map(|param| *param = output.clone())
            .ok_or("cannot find input")
    }

    /// Get data used by a node.
    fn get_node_data(&mut self, id: NodeId) -> Option<(Params, Results)> {
        let inputs = self.inputs.get(&id)?;
        let outputs = self.outputs.get_mut(&id)?;
        Some((Params { map: inputs }, Results { map: outputs }))
    }

    /// Get the outputs of a node.
    fn get_outputs(&self, id: NodeId) -> Result<Params, &'static str> {
        self.outputs
            .get(&id)
            .ok_or("missing output")
            .map(|p| Params { map: p })
    }
}

/// Parameter struct.

pub struct Params<'a> {
    map: &'a SharedNodeData,
}

impl<'a> Params<'a> {
    /// Get the value of a parameter.
    pub fn get<T: 'static>(&self, val: &ParamId) -> Result<Ref<'_, T>, &'static str> {
        self.map
            .get(val)
            .and_then(|x| {
                let borrow = x.borrow();
                if borrow.is::<T>() {
                    Some(Ref::map(borrow, |x| x.downcast_ref::<T>().unwrap()))
                } else {
                    None
                }
            })
            .ok_or("wrong type")
    }
}

/// Result struct.

pub struct Results<'a> {
    map: &'a mut SharedNodeData,
}

impl<'a> Results<'a> {
    /// Get the mutable value.
    pub fn get_mut<T: 'static>(&mut self, val: &ParamId) -> Result<RefMut<'_, T>, &'static str> {
        self.map
            .get(val)
            .and_then(|x| {
                let borrow = x.borrow_mut();
                if borrow.is::<T>() {
                    Some(RefMut::map(borrow, |x| x.downcast_mut::<T>().unwrap()))
                } else {
                    None
                }
            })
            .ok_or("wrong type")
    }

    /// Get the value.
    pub fn get<T: 'static>(&mut self, val: &ParamId) -> Result<Ref<'_, T>, &'static str> {
        self.map
            .get(val)
            .and_then(|x| {
                let borrow = x.borrow();
                if borrow.is::<T>() {
                    Some(Ref::map(borrow, |x| x.downcast_ref::<T>().unwrap()))
                } else {
                    None
                }
            })
            .ok_or("wrong type")
    }
}

/// Graph.
pub struct Graph {
    nodes: Vec<Processor>,
    links: Vec<(Path, Path)>,
    communication: Communication,
}

impl Graph {
    /// Created a new instance.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            links: Vec::new(),
            communication: Communication::new(),
        }
    }

    /// Add a processor with a given id to the graph.
    pub fn add(
        mut self,
        id: NodeId,
        processor: impl Transformer + InputConfiguration + OutputConfiguration + 'static,
    ) -> Result<Self, &'static str> {
        if self.communication.get_outputs(id).is_err() {
            self.communication.add_processor(id, &processor);
            self.nodes.push(Processor::new(id, processor));

            Ok(self)
        } else {
            Err("node alrealy exist")
        }
    }

    /// Connects a output data to an input one.
    pub fn connect(
        mut self,
        src: impl Into<Path>,
        dst: impl Into<Path>,
    ) -> Result<Self, &'static str> {
        let src = src.into();
        let dst = dst.into();
        self.communication.connect(src.clone(), dst.clone())?;

        self.links.push((src, dst));

        self.order_nodes();

        Ok(self)
    }

    /// Reorder nodes so there cannot a required node after.
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

    /// Comute one step of processing
    pub fn step(&mut self) -> Result<(), &'static str> {
        for node in self.nodes.iter_mut() {
            let (inputs, outputs) = self
                .communication
                .get_node_data(node.id)
                .ok_or("cannot get node shared data")?;
            node.process(inputs, outputs)?;
        }

        Ok(())
    }

    pub fn param_value(&mut self, node: NodeId) -> Result<Params, &'static str> {
        self.communication.get_outputs(node)
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

/// Transformer trait
pub trait Transformer {
    /// Process an input
    fn process(&mut self, inputs: Params, outputs: Results) -> Result<(), &'static str>;
}

/// Input configurationn
pub trait InputConfiguration {
    /// Inputs list
    fn inputs(&self) -> &[ParamId];

    /// Get the default of a parameter
    fn input_default(&self, val: ParamId) -> SharedData;
}

/// Output configuration
pub trait OutputConfiguration {
    /// Output list
    fn outputs(&self) -> &[ParamId];

    /// Get the default of a parameter
    fn output_default(&self, val: ParamId) -> SharedData;
}

#[cfg(test)]
mod tests {
    use crate::{
        architecture::{Graph, NodeId, ParamId},
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
        assert!(Graph::new()
            .add(SOURCE1, IncrementalGenerator::new())
            .expect("cannot insert")
            .add(SOURCE2, IncrementalGenerator::new())
            .is_ok())
    }

    #[test]
    fn test_add_same_name() {
        assert!(Graph::new()
            .add(SOURCE, IncrementalGenerator::new())
            .expect("cannot insert")
            .add(SOURCE, IncrementalGenerator::new())
            .is_err())
    }

    #[test]
    fn test_connect_success() {
        assert!(Graph::new()
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
        assert!(Graph::new()
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
        assert!(Graph::new()
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
        assert!(Graph::new()
            .add(SOURCE, IncrementalGenerator::new())
            .expect("cannot add source")
            .add(DOUBLER, AddValue::new())
            .expect("cannot add doubler")
            .connect((PIPPO, PLUTO), (DOUBLER, AddConfigConfigInput::INPUT))
            .is_err())
    }

    #[test]
    fn test_connect_missing_source_value() {
        assert!(Graph::new()
            .add(SOURCE, IncrementalGenerator::new())
            .expect("cannot add source")
            .add(DOUBLER, AddValue::new())
            .expect("cannot add doubler")
            .connect((SOURCE, PLUTO), (DOUBLER, AddConfigConfigInput::INPUT))
            .is_err())
    }
}
