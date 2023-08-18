#![allow(unused_variables)]

use std::{collections::HashMap, mem};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct Path {
    node: String,
    field: String,
}

impl Path {
    fn new(node: impl ToString, field: impl ToString) -> Self {
        Path {
            node: node.to_string(),
            field: field.to_string(),
        }
    }
}

#[derive(Debug)]
struct Board(HashMap<Path, u8>);

impl Board {
    fn new() -> Self {
        Board(HashMap::new())
    }

    fn contains(&self, path: &Path) -> bool {
        self.0.contains_key(path)
    }

    fn merge(&mut self, board: Board) {
        self.0.extend(board.0)
    }
}

struct Processor {
    name: String,
    fun: Box<dyn Transformer>,
}

impl Processor {
    fn new(name: impl ToString, fun: impl Transformer + 'static) -> Self {
        Self {
            name: name.to_string(),
            fun: Box::new(fun),
        }
    }

    fn inputs(&self) -> Vec<Path> {
        self.fun
            .inputs()
            .iter()
            .map(|x| Path::new(self.name.clone(), x))
            .collect()
    }

    fn outputs(&self) -> Vec<Path> {
        self.fun
            .outputs()
            .iter()
            .map(|x| Path::new(self.name.clone(), x))
            .collect()
    }

    fn process(&self, val: &HashMap<&str, u8>) -> Result<HashMap<String, u8>, ()> {
        self.fun.process(&val)
    }
}

#[derive(Debug, Clone)]
struct Link {
    src: Path,
    dst: Path,
}

impl Link {
    fn new(src: Path, dst: Path) -> Self {
        Self { src, dst }
    }
}

struct Links {
    links: Vec<Link>,
}

impl Links {
    fn new() -> Self {
        Self { links: Vec::new() }
    }

    fn push(&mut self, link: Link) {
        self.links.push(link)
    }

    fn iter_by_dst(&self, name: &str) -> impl Iterator<Item = &Link> {
        let name = name.to_owned();
        self.links.iter().filter(move |r| r.dst.node == name)
    }
}

struct Orchestrator {
    nodes: Vec<Processor>,
    links: Links,
}

impl Orchestrator {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            links: Links::new(),
        }
    }

    fn add(
        mut self,
        name: impl ToString,
        processor: impl Transformer + 'static,
    ) -> Result<Self, ()> {
        let name = name.to_string();
        if self.nodes.iter().find(|n| n.name == name).is_none() {
            self.nodes.push(Processor::new(name, processor));
            Ok(self)
        } else {
            Err(())
        }
    }

    fn connect(mut self, src: Path, dst: Path) -> Result<Self, ()> {
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

        self.links.push(Link::new(src, dst));

        Ok(self)
    }

    fn step(mut self) -> Result<Board, ()> {
        let mut nodes = mem::replace(&mut self.nodes, Vec::new());

        let mut board = Board::new();

        while !nodes.is_empty() {
            if let Some(node) = self
                .index_first_ready_node(&nodes, &board)
                .map(|index| nodes.remove(index))
            {
                let params: HashMap<_, _> = self
                    .links
                    .iter_by_dst(&node.name)
                    .map(|r| (&r.dst.field[..], board.0[&r.src]))
                    .collect();

                if let Ok(data) = node.process(&params) {
                    board.merge(Board(
                        data.iter()
                            .map(|r| (Path::new(node.name.clone(), r.0), *r.1))
                            .collect(),
                    ));
                }
            } else {
                return Err(());
            }
        }

        Ok(board)
    }

    fn index_first_ready_node(&self, nodes: &Vec<Processor>, board: &Board) -> Option<usize> {
        nodes
            .iter()
            .enumerate()
            .filter(|(index, node)| {
                self.links
                    .iter_by_dst(&node.name)
                    .all(|r| board.contains(&r.src))
            })
            .map(|(index, _)| index)
            .next()
    }
}

trait Transformer {
    fn inputs(&self) -> Vec<&'static str>;
    fn outputs(&self) -> Vec<&'static str>;

    fn process(&self, val: &HashMap<&str, u8>) -> Result<HashMap<String, u8>, ()>;
}

struct ZerosGenerator;

impl ZerosGenerator {
    fn new() -> Self {
        ZerosGenerator
    }
}

struct ZerosGeneratorProps;

impl From<&HashMap<&str, u8>> for ZerosGeneratorProps {
    fn from(_value: &HashMap<&str, u8>) -> Self {
        ZerosGeneratorProps
    }
}

struct ZerosGeneratorOutput(u8);

impl From<ZerosGeneratorOutput> for HashMap<String, u8> {
    fn from(value: ZerosGeneratorOutput) -> Self {
        HashMap::from([("output".to_owned(), 0)])
    }
}

impl ZerosGenerator {
    fn run(&self, props: ZerosGeneratorProps) -> Result<ZerosGeneratorOutput, ()> {
        Ok(ZerosGeneratorOutput(0))
    }
}

impl Transformer for ZerosGenerator {
    fn inputs(&self) -> Vec<&'static str> {
        Vec::new()
    }

    fn outputs(&self) -> Vec<&'static str> {
        vec!["output"]
    }

    fn process(&self, val: &HashMap<&str, u8>) -> Result<HashMap<String, u8>, ()> {
        Ok(self.run(val.into())?.into())
    }
}

struct AddOne;

struct AddOneProps(u8);

impl From<&HashMap<&str, u8>> for AddOneProps {
    fn from(value: &HashMap<&str, u8>) -> Self {
        AddOneProps(value["input"])
    }
}

struct AddOneOutput(u8);

impl From<AddOneOutput> for HashMap<String, u8> {
    fn from(value: AddOneOutput) -> Self {
        HashMap::from([("output".to_owned(), value.0)])
    }
}

impl AddOne {
    fn run(&self, props: AddOneProps) -> Result<AddOneOutput, ()> {
        Ok(AddOneOutput(props.0 + 1))
    }
}

impl AddOne {
    fn new() -> Self {
        AddOne
    }
}

impl Transformer for AddOne {
    fn inputs(&self) -> Vec<&'static str> {
        vec!["input"]
    }

    fn outputs(&self) -> Vec<&'static str> {
        vec!["output"]
    }

    fn process(&self, val: &HashMap<&str, u8>) -> Result<HashMap<String, u8>, ()> {
        Ok(self.run(val.into())?.into())
    }
}

fn main() -> Result<(), ()> {
    let zero = ZerosGenerator::new();
    let add_one = AddOne::new();

    let result = Orchestrator::new()
        .add("source", zero)?
        .add("doubler", add_one)?
        .connect(Path::new("source", "output"), Path::new("doubler", "input"))?
        .step();

    println!("{:?}", result);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_success() {
        assert!(Orchestrator::new()
            .add("source1", ZerosGenerator::new())
            .expect("cannot insert")
            .add("source2", ZerosGenerator::new())
            .is_ok())
    }

    #[test]
    fn test_add_same_name() {
        assert!(Orchestrator::new()
            .add("source", ZerosGenerator::new())
            .expect("cannot insert")
            .add("source", ZerosGenerator::new())
            .is_err())
    }

    #[test]
    fn test_connect_success() {
        assert!(Orchestrator::new()
            .add("source", ZerosGenerator::new())
            .expect("cannot add source")
            .add("doubler", AddOne::new())
            .expect("cannot add doubler")
            .connect(Path::new("source", "output"), Path::new("doubler", "input"))
            .is_ok())
    }

    #[test]
    fn test_connect_missing_destination_source() {
        assert!(Orchestrator::new()
            .add("source", ZerosGenerator::new())
            .expect("cannot add source")
            .add("doubler", AddOne::new())
            .expect("cannot add doubler")
            .connect(Path::new("source", "output"), Path::new("pippo", "pippo"))
            .is_err())
    }

    #[test]
    fn test_connect_missing_destination_value() {
        assert!(Orchestrator::new()
            .add("source", ZerosGenerator::new())
            .expect("cannot add source")
            .add("doubler", AddOne::new())
            .expect("cannot add doubler")
            .connect(Path::new("source", "output"), Path::new("doubler", "pippo"))
            .is_err())
    }

    #[test]
    fn test_connect_missing_source_source() {
        assert!(Orchestrator::new()
            .add("source", ZerosGenerator::new())
            .expect("cannot add source")
            .add("doubler", AddOne::new())
            .expect("cannot add doubler")
            .connect(Path::new("pippo", "pippo"), Path::new("doubler", "input"))
            .is_err())
    }

    #[test]
    fn test_connect_missing_source_value() {
        assert!(Orchestrator::new()
            .add("source", ZerosGenerator::new())
            .expect("cannot add source")
            .add("doubler", AddOne::new())
            .expect("cannot add doubler")
            .connect(Path::new("source", "pippo"), Path::new("doubler", "input"))
            .is_err())
    }
}
