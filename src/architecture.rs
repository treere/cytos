use std::{collections::HashMap, mem};

#[derive(Debug)]
pub enum Data {
    U8(u8),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Path {
    node: String,
    field: String,
}

impl Path {
    pub fn new(node: impl ToString, field: impl ToString) -> Self {
        Path {
            node: node.to_string(),
            field: field.to_string(),
        }
    }
}

#[derive(Debug)]
struct Board {
    data: Vec<(Path, Data)>,
}

impl Board {
    fn new() -> Self {
        Board { data: Vec::new() }
    }

    fn contains(&self, path: &Path) -> bool {
        self.data.iter().any(|r| &r.0 == path)
    }

    fn merge(&mut self, data: impl IntoIterator<Item = (Path, Data)>) {
        self.data.extend(data)
    }

    fn clear(&mut self) {
        self.data.clear()
    }

    fn get_by_src(&self, src: &Path) -> &Data {
        let (_, d) = self.data.iter().filter(|r| &r.0 == src).next().unwrap();
        d
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

    fn process(&self, val: &HashMap<&str, &Data>) -> Result<HashMap<&str, Data>, ()> {
        self.fun.process(val)
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

pub struct Orchestrator {
    nodes: Vec<Processor>,
    links: Links,
    board: Board,
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            links: Links::new(),
            board: Board::new(),
        }
    }

    pub fn add(
        mut self,
        name: impl ToString,
        processor: impl Transformer + 'static,
    ) -> Result<Self, ()> {
        let name = name.to_string();
        if !self.nodes.iter().any(|n| n.name == name) {
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

        self.links.push(Link::new(src, dst));

        Ok(self)
    }

    pub fn step(&mut self) -> Result<(), ()> {
        let mut nodes = mem::take(&mut self.nodes);
        self.nodes.reserve(nodes.len());

        self.board.clear();

        while !nodes.is_empty() {
            if let Some(node) = self
                .index_first_ready_node(&nodes, &self.board)
                .map(|index| nodes.remove(index))
            {
                let params: HashMap<_, _> = self
                    .links
                    .iter_by_dst(&node.name)
                    .map(|r| (&r.dst.field[..], self.board.get_by_src(&r.src)))
                    .collect();

                if let Ok(data) = node.process(&params) {
                    self.board.merge(
                        data.into_iter()
                            .map(|r| (Path::new(node.name.clone(), r.0), r.1)),
                    );
                }
                self.nodes.push(node);
            } else {
                return Err(());
            }
        }

        Ok(())
    }

    fn index_first_ready_node(&self, nodes: &[Processor], board: &Board) -> Option<usize> {
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

pub trait Transformer {
    fn inputs(&self) -> &[&str];
    fn outputs(&self) -> &[&str];

    fn process(&self, val: &HashMap<&str, &Data>) -> Result<HashMap<&str, Data>, ()>;
}
