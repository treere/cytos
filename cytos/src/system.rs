//! System module
//!
//! A system is a set of graph linked together

use crossbeam::channel::bounded;
use crossbeam::channel::unbounded;
use crossbeam::channel::Receiver;
use crossbeam::channel::Sender;
use indexmap::IndexMap;

use serde::Serialize;

use crate::loader::Registry;
use crate::repr::GraphRepr;
use crate::repr::SystemLink;
use crate::repr::SystemRepr;

use super::graph::Graph;
use super::graph::StepResult;

use super::GenericOwnedProp;
use super::{GraphId, NodeId, ParamId, Result, Value};

use std::thread::{Builder, JoinHandle};

impl SystemRepr {
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
        requests: &[SystemLink],
        sends: &[SystemLink],
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
        sends: &[SystemLink],
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
        requests: &[SystemLink],
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

/// System
#[derive(Default)]
pub struct System {
    /// Runners where there is a runner per graph
    runners: IndexMap<GraphId, Runner>,
}

impl System {
    /// Iterator on graph names
    pub fn graphs(&self) -> impl Iterator<Item = &GraphId> {
        self.runners.keys()
    }

    pub fn graph(&self, graph: GraphId) -> Result<GraphView<'_>> {
        let r = self.runners.get(&graph).ok_or("not found")?;
        Ok(GraphView { r })
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
    fn command(&self, command: Command) -> Result<Internal> {
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
            Command::MultiDump(vec) => {
                let dump: Result<Vec<_>> = vec
                    .into_iter()
                    .map(|(node, param)| self.graph.get_node(node).and_then(|n| n.dump(param)))
                    .collect();
                message.send_value(dump)
            }
            Command::MultiOwnedDump(vec) => {
                let dump: Result<Vec<_>> = vec
                    .into_iter()
                    .map(|(node, param)| {
                        self.graph.get_node(node).and_then(|n| n.dump_owned(param))
                    })
                    .collect();

                message.send_prop(dump)
            }
            command => self.common_dispatch_command(message, command),
        }
    }

    fn skip_dispatch_command(&mut self, message: Response, command: Command) {
        match command {
            Command::MultiDump(_) => {
                self.queue.push((command, message));
            }
            Command::MultiOwnedDump(_) => {
                self.queue.push((command, message));
            }
            command => self.common_dispatch_command(message, command),
        }
    }

    fn common_dispatch_command(&mut self, message: Response, command: Command) {
        match command {
            Command::ListNodes => message.send_value(Ok(self.graph.list_nodes())),
            Command::ListInputs(node) => {
                message.send_value(self.graph.get_node(node).map(|n| n.input_names()))
            }
            Command::ListOutputs(node) => {
                message.send_value(self.graph.get_node(node).map(|n| n.output_names()))
            }
            Command::MultiAssign(vec) => {
                let p: Result<Vec<_>> = vec
                    .into_iter()
                    .map(|(n, p, v)| self.graph.get_node_mut(n).and_then(|n| n.assign(p, v)))
                    .collect();
                message.send_value(p)
            }
            Command::MultiLoad(vec) => {
                let p: Result<Vec<_>> = vec
                    .into_iter()
                    .map(|(n, p, v)| self.graph.get_node_mut(n).and_then(|n| n.load(p, v)))
                    .collect();
                message.send_value(p)
            }
            Command::MultiOwnedAssign(vec) => {
                let p: Result<Vec<_>> = vec
                    .into_iter()
                    .map(|(n, p, v)| {
                        self.graph
                            .get_node_mut(n)
                            .and_then(|n| n.assign_owned(p, v))
                    })
                    .collect();
                message.send_value(p)
            }
            _ => unreachable!(),
        }
    }

    fn request_values(&mut self) -> Result<()> {
        if !self.requests.is_empty() {
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

                for ((node_id, param_id), response) in internals.iter().zip(response) {
                    self.graph
                        .get_node_mut(*node_id)
                        .and_then(|n| n.assign_owned(*param_id, response))?;
                }
            }
        }
        Ok(())
    }

    fn send_values(&mut self) -> Result<()> {
        if !self.sends.is_empty() {
            let (message, _receiver) = Response::new();

            for ((sender, external_nodes), internal_props) in &self.sends {
                let loads = internal_props
                    .iter()
                    .map(|(node_id, param_id)| {
                        self.graph
                            .get_node(*node_id)
                            .and_then(|n| n.dump_owned(*param_id))
                    })
                    .zip(external_nodes)
                    .map(|(v, (n, p))| v.map(|val| (*n, *p, val)))
                    .collect::<Result<Vec<_>>>()?;
                sender.send((Command::MultiOwnedAssign(loads), message.clone()))?;
            }
        }
        Ok(())
    }
}

type InternalCommand = (Command, Response);

pub struct GraphView<'a> {
    r: &'a Runner,
}

impl GraphView<'_> {
    fn command(&self, command: Command) -> Result<Value> {
        self.r.command(command).and_then(|r| match r {
            Internal::Value(value) => Ok(value),
            Internal::Prop(_generic_owned_prop) => Err("cannot return owned".into()),
        })
    }
    pub fn kill(&self) -> Result<Value> {
        self.command(Command::Kill)
    }
    pub fn start(&self) -> Result<Value> {
        self.command(Command::Start)
    }
    pub fn stop(&self) -> Result<Value> {
        self.command(Command::Stop)
    }
    pub fn status(&self) -> Result<Value> {
        self.command(Command::Status)
    }
    pub fn list_nodes(&self) -> Result<Value> {
        self.command(Command::ListNodes)
    }
    pub fn list_inputs(&self, node_id: NodeId) -> Result<Value> {
        self.command(Command::ListInputs(node_id))
    }
    pub fn list_outputs(&self, node_id: NodeId) -> Result<Value> {
        self.command(Command::ListOutputs(node_id))
    }
    pub fn dump(&self, data: Vec<(NodeId, ParamId)>) -> Result<Value> {
        self.command(Command::MultiDump(data))
    }
    pub fn assign(&self, data: Vec<(NodeId, ParamId, Value)>) -> Result<Value> {
        self.command(Command::MultiAssign(data))
    }
    pub fn load(&self, data: Vec<(NodeId, ParamId, Value)>) -> Result<Value> {
        self.command(Command::MultiLoad(data))
    }
}

/// Commands that a runner can send
enum Command {
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
    /// Multi assign command
    MultiAssign(Vec<(NodeId, ParamId, Value)>),
    /// Multi load command
    MultiLoad(Vec<(NodeId, ParamId, Value)>),
    /// Multi owned dump command
    MultiOwnedDump(Vec<(NodeId, ParamId)>),
    /// Multi assign owned command
    MultiOwnedAssign(Vec<(NodeId, ParamId, GenericOwnedProp)>),
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
