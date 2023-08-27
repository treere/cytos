//! Struct to manage graph architecture.

use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    cmp::Ordering,
    collections::HashMap,
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
            .and_then(|s| s.fun.output(src.param).ok_or("cannot find param"))?;

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
pub struct InputProp<T> {
    val: Rc<RefCell<T>>,
}

impl<T: 'static> InputProp<T> {
    pub fn new(val: T) -> Self {
        Self {
            val: Rc::new(RefCell::new(val)),
        }
    }

    pub fn get(&self) -> Ref<'_, T> {
        self.val.borrow()
    }

    pub fn get_any(&self) -> Rc<GenericProp> {
        self.val.clone()
    }

    pub fn change_value(&mut self, val: Rc<GenericProp>) -> Result<(), &'static str> {
        if let Ok(v) = val.downcast::<RefCell<T>>() {
            self.val = v;
            Ok(())
        } else {
            Err("invalid type")
        }
    }
}

pub struct OutputProp<T> {
    val: Rc<RefCell<T>>,
}

impl<T: 'static> OutputProp<T> {
    pub fn new(val: T) -> Self {
        Self {
            val: Rc::new(RefCell::new(val)),
        }
    }

    pub fn get(&self) -> Ref<'_, T> {
        self.val.borrow()
    }

    pub fn set(&self) -> RefMut<'_, T> {
        self.val.borrow_mut()
    }

    pub fn get_any(&self) -> Rc<GenericProp> {
        self.val.clone()
    }

    pub fn change_value(&mut self, val: Rc<GenericProp>) -> Result<(), &'static str> {
        if let Ok(v) = val.downcast::<RefCell<T>>() {
            self.val = v;
            Ok(())
        } else {
            Err("invalid type")
        }
    }
}

/// Generic Property to be casted back
pub type GenericProp = dyn Any;

/// Transformer trait
pub trait Transformer {
    /// Process an input
    fn step(&mut self) -> Result<(), &'static str>;

    /// Get the default of a parameter
    fn input(&self, val: ParamId) -> Option<Rc<GenericProp>>;

    /// Set input
    fn set_input(&mut self, name: ParamId, val: Rc<GenericProp>) -> Result<(), &'static str>;

    /// Get the default of a parameter
    fn output(&self, val: ParamId) -> Option<Rc<GenericProp>>;
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
