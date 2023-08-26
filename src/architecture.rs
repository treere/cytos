//! Struct to manage graph architecture.

use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    cmp::Ordering,
    collections::HashMap,
    marker::PhantomData,
    rc::Rc,
};

pub type NodeId = &'static str;
pub type ParamId = &'static str;

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
    fn process(&mut self) -> Result<(), &'static str> {
        self.fun.step()
    }
}

/// Shared param data.
pub type SharedData = Rc<RefCell<dyn Any + 'static>>;

fn new_shared<T: 'static>(v: T) -> SharedData {
    Rc::new(RefCell::new(v))
}

/// Graph.
pub struct Graph {
    nodes: Vec<Processor>,
    links: Vec<(Path, Path)>,
}

impl Graph {
    /// Created a new instance.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            links: Vec::new(),
        }
    }

    /// Add a processor with a given id to the graph.
    pub fn add(
        mut self,
        id: NodeId,
        processor: impl Transformer + 'static,
    ) -> Result<Self, &'static str> {
        self.nodes.push(Processor::new(id, processor));

        Ok(self)
    }

    /// Connects a output data to an input one.
    pub fn connect(
        mut self,
        src: impl Into<Path>,
        dst: impl Into<Path>,
    ) -> Result<Self, &'static str> {
        let src = src.into();
        let dst = dst.into();

        let output = self
            .nodes
            .iter()
            .find(|p| p.id == src.node)
            .ok_or("cannot find source")
            .map(|s| s.fun.output(src.param))?;

        self.nodes
            .iter_mut()
            .find(|p| p.id == dst.node)
            .ok_or("cannot find dest")
            .and_then(|d| d.fun.set_input(dst.param, output))?;

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
            node.process()?;
        }

        Ok(())
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

/// A property
pub struct Prop<T> {
    val: SharedData,
    _ph: PhantomData<T>,
}

impl<T: 'static> Prop<T> {
    pub fn new(val: T) -> Self {
        Self {
            val: new_shared(val),
            _ph: PhantomData,
        }
    }

    pub fn get(&self) -> Ref<'_, T> {
        Ref::map(self.val.borrow(), |x| x.downcast_ref::<T>().unwrap())
    }

    pub fn set(&self) -> RefMut<'_, T> {
        RefMut::map(self.val.borrow_mut(), |x| x.downcast_mut::<T>().unwrap())
    }

    pub fn get_shared(&self) -> SharedData {
        self.val.clone()
    }

    pub fn change_value(&mut self, val: SharedData) -> Result<(), &'static str> {
        self.val = val;
        Ok(())
    }
}

/// Transformer trait
pub trait Transformer {
    /// Process an input
    fn step(&mut self) -> Result<(), &'static str>;

    /// Inputs list
    fn inputs_name(&self) -> &[ParamId];

    /// Get the default of a parameter
    fn input(&self, val: ParamId) -> SharedData;

    /// Set input
    fn set_input(&mut self, name: ParamId, val: SharedData) -> Result<(), &'static str>;

    /// Output list
    fn outputs_name(&self) -> &[ParamId];

    /// Get the default of a parameter
    fn output(&self, val: ParamId) -> SharedData;
}

#[cfg(test)]
mod tests {
    use crate::{
        architecture::Graph,
        transformer::{
            AddValue, AddValueConfigInput, IncrementalGenerator, IncrementalGeneratorConfigOutput,
        },
    };

    #[test]
    fn test_add_success() {
        assert!(Graph::new()
            .add("SOURCE1", IncrementalGenerator::new())
            .expect("cannot insert")
            .add("SOURCE2", IncrementalGenerator::new())
            .is_ok())
    }

    #[test]
    fn test_add_same_name() {
        assert!(Graph::new()
            .add("SOURCE", IncrementalGenerator::new())
            .expect("cannot insert")
            .add("SOURCE", IncrementalGenerator::new())
            .is_err())
    }

    #[test]
    fn test_connect_success() {
        assert!(Graph::new()
            .add("SOURCE", IncrementalGenerator::new())
            .expect("cannot add source")
            .add("DOUBLER", AddValue::new())
            .expect("cannot add doubler")
            .connect(
                ("SOURCE", IncrementalGeneratorConfigOutput::OUTPUT),
                ("DOUBLER", AddValueConfigInput::INPUT)
            )
            .is_ok())
    }

    #[test]
    fn test_connect_missing_destination_source() {
        assert!(Graph::new()
            .add("SOURCE", IncrementalGenerator::new())
            .expect("cannot add source")
            .add("DOUBLER", AddValue::new())
            .expect("cannot add doubler")
            .connect(
                ("SOURCE", IncrementalGeneratorConfigOutput::OUTPUT),
                ("PIPPO", "PLUTO")
            )
            .is_err())
    }

    #[test]
    fn test_connect_missing_destination_value() {
        assert!(Graph::new()
            .add("SOURCE", IncrementalGenerator::new())
            .expect("cannot add source")
            .add("DOUBLER", AddValue::new())
            .expect("cannot add doubler")
            .connect(
                ("SOURCE", IncrementalGeneratorConfigOutput::OUTPUT),
                ("DOUBLER", "PLUTO")
            )
            .is_err())
    }

    #[test]
    fn test_connect_missing_source_source() {
        assert!(Graph::new()
            .add("SOURCE", IncrementalGenerator::new())
            .expect("cannot add source")
            .add("DOUBLER", AddValue::new())
            .expect("cannot add doubler")
            .connect(("PIPPO", "PLUTO"), ("DOUBLER", AddValueConfigInput::INPUT))
            .is_err())
    }

    #[test]
    fn test_connect_missing_source_value() {
        assert!(Graph::new()
            .add("SOURCE", IncrementalGenerator::new())
            .expect("cannot add source")
            .add("DOUBLER", AddValue::new())
            .expect("cannot add doubler")
            .connect(("SOURCE", "PLUTO"), ("DOUBLER", AddValueConfigInput::INPUT))
            .is_err())
    }
}
