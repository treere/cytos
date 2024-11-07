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

use super::graph::{Graph, GraphRepr};

use super::{GraphId, NodeId, ParamId, Result, Value};

use std::collections::HashMap;
use std::thread::{Builder, JoinHandle};

/// SystemRepr
///
/// Deserializable System Representation
#[derive(Deserialize, Debug)]
pub struct SystemRepr {
    /// Graphs by name
    #[serde(default)]
    graphs: HashMap<GraphId, GraphRepr>,

    /// Links between graphs
    links: Vec<Link>,
}

impl SystemRepr {
    /// Create a SystemRepr loading a file
    pub fn from_json(file: &str) -> Result<Self> {
        serde_json::from_str(file).map_err(|v| format!("cannot read file: {v}").into())
    }

    /// Convert a system representation into a System
    pub fn to_system(self, loader: &Registry) -> Result<System> {
        let (graphs, senders): (HashMap<_, _>, HashMap<_, _>) = self
            .graphs
            .into_iter()
            .map(|(graph_id, graph_repr)| {
                let (sender, receiver) = unbounded::<(Command, Message)>();
                ((graph_id, (graph_repr, receiver)), (graph_id, sender))
            })
            .unzip();

        let runners = graphs
            .into_iter()
            .map(|(graph_id, (graph_repr, receiver))| {
                load_runner(
                    graph_id,
                    graph_repr,
                    receiver,
                    loader.clone(),
                    senders.clone(),
                    &self.links,
                )
            })
            .collect::<Result<IndexMap<_, _>>>()?;

        Ok(System { runners })
    }
}

/// Link between params of different graphs
#[derive(Deserialize, Debug, Clone)]
struct Link {
    /// Source node
    src: (GraphId, NodeId, ParamId),

    /// Destination node
    dst: (GraphId, NodeId, ParamId),
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

    /// Iterator on graph names
    pub fn keys(&self) -> impl Iterator<Item = &GraphId> {
        self.runners.keys()
    }
}

fn load_runner(
    id: GraphId,
    graph_repr: GraphRepr,
    receiver: Receiver<(Command, Message)>,
    loader: Registry,
    senders: HashMap<GraphId, Sender<(Command, Message)>>,
    links: &Vec<Link>,
) -> Result<(GraphId, Runner)> {
    let sender = senders.get(&id).ok_or("missing sender")?.clone();

    let runner = Runner::try_from_repr(id, graph_repr, loader, (sender, receiver), senders, links)
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

/// Message is a letter that contains an sender which can be used to set a response
#[derive(Clone)]
pub struct Message {
    sender: Sender<ResponseResult>,
}

impl Message {
    /// Creates a message and returns the received where it is possible to listen to the response
    fn new() -> (Self, Receiver<ResponseResult>) {
        let (sender, receiver) = bounded::<ResponseResult>(0);

        (Self { sender }, receiver)
    }

    /// Set the response and consume the message
    fn set<T: Serialize>(self, resp: Result<T>) {
        let resp = resp
            .and_then(|v| Value::load(&v))
            .map_err(|r| r.to_string());
        self.sender.send(resp).expect("cannot send");
    }
}

/// External address
type ExternalReference = (Sender<(Command, Message)>, Command);

/// Internal address
type InternalReference = (NodeId, ParamId);

/// The runner worker
struct InternalRunner {
    /// The graph
    graph: Graph,
    /// A receiver for the commands
    receiver: Receiver<(Command, Message)>,
    /// External links
    external: Vec<(ExternalReference, InternalReference)>,
}

impl InternalRunner {
    /// Run the internal runner
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

    /// Dispatch a command to the graph
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

/// Runner that wraps the internal runner
struct Runner {
    /// Thread with the internal runner inside
    thread: Option<JoinHandle<()>>,
    /// Sender to the internal runner
    sender: Sender<(Command, Message)>,
}

type ChannelTuple = (Sender<(Command, Message)>, Receiver<(Command, Message)>);

impl Runner {
    /// Creates a runner from a graph repr
    fn try_from_repr(
        name: GraphId,
        repr: GraphRepr,
        reg: Registry,
        (sender, receiver): ChannelTuple,
        senders: HashMap<GraphId, Sender<(Command, Message)>>,
        links: &Vec<Link>,
    ) -> Result<Self> {
        let links: Vec<_> = links.iter().filter(|l| l.dst.0 == name).cloned().collect();

        let thread = Builder::new()
            .name(name.to_string())
            .spawn(move || {
                let external = links
                    .into_iter()
                    .map(|x| {
                        let (g, n, p) = x.src;
                        let (_, nd, pd) = x.dst;

                        ((senders[&g].clone(), Command::Dump(n, p)), (nd, pd))
                    })
                    .collect::<Vec<_>>();

                let graph = repr.to_graph(&reg).expect("Cannot build graph");

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

    /// Send a command to the internal runner
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
