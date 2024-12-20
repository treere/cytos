//! System module
//!
//! A system is a set of graph linked together

use crossbeam::channel::bounded;
use crossbeam::channel::unbounded;
use crossbeam::channel::Receiver;
use crossbeam::channel::Sender;
use indexmap::IndexMap;
use log::trace;

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

fn create_runner(
    repr: GraphRepr,
    receiver: Receiver<InternalCommand>,
    senders: IndexMap<GraphId, Sender<InternalCommand>>,
    registry: Registry,
    requests: Vec<&SystemLink>,
    sends: Vec<&SystemLink>,
) -> Result<impl FnOnce()> {
    let requests = create_requests(&senders, requests)?;
    let sends = create_sends(&senders, sends)?;

    Ok(move || {
        let graph = repr.into_graph(&registry).expect("Cannot build graph");

        Worker::new(graph, receiver, requests, sends).run();
    })
}

fn create_sends(
    senders: &IndexMap<GraphId, Sender<InternalCommand>>,
    mut sends: Vec<&SystemLink>,
) -> Result<LinksToExternal> {
    sends.sort_by_key(|x| x.dst.0);

    sends[..]
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
    senders: &IndexMap<GraphId, Sender<InternalCommand>>,
    mut requests: Vec<&SystemLink>,
) -> Result<LinksToExternal> {
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
                let requests = self.requests.iter().filter(|l| l.dst.0 == id).collect();
                let sends = self.sends.iter().filter(|l| l.src.0 == id).collect();

                create_runner(
                    repr,
                    receiver,
                    senders.clone(),
                    registry.clone(),
                    requests,
                    sends,
                )
                .and_then(|thread| -> Result<(GraphId, JoinHandle<()>)> {
                    Builder::new()
                        .name(id.to_string())
                        .spawn(thread)
                        .map(|thread| (id, thread))
                        .map_err(|x| x.into())
                })
            })
            .collect::<Result<IndexMap<_, _>>>()?;

        Ok(System { runners, senders })
    }
}

/// System
#[derive(Default)]
pub struct System {
    /// Runners where there is a runner per graph
    runners: IndexMap<GraphId, JoinHandle<()>>,
    /// Senders to communicate to graphs
    senders: IndexMap<GraphId, Sender<(Command, Response)>>,
}

impl System {
    /// Iterator on graph names
    pub fn graphs(&self) -> impl Iterator<Item = &GraphId> {
        self.runners.keys()
    }

    pub fn graph(&self, graph: GraphId) -> Result<GraphView<'_>> {
        Ok(GraphView {
            senders: &self.senders,
            graph,
        })
    }
}

impl Drop for System {
    fn drop(&mut self) {
        self.senders.values().for_each(|s| {
            let (message, _receiver) = Response::new();
            s.send((Command::State(StateCommand::Kill), message))
                .expect("cannot send");
        });

        let mut queue = IndexMap::default();
        std::mem::swap(&mut self.runners, &mut queue);
        queue.into_iter().for_each(|(_, t)| {
            let _ = t.join();
        });
    }
}

/// The runner worker
struct Worker {
    /// The graph
    graph: Graph,
    /// A receiver for the commands
    receiver: Receiver<InternalCommand>,
    /// Requests between graphs
    requests: LinksToExternal,
    /// Sends between graphs
    sends: LinksToExternal,
    /// Queue
    queue: Vec<(ParamCommand, Response)>,
}

impl Worker {
    fn new(
        graph: Graph,
        receiver: Receiver<InternalCommand>,
        requests: LinksToExternal,
        sends: LinksToExternal,
    ) -> Self {
        Worker {
            graph,
            receiver,
            requests,
            sends,
            queue: Vec::default(),
        }
    }
    /// Run the internal runner
    fn run(mut self) {
        trace!("graph loaded");
        'main: loop {
            while let Ok((command, message)) = self.receiver.recv() {
                trace!("received command {:?}", command);
                match command {
                    Command::State(StateCommand::Kill) => break 'main,
                    Command::State(StateCommand::Start) => break,
                    Command::State(StateCommand::Status) => message.send_value(Ok("Idle")),
                    Command::State(StateCommand::Stop) => (),
                    Command::Node(node_command) => {
                        self.dispatch_node_command(node_command, message)
                    }
                    Command::Structure(structure_command) => {
                        self.dispatch_structure_command(*structure_command, message)
                    }
                    Command::Param(param_command) => {
                        self.dispatch_param_command(param_command, message, &StepResult::Done)
                    }
                }
            }

            self.graph.initialize().expect("cannot initialize");
            trace!("graph starting");

            'outer: loop {
                self.request_values().expect("cannot request");

                if let Ok(cause) = self.graph.step() {
                    self.send_values().expect("cannot send");
                    while let Ok((command, message)) = self.receiver.try_recv() {
                        trace!("received command {:?}", command);
                        match command {
                            Command::State(StateCommand::Kill) => break 'main,
                            Command::State(StateCommand::Stop) => break 'outer,
                            Command::State(StateCommand::Status) => {
                                message.send_value(Ok("Running"))
                            }
                            Command::State(StateCommand::Start) => (),
                            Command::Node(node_command) => {
                                self.dispatch_node_command(node_command, message)
                            }
                            Command::Structure(structure_command) => {
                                self.dispatch_structure_command(*structure_command, message)
                            }

                            Command::Param(param_command) => {
                                self.dispatch_param_command(param_command, message, &cause)
                            }
                        }
                    }
                } else {
                    break 'outer;
                }
            }
            trace!("graph stopping");
            self.graph.terminate().expect("cannot terminate");
        }
    }

    /// Dispatch a command to the graph
    fn dispatch_param_command(
        &mut self,
        command: ParamCommand,
        message: Response,
        cause: &StepResult,
    ) {
        match cause {
            StepResult::Done => {
                let mut queue = vec![];
                std::mem::swap(&mut self.queue, &mut queue);
                queue.into_iter().for_each(|(command, message)| {
                    self.dispatch_param_command_on_done(command, message)
                });
                self.dispatch_param_command_on_done(command, message)
            }
            StepResult::Skip => self.dispatch_param_command_on_skip(command, message),
        }
    }

    fn dispatch_param_command_on_done(&mut self, command: ParamCommand, message: Response) {
        match command {
            ParamCommand::Dump(vec) => {
                let dump: Result<Vec<_>> = vec
                    .into_iter()
                    .map(|(node, param)| self.graph.get_node(node).and_then(|n| n.dump(param)))
                    .collect();
                message.send_value(dump)
            }
            ParamCommand::OwnedDump(vec) => {
                let dump: Result<Vec<_>> = vec
                    .into_iter()
                    .map(|(node, param)| {
                        self.graph.get_node(node).and_then(|n| n.dump_owned(param))
                    })
                    .collect();

                message.send_prop(dump)
            }
            command => self.common_dispatch_command(command, message),
        }
    }

    fn dispatch_param_command_on_skip(&mut self, command: ParamCommand, message: Response) {
        match command {
            ParamCommand::Dump(_) => {
                self.queue.push((command, message));
            }
            ParamCommand::OwnedDump(_) => {
                self.queue.push((command, message));
            }
            command => self.common_dispatch_command(command, message),
        }
    }

    fn dispatch_structure_command(
        &mut self,
        structure_command: StructureCommand,
        message: Response,
    ) {
        match structure_command {
            StructureCommand::ListLinks => message.send_value(Ok(self.graph.collect_links())),
            StructureCommand::AddLink((src, dst)) => {
                let s = self.graph.get_node(src.0).unwrap();
                let s = (*s).output(src.1).unwrap();
                self.graph
                    .get_node_mut(dst.0)
                    .unwrap()
                    .link(dst.1, s)
                    .unwrap();

                message.send_value(Ok(()))
            }
            StructureCommand::ListSender(senders) => {
                let senders: Vec<_> = self
                    .sends
                    .iter()
                    .flat_map(|((s, v1), v2)| {
                        let g = senders
                            .iter()
                            .find(|(_, s2)| s.same_channel(s2))
                            .map(|(g, _)| *g)
                            .unwrap();

                        v1.iter()
                            .zip(v2.iter())
                            .map(|((n, p), (n2, p2))| ((g, *n, *p), (*n2, *p2)))
                            .collect::<Vec<_>>()
                    })
                    .collect();
                message.send_value(Ok(senders))
            }
            StructureCommand::AddSender(((external_sender, external_destination), destination)) => {
                if let Some(((_, external), internal)) = self
                    .sends
                    .iter_mut()
                    .find(|((sender, _), _)| external_sender.same_channel(sender))
                {
                    external.push(external_destination);
                    internal.push(destination)
                } else {
                    self.sends.push((
                        (external_sender, vec![external_destination]),
                        vec![destination],
                    ));
                }
                message.send_value(Ok(()))
            }
            StructureCommand::RemoveSender((
                (external_sender, external_destination),
                destination,
            )) => {
                if let Some(((_, external), internal)) = self
                    .sends
                    .iter_mut()
                    .find(|((sender, _), _)| external_sender.same_channel(sender))
                {
                    external.retain(|n| *n != external_destination);
                    internal.retain(|n| *n != destination);
                }

                self.sends.retain(|(_, internal)| !internal.is_empty());
                message.send_value(Ok(()))
            }
            StructureCommand::ListReceiver(senders) => {
                let senders: Vec<_> = self
                    .requests
                    .iter()
                    .flat_map(|((s, v1), v2)| {
                        let g = senders
                            .iter()
                            .find(|(_, s2)| s.same_channel(s2))
                            .map(|(g, _)| *g)
                            .unwrap();

                        v1.iter()
                            .zip(v2.iter())
                            .map(|((n, p), (n2, p2))| ((g, *n, *p), (*n2, *p2)))
                            .collect::<Vec<_>>()
                    })
                    .collect();
                message.send_value(Ok(senders))
            }

            StructureCommand::AddReceiver((
                (external_sender, external_destination),
                destination,
            )) => {
                if let Some(((_, external), internal)) = self
                    .requests
                    .iter_mut()
                    .find(|((sender, _), _)| external_sender.same_channel(sender))
                {
                    external.push(external_destination);
                    internal.push(destination)
                } else {
                    self.requests.push((
                        (external_sender, vec![external_destination]),
                        vec![destination],
                    ));
                }
                message.send_value(Ok(()))
            }
            StructureCommand::RemoveReceiver((
                (external_sender, external_destination),
                destination,
            )) => {
                if let Some(((_, external), internal)) = self
                    .requests
                    .iter_mut()
                    .find(|((sender, _), _)| external_sender.same_channel(sender))
                {
                    external.retain(|n| *n != external_destination);
                    internal.retain(|n| *n != destination);
                }

                self.requests.retain(|(_, internal)| !internal.is_empty());
                message.send_value(Ok(()))
            }
        }
    }

    fn dispatch_node_command(&mut self, node_command: NodeCommand, message: Response) {
        match node_command {
            NodeCommand::ListNodes => message.send_value(Ok(self.graph.list_nodes())),
            NodeCommand::ListInputs(node) => {
                message.send_value(self.graph.get_node(node).map(|n| n.input_names()))
            }
            NodeCommand::ListOutputs(node) => {
                message.send_value(self.graph.get_node(node).map(|n| n.output_names()))
            }
            NodeCommand::RemoveNode(node) => message.send_value(self.graph.remove(node)),
        }
    }

    fn common_dispatch_command(&mut self, command: ParamCommand, message: Response) {
        match command {
            ParamCommand::Assign(vec) => {
                let p: Result<Vec<_>> = vec
                    .into_iter()
                    .map(|(n, p, v)| self.graph.get_node_mut(n).and_then(|n| n.assign(p, v)))
                    .collect();
                message.send_value(p)
            }
            ParamCommand::Load(vec) => {
                let p: Result<Vec<_>> = vec
                    .into_iter()
                    .map(|(n, p, v)| self.graph.get_node_mut(n).and_then(|n| n.load(p, v)))
                    .collect();
                message.send_value(p)
            }
            ParamCommand::OwnedAssign(vec) => {
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
            trace!("requesting values start");
            let (message, receiver) = Response::new();
            for ((sender, nodes), internals) in &self.requests {
                let response: Vec<_> = match sender.send((
                    Command::Param(ParamCommand::OwnedDump(nodes.clone())),
                    message.clone(),
                )) {
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
            trace!("requesting values end");
        }
        Ok(())
    }

    fn send_values(&mut self) -> Result<()> {
        if !self.sends.is_empty() {
            trace!("sending values start");
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
                sender.send((
                    Command::Param(ParamCommand::OwnedAssign(loads)),
                    message.clone(),
                ))?;
            }
            trace!("sending values end");
        }
        Ok(())
    }
}

pub struct GraphView<'a> {
    senders: &'a IndexMap<GraphId, Sender<(Command, Response)>>,
    graph: GraphId,
}

impl GraphView<'_> {
    fn command(&self, command: Command) -> Result<Value> {
        let sender = self.senders.get(&self.graph).ok_or("not found")?;
        let (message, receiver) = Response::new();

        match sender.send((command, message)) {
            Ok(()) => receiver
                .recv()
                .unwrap_or(Ok(Internal::Value(Value::load(&())?)))
                .map_err(Into::into),
            Err(_) => Err("Cannot send".into()),
        }
        .and_then(|r| match r {
            Internal::Value(value) => Ok(value),
            Internal::Prop(_generic_owned_prop) => Err("cannot return owned".into()),
        })
    }
    pub fn kill(&self) -> Result<Value> {
        self.command(Command::State(StateCommand::Kill))
    }
    pub fn start(&self) -> Result<Value> {
        self.command(Command::State(StateCommand::Start))
    }
    pub fn stop(&self) -> Result<Value> {
        self.command(Command::State(StateCommand::Stop))
    }
    pub fn status(&self) -> Result<Value> {
        self.command(Command::State(StateCommand::Status))
    }
    pub fn list_nodes(&self) -> Result<Value> {
        self.command(Command::Node(NodeCommand::ListNodes))
    }
    pub fn list_inputs(&self, node_id: NodeId) -> Result<Value> {
        self.command(Command::Node(NodeCommand::ListInputs(node_id)))
    }
    pub fn list_outputs(&self, node_id: NodeId) -> Result<Value> {
        self.command(Command::Node(NodeCommand::ListOutputs(node_id)))
    }
    pub fn remove_node(&self, node_id: NodeId) -> Result<Value> {
        self.command(Command::Node(NodeCommand::RemoveNode(node_id)))
    }
    pub fn dump(&self, data: Vec<(NodeId, ParamId)>) -> Result<Value> {
        self.command(Command::Param(ParamCommand::Dump(data)))
    }
    pub fn assign(&self, data: Vec<(NodeId, ParamId, Value)>) -> Result<Value> {
        self.command(Command::Param(ParamCommand::Assign(data)))
    }
    pub fn load(&self, data: Vec<(NodeId, ParamId, Value)>) -> Result<Value> {
        self.command(Command::Param(ParamCommand::Load(data)))
    }
    pub fn list_links(&self) -> Result<Value> {
        self.command(Command::Structure(Box::new(StructureCommand::ListLinks)))
    }

    pub fn add_link(&self, src: (NodeId, ParamId), dst: (NodeId, ParamId)) -> Result<Value> {
        self.command(Command::Structure(Box::new(StructureCommand::AddLink((
            src, dst,
        )))))
    }
    pub fn list_senders(&self) -> Result<Value> {
        self.command(Command::Structure(Box::new(StructureCommand::ListSender(
            self.senders.clone(),
        ))))
    }
    pub fn add_sender(
        &self,
        src: (NodeId, ParamId),
        dst: (GraphId, NodeId, ParamId),
    ) -> Result<Value> {
        let sender = self.senders.get(&dst.0).ok_or("not found")?;
        self.command(Command::Structure(Box::new(StructureCommand::AddSender((
            (sender.clone(), (dst.1, dst.2)),
            src,
        )))))
    }
    pub fn remove_sender(
        &self,
        src: (NodeId, ParamId),
        dst: (GraphId, NodeId, ParamId),
    ) -> Result<Value> {
        let sender = self.senders.get(&dst.0).ok_or("not found")?;
        self.command(Command::Structure(Box::new(
            StructureCommand::RemoveSender(((sender.clone(), (dst.1, dst.2)), src)),
        )))
    }
    pub fn list_receivers(&self) -> Result<Value> {
        self.command(Command::Structure(Box::new(
            StructureCommand::ListReceiver(self.senders.clone()),
        )))
    }
    pub fn add_receiver(
        &self,
        src: (GraphId, NodeId, ParamId),
        dst: (NodeId, ParamId),
    ) -> Result<Value> {
        let sender = self.senders.get(&src.0).ok_or("not found")?;
        self.command(Command::Structure(Box::new(StructureCommand::AddReceiver(
            ((sender.clone(), (src.1, src.2)), dst),
        ))))
    }
    pub fn remove_receiver(
        &self,
        src: (GraphId, NodeId, ParamId),
        dst: (NodeId, ParamId),
    ) -> Result<Value> {
        let sender = self.senders.get(&src.0).ok_or("not found")?;
        self.command(Command::Structure(Box::new(
            StructureCommand::RemoveReceiver(((sender.clone(), (src.1, src.2)), dst)),
        )))
    }
}

#[derive(Debug)]
enum StateCommand {
    /// Kill the runner
    Kill,
    /// Start the runner
    Start,
    /// Stop the runner
    Stop,
    /// Receive the runner status
    Status,
}

#[derive(Debug)]
enum NodeCommand {
    /// List the nodes of the graph inside the runner
    ListNodes,
    /// List the inputs of a node
    ListInputs(NodeId),
    /// List the outputs of a node
    ListOutputs(NodeId),
    /// Remove a node
    RemoveNode(NodeId),
}

#[derive(Debug)]
enum ParamCommand {
    /// Multi dump command
    Dump(Vec<(NodeId, ParamId)>),
    /// Multi assign command
    Assign(Vec<(NodeId, ParamId, Value)>),
    /// Multi load command
    Load(Vec<(NodeId, ParamId, Value)>),
    /// Multi owned dump command
    OwnedDump(Vec<(NodeId, ParamId)>),
    /// Multi assign owned command
    OwnedAssign(Vec<(NodeId, ParamId, GenericOwnedProp)>),
}

#[derive(Debug)]
enum StructureCommand {
    /// List links
    ListLinks,
    /// Link nodes
    AddLink(((NodeId, ParamId), (NodeId, ParamId))),
    /// List senders
    ListSender(IndexMap<GraphId, Sender<(Command, Response)>>),
    /// Add sender
    AddSender((ExternalDestination, Destination)),
    /// Remove sender
    RemoveSender((ExternalDestination, Destination)),
    /// List receivers
    ListReceiver(IndexMap<GraphId, Sender<(Command, Response)>>),
    /// Add a request
    AddReceiver((ExternalDestination, Destination)),
    /// Remove a request
    RemoveReceiver((ExternalDestination, Destination)),
}

#[derive(Debug)]
/// Commands that a runner can send
enum Command {
    /// State command
    State(StateCommand),
    Node(NodeCommand),
    Param(ParamCommand),
    Structure(Box<StructureCommand>),
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

type InternalCommand = (Command, Response);

/// Internal address
type Destination = (NodeId, ParamId);

/// External address
type ExternalDestination = (Sender<InternalCommand>, Destination);

/// External addresses
type ExternalDestinations = (Sender<InternalCommand>, Vec<Destination>);

/// Link between an external and an internal resource
type LinksToExternal = Vec<(ExternalDestinations, Vec<Destination>)>;
