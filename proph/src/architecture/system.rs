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

use super::graph::StepResult;
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

    /// Request between graphs
    requests: Vec<Link>,
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
                    &self.requests,
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
        requests: &[Link],
    ) -> Result<(GraphId, Runner)> {
        let sender = senders.get(&graph_id).ok_or("missing sender")?.clone();

        let requests = Self::create_requests(graph_id, senders, requests)?;

        let thread = Builder::new()
            .name(graph_id.to_string())
            .spawn(move || {
                let graph = graph_repr
                    .into_graph(&registry)
                    .expect("Cannot build graph");

                InternalRunner {
                    graph,
                    receiver,
                    requests,
                    queue: Vec::default(),
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

    fn create_requests(
        graph_id: GraphId,
        senders: IndexMap<GraphId, Sender<(Command, Message)>>,
        requests: &[Link],
    ) -> Result<Vec<(ExternalReference, Vec<InternalReference>)>> {
        let mut requests: Vec<_> = requests.iter().filter(|l| l.dst.0 == graph_id).collect();

        requests.sort_by_key(|x| x.src.0);

        requests[..]
            .chunk_by(|a, b| a.src.0 == b.src.0)
            .map(|requests| {
                let g = requests[0].src.0;
                let (sources, destinations) = requests
                    .iter()
                    .map(|request| {
                        (
                            (request.src.1, request.src.2),
                            (request.dst.1, request.dst.2),
                        )
                    })
                    .unzip();

                senders
                    .get(&g)
                    .cloned()
                    .map(|sender| ((sender, Command::MultiDump(sources)), destinations))
            })
            .collect::<Option<Vec<_>>>()
            .ok_or("missin sender".into())
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
    /// Multi dump command
    MultiDump(Vec<(NodeId, ParamId)>),
    /// Multi load command
    MultiLoad(Vec<(NodeId, ParamId, Value)>),
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
    /// Requests between graphs
    requests: Vec<(ExternalReference, Vec<InternalReference>)>,
    /// Queue
    queue: Vec<(Command, Message)>,
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
                    Command::Stop => (),
                    command => self.dispatch_command(command, message, &StepResult::Done),
                }
            }

            self.graph.initialize().expect("cannot initialize");
            'outer: loop {
                let (message, receiver) = Message::new();
                for ((sender, command), internals) in &self.requests {
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

                if let Ok(cause) = self.graph.step() {
                    while let Ok((command, message)) = self.receiver.try_recv() {
                        match command {
                            Command::Kill => break 'main,
                            Command::Stop => break 'outer,
                            Command::Status => message.prepare_and_set(Ok("Running")),
                            Command::Start => (),
                            command => self.dispatch_command(command, message, &cause),
                        }
                    }
                } else {
                    break 'outer;
                }
            }
            self.graph.terminate().expect("cannot terminate");
        }
    }

    /// Dispatch a command to the graph
    fn dispatch_command(&mut self, command: Command, message: Message, cause: &StepResult) {
        match cause {
            StepResult::Done => {
                let mut queue = vec![];
                std::mem::swap(&mut self.queue, &mut queue);
                queue
                    .into_iter()
                    .for_each(|(command, message)| self.done_dispatch_command(message, command));
                self.done_dispatch_command(message, command)
            }
            StepResult::Skip => self.skip_dispatch_command(message, command),
        }
    }

    fn done_dispatch_command(&mut self, message: Message, command: Command) {
        match command {
            Command::ListNodes => message.set(Message::prepare(Ok(self.graph.list_nodes()))),
            Command::ListInputs(node) => {
                message.set(Message::prepare(self.graph.list_node_inputs(node)))
            }
            Command::ListOutputs(node) => {
                message.set(Message::prepare(self.graph.list_node_outputs(node)))
            }
            Command::MultiDump(vec) => {
                let p: Result<Vec<_>> = vec.into_iter().map(|c| self.graph.dumper_for(c)).collect();
                message.set(Message::prepare(p))
            }
            Command::MultiLoad(vec) => {
                let p: Result<Vec<_>> = vec
                    .into_iter()
                    .map(|(n, p, v)| self.graph.load((n, p), v))
                    .collect();
                message.set(Message::prepare(p))
            }
            Command::Kill | Command::Start | Command::Stop | Command::Status => unreachable!(),
        }
    }

    fn skip_dispatch_command(&mut self, message: Message, command: Command) {
        match command {
            Command::ListNodes => message.set(Message::prepare(Ok(self.graph.list_nodes()))),
            Command::ListInputs(node) => {
                message.set(Message::prepare(self.graph.list_node_inputs(node)))
            }
            Command::ListOutputs(node) => {
                message.set(Message::prepare(self.graph.list_node_outputs(node)))
            }
            Command::MultiDump(_) => {
                self.queue.push((command, message));
            }
            Command::MultiLoad(vec) => {
                let p: Result<Vec<_>> = vec
                    .into_iter()
                    .map(|(n, p, v)| self.graph.load((n, p), v))
                    .collect();
                message.set(Message::prepare(p))
            }
            Command::Kill | Command::Start | Command::Stop | Command::Status => unreachable!(),
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
