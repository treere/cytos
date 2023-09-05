//! Struct to manage graph architecture.

use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    cmp::Ordering,
    collections::HashMap,
    ops::Deref,
    rc::Rc,
};

pub type NodeId = String;
pub type ParamId = String;

/// Identify a param inside a node.
#[derive(Debug, PartialEq, Eq, Clone)]
struct Path {
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
pub struct Processor {
    /// Node identifier.
    id: NodeId,

    /// Wrapped transformer.
    fun: Box<dyn Transformer>,
}

impl Processor {
    /// Create a new Processor.
    pub fn new(id: NodeId, fun: impl Transformer + 'static) -> Self {
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
}

impl Graph {
    /// Created a new instance.
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Add a processor with a given id to the graph.
    pub fn insert(mut self, processor: Processor) -> Result<Self, &'static str> {
        if self.nodes.iter().all(|x| x.id != processor.id) {
            self.nodes.push(processor);

            Ok(self)
        } else {
            Err("already exist")
        }
    }

    /// Connects a output data to an input one.
    pub fn connect(
        mut self,
        src: (NodeId, ParamId),
        dst: (NodeId, ParamId),
    ) -> Result<Self, &'static str> {
        let src: Path = src.into();
        let dst: Path = dst.into();

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
            .and_then(|d| d.fun.link(dst.param, output))?;

        self.order_nodes();

        Ok(self)
    }

    /// Comute one step of processing
    pub fn step(&mut self) -> Result<(), &'static str> {
        for node in self.nodes.iter_mut() {
            node.process()?;
        }

        Ok(())
    }

    pub fn param_value(&self, node: NodeId) -> Option<&dyn Transformer> {
        self.nodes
            .iter()
            .find(|x| x.id == node)
            .map(|p| p.fun.deref())
    }

    /// Reorder nodes so there cannot a required node after.
    fn order_nodes(&mut self) {
        let links = self.find_links();

        let mut p = HashMap::new();
        for (s, d) in links.iter() {
            p.entry(d.node.clone())
                .or_insert(Vec::new())
                .push(s.node.clone())
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

    fn find_links(&self) -> Vec<(Path, Path)> {
        let inputs: Vec<_> = self
            .nodes
            .iter()
            .flat_map(|n| {
                n.fun
                    .input_names()
                    .iter()
                    .map(|p| {
                        (
                            Path::new(n.id.clone(), p.clone()),
                            n.fun.input(p.clone()).unwrap(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        let outputs: Vec<_> = self
            .nodes
            .iter()
            .flat_map(|n| {
                n.fun
                    .output_names()
                    .iter()
                    .map(|p| {
                        (
                            Path::new(n.id.clone(), p.clone()),
                            n.fun.output(p.clone()).unwrap(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        let mut v = vec![];
        for i in inputs.iter() {
            for j in outputs.iter() {
                if Rc::ptr_eq(&i.1.prop, &j.1.prop) {
                    v.push((j.0.clone(), i.0.clone()));
                }
            }
        }
        v
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

    pub fn change_value(&mut self, val: GenericOutputProp) -> Result<(), &'static str> {
        if let Ok(v) = val.prop.downcast::<RefCell<T>>() {
            self.val = v;
            Ok(())
        } else {
            Err("invalid type")
        }
    }

    pub fn as_generic(&self) -> GenericInputProp {
        GenericInputProp {
            prop: self.val.clone(),
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

    pub fn as_generic(&self) -> GenericOutputProp {
        GenericOutputProp {
            prop: self.val.clone(),
        }
    }
}

/// Generic Property to be casted back
pub struct GenericOutputProp {
    prop: Rc<dyn Any>,
}

impl GenericOutputProp {
    pub fn try_read<T: 'static>(&self, f: impl Fn(&T)) -> Result<(), &'static str> {
        if let Ok(v) = self.prop.clone().downcast::<RefCell<T>>() {
            f(v.borrow().deref());
            Ok(())
        } else {
            Err("wrong type")
        }
    }
}

pub struct GenericInputProp {
    prop: Rc<dyn Any>,
}

impl GenericInputProp {
    pub fn try_read<T: 'static>(&self, f: impl Fn(&T)) -> Result<(), &'static str> {
        if let Ok(v) = self.prop.clone().downcast::<RefCell<T>>() {
            f(v.borrow().deref());
            Ok(())
        } else {
            Err("wrong type")
        }
    }

    pub fn is_linked_to(&self, other: &GenericOutputProp) -> bool {
        Rc::ptr_eq(&self.prop, &other.prop)
    }
}

/// Transformer trait
pub trait Transformer {
    /// Process an input
    fn step(&mut self) -> Result<(), &'static str>;

    /// Set input
    fn link(&mut self, name: ParamId, val: GenericOutputProp) -> Result<(), &'static str>;

    /// Get the default of a parameter
    fn output(&self, val: ParamId) -> Option<GenericOutputProp>;

    fn input(&self, val: ParamId) -> Option<GenericInputProp>;

    fn input_names(&self) -> Vec<ParamId> {
        vec![]
    }

    fn output_names(&self) -> Vec<ParamId> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        architecture::{Graph, Processor},
        transformer::{
            AddValue, AddValueConfigInput, IncrementalGenerator, IncrementalGeneratorConfigOutput,
        },
    };

    #[test]
    fn test_add_success() {
        assert!(Graph::new()
            .insert(Processor::new(
                "SOURCE1".to_string(),
                IncrementalGenerator::new()
            ))
            .expect("cannot insert")
            .insert(Processor::new(
                "SOURCE2".to_string(),
                IncrementalGenerator::new()
            ))
            .is_ok())
    }

    #[test]
    fn test_add_same_name() {
        assert!(Graph::new()
            .insert(Processor::new(
                "SOURCE".to_string(),
                IncrementalGenerator::new()
            ))
            .expect("cannot insert")
            .insert(Processor::new(
                "SOURCE".to_string(),
                IncrementalGenerator::new()
            ))
            .is_err())
    }

    #[test]
    fn test_connect_success() {
        assert!(Graph::new()
            .insert(Processor::new(
                "SOURCE".to_owned(),
                IncrementalGenerator::new()
            ))
            .expect("cannot add source")
            .insert(Processor::new("DOUBLER".to_owned(), AddValue::new()))
            .expect("cannot add doubler")
            .connect(
                (
                    "SOURCE".to_owned(),
                    IncrementalGeneratorConfigOutput::OUTPUT.to_owned()
                ),
                ("DOUBLER".to_owned(), AddValueConfigInput::INPUT.to_owned())
            )
            .is_ok())
    }

    #[test]
    fn test_connect_missing_destination_source() {
        assert!(Graph::new()
            .insert(Processor::new(
                "SOURCE".to_owned(),
                IncrementalGenerator::new()
            ))
            .expect("cannot add source")
            .insert(Processor::new("DOUBLER".to_owned(), AddValue::new()))
            .expect("cannot add doubler")
            .connect(
                (
                    "SOURCE".to_owned(),
                    IncrementalGeneratorConfigOutput::OUTPUT.to_owned()
                ),
                ("PIPPO".to_owned(), "PLUTO".to_owned())
            )
            .is_err())
    }

    #[test]
    fn test_connect_missing_destination_value() {
        assert!(Graph::new()
            .insert(Processor::new(
                "SOURCE".to_owned(),
                IncrementalGenerator::new()
            ))
            .expect("cannot add source")
            .insert(Processor::new("DOUBLER".to_owned(), AddValue::new()))
            .expect("cannot add doubler")
            .connect(
                (
                    "SOURCE".to_owned(),
                    IncrementalGeneratorConfigOutput::OUTPUT.to_owned()
                ),
                ("DOUBLER".to_owned(), "PLUTO".to_owned())
            )
            .is_err())
    }

    #[test]
    fn test_connect_missing_source_source() {
        assert!(Graph::new()
            .insert(Processor::new(
                "SOURCE".to_owned(),
                IncrementalGenerator::new()
            ))
            .expect("cannot add source")
            .insert(Processor::new("DOUBLER".to_owned(), AddValue::new()))
            .expect("cannot add doubler")
            .connect(
                ("PIPPO".to_owned(), "PLUTO".to_owned()),
                ("DOUBLER".to_owned(), AddValueConfigInput::INPUT.to_owned())
            )
            .is_err())
    }

    #[test]
    fn test_connect_missing_source_value() {
        assert!(Graph::new()
            .insert(Processor::new(
                "SOURCE".to_owned(),
                IncrementalGenerator::new()
            ))
            .expect("cannot add source")
            .insert(Processor::new("DOUBLER".to_owned(), AddValue::new()))
            .expect("cannot add doubler")
            .connect(
                ("SOURCE".to_owned(), "PLUTO".to_owned()),
                ("DOUBLER".to_owned(), AddValueConfigInput::INPUT.to_owned())
            )
            .is_err())
    }
}
