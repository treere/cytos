//! System module
//!
//! A system is a set of graph linked together

use crossbeam::channel::bounded;
use crossbeam::channel::unbounded;
use crossbeam::channel::Receiver;
use crossbeam::channel::Sender;
use indexmap::IndexMap;
use serde::Deserialize;
use serde::Serialize;

use crate::loader::Registry;

use super::graph::{Graph, GraphRepr, LinkSource};

use super::{GraphId, NodeId, ParamId, Result, Value};

use std::collections::HashMap;
use std::thread::{Builder, JoinHandle};

/// SystemRepr
///
/// Deserializable System Representation
#[derive(Deserialize, Debug)]
pub struct SystemRepr {
    /// Graphs in the repr
    #[serde(default)]
    graphs: Vec<GraphRepr>,
}

impl SystemRepr {
    /// Create a SystemRepr loading a file
    pub fn from_json(file: &str) -> Result<Self> {
        serde_json::from_str(file).map_err(|v| format!("cannot read file: {v}").into())
    }
}

/// System
#[derive(Default)]
pub struct System {
    /// Runners where there is a runner per graph
    runners: IndexMap<GraphId, Runner>,
}

impl System {
    /// Send a command to a runner
    pub fn command(&mut self, graph: GraphId, command: Command) -> Result<Value> {
        self.runners
            .get_mut(&graph)
            .ok_or("not found")?
            .command(command)
    }

    /// Convert a system representation into a System
    pub fn from_repr(repr: SystemRepr, loader: &Registry) -> Result<Self> {
        let channels: HashMap<_, _> = repr
            .graphs
            .iter()
            .map(|graph| {
                let (sender, receiver) = unbounded::<(Command, Message)>();
                (graph.name, (sender, Some(receiver)))
            })
            .collect();

        let senders: HashMap<_, _> = channels
            .iter()
            .map(|(graph_id, (sender, _))| (*graph_id, sender.clone()))
            .collect();

        let mut receivers: HashMap<_, _> = channels
            .into_iter()
            .map(|(graph_id, (_, receiver))| (graph_id, receiver))
            .collect();

        let runners = repr
            .graphs
            .into_iter()
            .map(|graph_repr| {
                load_runner(graph_repr, &mut receivers, loader.clone(), senders.clone())
            })
            .collect::<Result<IndexMap<_, _>>>()?;

        Ok(Self { runners })
    }

    /// Iterator on graph names
    pub fn keys(&self) -> impl Iterator<Item = &GraphId> {
        self.runners.keys()
    }
}

fn load_runner(
    graph_repr: GraphRepr,
    receivers: &mut HashMap<GraphId, Option<Receiver<(Command, Message)>>>,
    loader: Registry,
    senders: HashMap<GraphId, Sender<(Command, Message)>>,
) -> Result<(GraphId, Runner)> {
    let id = graph_repr.name;

    let receiver = receivers.get_mut(&id).ok_or("missing channel")?;

    let receiver = receiver.take().ok_or("missing receiver")?;

    let sender = senders.get(&id).ok_or("missing sender")?.clone();

    let runner = Runner::try_from_repr(id, graph_repr, loader, (sender, receiver), senders)
        .or(Err("cannot create runner"))?;

    Ok((id, runner))
}

/// Commands that a runner can send
#[derive(Debug, Clone)]
pub enum Command {
    /// Kill the runner
    Kill,
    /// Start the runner
    Start,
    /// Stop the runner
    Stop,
    /// Receive the runner status
    Status,
    /// List the nodes of the graph inside the runner
    ListNodes,
    /// List the inputs of a node
    ListInputs(NodeId),
    /// List the outputs of a node
    ListOutputs(NodeId),
    /// Dump the value of a node
    Dump(NodeId, ParamId),
    /// Load a value into a node
    Load(NodeId, ParamId, Value),
}

type ResponseResult = std::result::Result<Value, String>;

#[derive(Clone)]
pub struct Message {
    sender: Sender<ResponseResult>,
}

impl Message {
    fn new() -> (Self, Receiver<ResponseResult>) {
        let (sender, receiver) = bounded::<ResponseResult>(0);

        (Self { sender }, receiver)
    }

    fn set<T: Serialize>(self, resp: Result<T>) {
        let resp = resp
            .and_then(|v| Value::load(&v))
            .map_err(|r| r.to_string());
        self.sender.send(resp).expect("cannot send");
    }
}

type ExternalReference = (Sender<(Command, Message)>, Command);

type InternalReference = (NodeId, ParamId);

struct InternalRunner {
    graph: Graph,
    receiver: Receiver<(Command, Message)>,
    external: Vec<(ExternalReference, InternalReference)>,
}

impl InternalRunner {
    fn run(mut self) {
        'main: loop {
            while let Ok((command, message)) = self.receiver.recv() {
                match command {
                    Command::Kill => break 'main,
                    Command::Start => break,
                    Command::Status => message.set(Ok("Idle")),
                    _ => self.dispatch_command(command, message),
                }
            }

            self.graph.initialize().expect("cannot initialize");
            'outer: loop {
                while let Ok((command, message)) = self.receiver.try_recv() {
                    match command {
                        Command::Kill => break 'main,
                        Command::Stop => break 'outer,
                        Command::Status => message.set(Ok("Running")),
                        _ => self.dispatch_command(command, message),
                    }
                }

                let (message, receiver) = Message::new();
                for ((sender, command), internal) in &self.external {
                    let response = match sender.send((command.clone(), message.clone())) {
                        Ok(()) => receiver.recv().unwrap(),
                        Err(_) => Err("Cannot send".into()),
                    }
                    .unwrap();

                    self.graph.load(*internal, response).unwrap();
                }

                if self.graph.step().is_err() {
                    break 'outer;
                }
            }
            self.graph.terminate().expect("cannot terminate");
        }
    }

    fn dispatch_command(&mut self, command: Command, message: Message) {
        match command {
            Command::ListNodes => message.set(Ok(self.graph.list_nodes())),
            Command::ListInputs(node) => message.set(self.graph.list_node_inputs(node)),
            Command::ListOutputs(node) => message.set(self.graph.list_node_outputs(node)),
            Command::Dump(node, param) => message.set(self.graph.dumper_for((node, param))),
            Command::Load(node, param, value) => {
                message.set(self.graph.load((node, param), value));
            }
            _ => (),
        }
    }
}

struct Runner {
    thread: Option<JoinHandle<()>>,
    sender: Sender<(Command, Message)>,
}

type ChannelTuple = (Sender<(Command, Message)>, Receiver<(Command, Message)>);

impl Runner {
    fn try_from_repr(
        name: GraphId,
        mut repr: GraphRepr,
        reg: Registry,
        (sender, receiver): ChannelTuple,
        senders: HashMap<GraphId, Sender<(Command, Message)>>,
    ) -> Result<Self> {
        let thread = Builder::new()
            .name(name.to_string())
            .spawn(move || {
                let external = repr
                    .links
                    .iter()
                    .filter(|x| matches!(x.src, LinkSource::External(_, _, _)))
                    .map(|x| match &x.src {
                        LinkSource::Internal(_, _) => unreachable!(),
                        LinkSource::External(g, n, p) => {
                            ((senders[g].clone(), Command::Dump(*n, *p)), x.dst)
                        }
                    })
                    .collect::<Vec<_>>();

                repr.links
                    .retain(|x| matches!(x.src, LinkSource::Internal(_, _)));
                let (_, graph) = repr.to_graph(&reg).expect("Cannot build graph");

                InternalRunner {
                    graph,
                    receiver,
                    external,
                }
                .run();
            })
            .or(Err("cannot run thread"))?;

        Ok(Self {
            thread: Some(thread),
            sender,
        })
    }

    pub fn command(&mut self, command: Command) -> Result<Value> {
        let (message, receiver) = Message::new();

        match self.sender.send((command, message)) {
            Ok(()) => receiver
                .recv()
                .unwrap_or(Ok(Value::load(&()).unwrap()))
                .map_err(Into::into),
            Err(_) => Err("Cannot send".into()),
        }
    }
}

impl Drop for Runner {
    fn drop(&mut self) {
        if let Some(t) = self.thread.take() {
            let (message, _receiver) = Message::new();
            self.sender
                .send((Command::Kill, message))
                .expect("cannot send");

            t.join().expect("cannot join");
        }
    }
}
