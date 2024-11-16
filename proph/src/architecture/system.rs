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

use super::GenericOwnedProp;
use super::{GraphId, NodeId, ParamId, Result, Value};

use std::collections::HashMap;
use std::thread::{Builder, JoinHandle};

type InternalCommand = (Command, Response);

/// SystemRepr
///
/// Deserializable System Representation
#[derive(Deserialize, Debug)]
pub struct SystemRepr {
    /// Graphs by name
    #[serde(default)]
    graphs: HashMap<GraphId, GraphRepr>,

    #[serde(default)]
    /// Request between graphs
    requests: Vec<Link>,

    #[serde(default)]
    /// Send between graphs
    sends: Vec<Link>,
}

impl SystemRepr {
    /// Create a SystemRepr loading a file
    pub fn from_json(file: &str) -> Result<Self> {
        serde_json::from_str(file).map_err(|v| format!("cannot read file: {v}").into())
    }

    /// Convert a system representation into a System
    pub fn to_system(self, registry: &Registry) -> Result<System> {
        // From the map of id, reprs create a map of ip, repr and receiver
        // and a map of id, sender.
        let (graphs, senders): (IndexMap<_, _>, IndexMap<_, _>) = self
            .graphs
            .into_iter()
            .map(|(graph_id, graph_repr)| {
                let (sender, receiver) = unbounded::<InternalCommand>();
                ((graph_id, (graph_repr, receiver)), (graph_id, sender))
            })
            .unzip();

        let runners = graphs
            .into_iter()
            .map(|(id, (repr, receiver))| {
                let sender = senders.get(&id).ok_or("missing sender")?.clone();

                Self::create_runner(
                    id,
                    repr,
                    receiver,
                    senders.clone(),
                    registry.clone(),
                    &self.requests,
                    &self.sends,
                )
                .map(|thread| {
                    (
                        id,
                        Runner {
                            thread: Some(thread),
                            sender,
                        },
                    )
                })
            })
            .collect::<Result<IndexMap<_, _>>>()?;

        Ok(System { runners })
    }

    fn create_runner(
        id: GraphId,
        repr: GraphRepr,
        receiver: Receiver<InternalCommand>,
        senders: IndexMap<GraphId, Sender<InternalCommand>>,
        registry: Registry,
        requests: &[Link],
        sends: &[Link],
    ) -> Result<JoinHandle<()>> {
        let requests = Self::create_requests(id, &senders, requests)?;
        let sends = Self::create_sends(id, &senders, sends)?;

        Builder::new()
            .name(id.to_string())
            .spawn(move || {
                let graph = repr.into_graph(&registry).expect("Cannot build graph");

                InternalRunner {
                    graph,
                    receiver,
                    requests,
                    sends,
                    queue: Vec::default(),
                }
                .run();
            })
            .or(Err("cannot run thread".into()))
    }

    fn create_sends(
        graph_id: GraphId,
        senders: &IndexMap<GraphId, Sender<InternalCommand>>,
        sends: &[Link],
    ) -> Result<Vec<(ExternalDestination, Vec<Destination>)>> {
        let mut requests: Vec<_> = sends.iter().filter(|l| l.src.0 == graph_id).collect();

        requests.sort_by_key(|x| x.dst.0);

        requests[..]
            .chunk_by(|a, b| a.dst.0 == b.dst.0)
            .map(|requests| {
                let g = requests[0].dst.0;
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
                    .map(|sender| ((sender, destinations), sources))
            })
            .collect::<Option<Vec<_>>>()
            .ok_or("missin sender".into())
    }

    fn create_requests(
        id: GraphId,
        senders: &IndexMap<GraphId, Sender<InternalCommand>>,
        requests: &[Link],
    ) -> Result<Vec<(ExternalDestination, Vec<Destination>)>> {
        let mut requests: Vec<_> = requests.iter().filter(|l| l.dst.0 == id).collect();

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
                    .map(|sender| ((sender, sources), destinations))
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
    pub fn command(&self, graph: GraphId, command: Command) -> Result<Value> {
        self.internal_command(graph, command).and_then(|r| match r {
            Internal::Value(value) => Ok(value),
            Internal::Prop(_generic_owned_prop) => Err("cannot return owned".into()),
        })
    }

    /// Send a command to a runner
    fn internal_command(&self, graph: GraphId, command: Command) -> Result<Internal> {
        self.runners
            .get(&graph)
            .ok_or("not found")?
            .command(command)
    }

    /// Iterator on graph names
    pub fn graphs(&self) -> impl Iterator<Item = &GraphId> {
        self.runners.keys()
    }
}

/// Commands that a runner can send
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
    /// Multi owned dump command
    MultiOwnedDump(Vec<(NodeId, ParamId)>),
    /// Multi load command
    MultiOwnedLoad(Vec<(NodeId, ParamId, GenericOwnedProp)>),
}

enum Internal {
    Value(Value),
    Prop(Vec<GenericOwnedProp>),
}

impl std::fmt::Debug for Internal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Internal::Value(value) => write!(f, "{:?}", value),
            Internal::Prop(_generic_owned_prop) => write!(f, "GenericOwnedProp"),
        }
    }
}

type ResponseResult = std::result::Result<Internal, String>;

/// Reponse is a letter that contains an sender which can be used to set a response
#[derive(Clone)]
pub struct Response {
    sender: Sender<ResponseResult>,
}

impl Response {
    /// Creates a message and returns the received where it is possible to listen to the response
    fn new() -> (Self, Receiver<ResponseResult>) {
        let (sender, receiver) = bounded::<ResponseResult>(0);

        (Self { sender }, receiver)
    }

    /// Set the response and consume the message
    fn send_value<T: Serialize>(self, resp: Result<T>) {
        let resp = resp
            .and_then(|v| Value::load(&v))
            .map(Internal::Value)
            .map_err(|r| r.to_string());

        self.sender.send(resp).expect("cannot send");
    }

    fn send_prop(self, resp: Result<Vec<GenericOwnedProp>>) {
        let resp = resp.map(Internal::Prop).map_err(|r| r.to_string());

        self.sender.send(resp).expect("cannot send");
    }
}

/// Internal address
type Destination = (NodeId, ParamId);

/// External address
type ExternalDestination = (Sender<InternalCommand>, Vec<Destination>);

/// The runner worker
struct InternalRunner {
    /// The graph
    graph: Graph,
    /// A receiver for the commands
    receiver: Receiver<InternalCommand>,
    /// Requests between graphs
    requests: Vec<(ExternalDestination, Vec<Destination>)>,
    /// Sends between graphs
    sends: Vec<(ExternalDestination, Vec<Destination>)>,
    /// Queue
    queue: Vec<InternalCommand>,
}

impl InternalRunner {
    /// Run the internal runner
    fn run(mut self) {
        'main: loop {
            while let Ok((command, message)) = self.receiver.recv() {
                match command {
                    Command::Kill => break 'main,
                    Command::Start => break,
                    Command::Status => message.send_value(Ok("Idle")),
                    Command::Stop => (),
                    command => self.dispatch_command(command, message, &StepResult::Done),
                }
            }

            self.graph.initialize().expect("cannot initialize");
            'outer: loop {
                self.request_values().expect("cannot request");

                if let Ok(cause) = self.graph.step() {
                    self.send_values().expect("cannot send");
                    while let Ok((command, message)) = self.receiver.try_recv() {
                        match command {
                            Command::Kill => break 'main,
                            Command::Stop => break 'outer,
                            Command::Status => message.send_value(Ok("Running")),
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
    fn dispatch_command(&mut self, command: Command, message: Response, cause: &StepResult) {
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

    fn done_dispatch_command(&mut self, message: Response, command: Command) {
        match command {
            Command::ListNodes => message.send_value(Ok(self.graph.list_nodes())),
            Command::ListInputs(node) => message.send_value(self.graph.list_node_inputs(node)),
            Command::ListOutputs(node) => message.send_value(self.graph.list_node_outputs(node)),
            Command::MultiDump(vec) => {
                let dump: Result<Vec<_>> = vec.into_iter().map(|c| self.graph.dump(c)).collect();
                message.send_value(dump)
            }
            Command::MultiLoad(vec) => {
                let p: Result<Vec<_>> = vec
                    .into_iter()
                    .map(|(n, p, v)| self.graph.load((n, p), v))
                    .collect();
                message.send_value(p)
            }
            Command::Kill | Command::Start | Command::Stop | Command::Status => unreachable!(),
            Command::MultiOwnedDump(vec) => {
                let p: Result<Vec<_>> = vec.into_iter().map(|c| self.graph.dump_owned(c)).collect();
                message.send_prop(p)
            }
            Command::MultiOwnedLoad(vec) => {
                let p: Result<Vec<_>> = vec
                    .into_iter()
                    .map(|(n, p, v)| self.graph.load_owned((n, p), v))
                    .collect();
                message.send_value(p)
            }
        }
    }

    fn skip_dispatch_command(&mut self, message: Response, command: Command) {
        match command {
            Command::ListNodes => message.send_value(Ok(self.graph.list_nodes())),
            Command::ListInputs(node) => message.send_value(self.graph.list_node_inputs(node)),
            Command::ListOutputs(node) => message.send_value(self.graph.list_node_outputs(node)),
            Command::MultiDump(_) => {
                self.queue.push((command, message));
            }
            Command::MultiLoad(vec) => {
                let p: Result<Vec<_>> = vec
                    .into_iter()
                    .map(|(n, p, v)| self.graph.load((n, p), v))
                    .collect();
                message.send_value(p)
            }
            Command::Kill | Command::Start | Command::Stop | Command::Status => unreachable!(),
            Command::MultiOwnedDump(_) => {
                self.queue.push((command, message));
            }
            Command::MultiOwnedLoad(vec) => {
                let loads: Result<Vec<_>> = vec
                    .into_iter()
                    .map(|(n, p, v)| self.graph.load_owned((n, p), v))
                    .collect();
                message.send_value(loads)
            }
        }
    }

    fn request_values(&mut self) -> Result<()> {
        let (message, receiver) = Response::new();
        for ((sender, nodes), internals) in &self.requests {
            let response: Vec<_> =
                match sender.send((Command::MultiOwnedDump(nodes.clone()), message.clone())) {
                    Ok(()) => receiver.recv()?,
                    Err(_) => Err("Cannot send".into()),
                }
                .map(|r| match r {
                    Internal::Value(_) => unreachable!(),
                    Internal::Prop(v) => v,
                })?;

            for (internal, response) in internals.iter().zip(response) {
                self.graph.load_owned(*internal, response)?;
            }
        }
        Ok(())
    }

    fn send_values(&mut self) -> Result<()> {
        let (message, _receiver) = Response::new();

        for ((sender, external_nodes), internal_props) in &self.sends {
            let loads = internal_props
                .iter()
                .map(|node| self.graph.dump_owned(*node))
                .zip(external_nodes)
                .map(|(v, (n, p))| v.map(|val| (*n, *p, val)))
                .collect::<Result<Vec<_>>>()?;
            sender.send((Command::MultiOwnedLoad(loads), message.clone()))?;
        }
        Ok(())
    }
}

/// Runner that wraps the internal runner
struct Runner {
    /// Thread with the internal runner inside
    thread: Option<JoinHandle<()>>,
    /// Sender to the internal runner
    sender: Sender<InternalCommand>,
}

impl Runner {
    /// Send a command to the internal runner
    pub fn command(&self, command: Command) -> Result<Internal> {
        let (message, receiver) = Response::new();

        match self.sender.send((command, message)) {
            Ok(()) => receiver
                .recv()
                .unwrap_or(Ok(Internal::Value(Value::load(&())?)))
                .map_err(Into::into),
            Err(_) => Err("Cannot send".into()),
        }
    }
}

impl Drop for Runner {
    fn drop(&mut self) {
        if let Some(t) = self.thread.take() {
            let (message, _receiver) = Response::new();
            self.sender
                .send((Command::Kill, message))
                .expect("cannot send");

            t.join().expect("cannot join");
        }
    }
}
