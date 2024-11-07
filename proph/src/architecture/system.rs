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
    pub fn to_system(self, registry: &Registry) -> Result<System> {
        let (graphs, senders): (IndexMap<_, _>, IndexMap<_, _>) = self
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
                Self::create_runner(
                    graph_id,
                    graph_repr,
                    receiver,
                    senders.clone(),
                    registry.clone(),
                    &self.links,
                )
            })
            .collect::<Result<IndexMap<_, _>>>()?;

        Ok(System { runners })
    }

    fn create_runner(
        graph_id: GraphId,
        graph_repr: GraphRepr,
        receiver: Receiver<(Command, Message)>,
        senders: IndexMap<GraphId, Sender<(Command, Message)>>,
        registry: Registry,
        links: &[Link],
    ) -> Result<(GraphId, Runner)> {
        let sender = senders.get(&graph_id).ok_or("missing sender")?.clone();

        let mut links: Vec<_> = links
            .iter()
            .filter(|l| l.dst.0 == graph_id)
            .cloned()
            .collect();

        links.sort_by_key(|x| x.src.0);

        let links = links[..]
            .chunk_by(|a, b| a.src.0 == b.src.0)
            .map(|links| {
                let (commands, destinations): (Vec<Command>, Vec<(NodeId, ParamId)>) = links
                    .iter()
                    .map(|link| {
                        (
                            Command::Dump(link.src.1, link.src.2),
                            (link.dst.1, link.dst.2),
                        )
                    })
                    .unzip();
                let g = links[0].src.0;

                senders
                    .get(&g)
                    .cloned()
                    .map(|sender| ((sender, Command::Multi(commands)), destinations))
            })
            .collect::<Option<Vec<_>>>()
            .ok_or("missin sender")?;

        let thread = Builder::new()
            .name(graph_id.to_string())
            .spawn(move || {
                let graph = graph_repr
                    .into_graph(&registry)
                    .expect("Cannot build graph");

                InternalRunner {
                    graph,
                    receiver,
                    links,
                }
                .run();
            })
            .or(Err("cannot run thread"))?;

        Ok((
            graph_id,
            Runner {
                thread: Some(thread),
                sender,
            },
        ))
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
    pub fn graphs(&self) -> impl Iterator<Item = &GraphId> {
        self.runners.keys()
    }
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
    /// Multi command. Kill, Start, Stop and Status are not processed
    Multi(Vec<Command>),
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
    fn prepare_and_set<T: Serialize>(self, resp: Result<T>) {
        let resp = Self::prepare(resp);
        self.sender.send(resp).expect("cannot send");
    }

    fn set(self, resp: ResponseResult) {
        self.sender.send(resp).expect("cannot send");
    }

    fn prepare<T: Serialize>(resp: Result<T>) -> ResponseResult {
        resp.and_then(|v| Value::load(&v))
            .map_err(|r| r.to_string())
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
    /// Links between graphs
    links: Vec<(ExternalReference, Vec<InternalReference>)>,
}

impl InternalRunner {
    /// Run the internal runner
    fn run(mut self) {
        'main: loop {
            while let Ok((command, message)) = self.receiver.recv() {
                match command {
                    Command::Kill => break 'main,
                    Command::Start => break,
                    Command::Status => message.prepare_and_set(Ok("Idle")),
                    command => {
                        if let Some(result) = self.dispatch_command(command) {
                            message.set(result);
                        }
                    }
                }
            }

            self.graph.initialize().expect("cannot initialize");
            'outer: loop {
                while let Ok((command, message)) = self.receiver.try_recv() {
                    match command {
                        Command::Kill => break 'main,
                        Command::Stop => break 'outer,
                        Command::Status => message.prepare_and_set(Ok("Running")),
                        command => {
                            if let Some(result) = self.dispatch_command(command) {
                                message.set(result);
                            }
                        }
                    }
                }

                let (message, receiver) = Message::new();
                for ((sender, command), internals) in &self.links {
                    let response: Vec<Value> =
                        match sender.send((command.clone(), message.clone())) {
                            Ok(()) => receiver.recv().unwrap(),
                            Err(_) => Err("Cannot send".into()),
                        }
                        .map(|r| r.dump().unwrap())
                        .unwrap();

                    internals
                        .iter()
                        .zip(response)
                        .for_each(|(internal, response)| {
                            self.graph.load(*internal, response).unwrap()
                        })
                }

                if self.graph.step().is_err() {
                    break 'outer;
                }
            }
            self.graph.terminate().expect("cannot terminate");
        }
    }

    /// Dispatch a command to the graph
    fn dispatch_command(&mut self, command: Command) -> Option<ResponseResult> {
        match command {
            Command::ListNodes => Some(Message::prepare(Ok(self.graph.list_nodes()))),
            Command::ListInputs(node) => Some(Message::prepare(self.graph.list_node_inputs(node))),
            Command::ListOutputs(node) => {
                Some(Message::prepare(self.graph.list_node_outputs(node)))
            }
            Command::Dump(node, param) => {
                Some(Message::prepare(self.graph.dumper_for((node, param))))
            }
            Command::Load(node, param, value) => {
                Some(Message::prepare(self.graph.load((node, param), value)))
            }

            Command::Kill | Command::Start | Command::Stop | Command::Status => None,
            Command::Multi(vec) => Some(Message::prepare(Ok(vec
                .into_iter()
                .map(|c| self.dispatch_command(c))
                .collect::<Vec<_>>()))),
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

impl Runner {
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
