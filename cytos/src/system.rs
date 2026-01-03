//! System module
//!
//! A system is a set of graph linked together

use indexmap::IndexMap;
use tracing::trace;

use serde::Serialize;

use crate::Transformer;
use crate::loader::Registry;
use crate::queue::BlockReceiver;

use crate::queue::BlockSender;
use crate::queue::Receiver;
use crate::queue::Sender;
use crate::queue::bounded;
use crate::queue::unbounded;
use crate::repr::GraphRepr;
use crate::repr::LinkKind;
use crate::repr::SystemLink;
use crate::repr::SystemRepr;

use super::graph::Graph;
use super::graph::StepResult;

use super::GenericOwnedProp;
use super::{GraphId, NodeId, ParamId, Result, Value};

use std::thread;
use std::thread::{Builder, JoinHandle};
use std::time::Duration;

fn create_runner(
    repr: GraphRepr,
    receiver: BlockReceiver<InternalCommand>,
    senders: &IndexMap<GraphId, BlockSender<InternalCommand>>,
    registry: Registry,
    requests: Vec<&SystemLink>,
) -> Result<impl FnOnce() + use<>> {
    let requests = create_requests(senders, requests)?;

    Ok(move || {
        let graph = repr.into_graph(&registry).expect("Cannot build graph");

        Worker::new(graph, receiver, requests).run();
    })
}

fn create_requests(
    senders: &IndexMap<GraphId, BlockSender<InternalCommand>>,
    mut requests: Vec<&SystemLink>,
) -> Result<LinksToExternal> {
    requests.sort_by_key(|x| (x.src.0, &x.kind));

    requests[..]
        .chunk_by(|a, b| a.src.0 == b.src.0 && a.kind == b.kind)
        .map(|requests| {
            let graph_id = requests[0].src.0;
            let kind = requests[0].kind.clone();
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
                .get(&graph_id)
                .cloned()
                .map(|sender| LinkToExternal::new_link(kind, sender, sources, destinations))
        })
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| "missin sender".into())
}

impl SystemRepr {
    /// Convert a system representation into a System
    ///
    /// # Errors
    ///
    /// Will return `Errors` when cannot create a runner
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

                create_runner(repr, receiver, &senders, registry.clone(), requests).and_then(
                    |thread| -> Result<(GraphId, JoinHandle<()>)> {
                        Builder::new()
                            .name(id.to_string())
                            .spawn(thread)
                            .map(|thread| (id, thread))
                            .map_err(std::convert::Into::into)
                    },
                )
            })
            .collect::<Result<IndexMap<_, _>>>()?;

        Ok(System { runners, senders })
    }
}

/// A system that manages multiple graphs running concurrently in separate threads.
///
/// Each graph is executed in its own thread and can communicate via links defined in the system representation.
#[derive(Default)]
pub struct System {
    /// Runners where there is a runner per graph
    runners: IndexMap<GraphId, JoinHandle<()>>,
    /// Senders to communicate to graphs
    senders: IndexMap<GraphId, BlockSender<(Command, Response)>>,
}

impl System {
    /// Iterator on graph names
    pub fn graphs(&self) -> impl Iterator<Item = &GraphId> {
        self.runners.keys()
    }

    /// Return a specific graph view
    ///
    /// # Errors
    ///
    /// Will return an error if there is not `graph` in the `system`
    pub fn graph(&self, graph: GraphId) -> Result<GraphView<'_>> {
        let _sender = self.senders.get(&graph).ok_or("not found")?;
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
    /// A receiver for the commands
    receiver: BlockReceiver<InternalCommand>,

    /// Dispatcher
    dispatcher: Dispatcher,
}

impl Worker {
    fn new(
        graph: Graph,
        receiver: BlockReceiver<InternalCommand>,
        requests: LinksToExternal,
    ) -> Self {
        Self {
            receiver,
            dispatcher: Dispatcher {
                graph,
                requests,
                queue: Vec::default(),
            },
        }
    }

    /// Run the internal runner
    fn run(mut self) {
        trace!("graph loaded");
        'main: loop {
            if self.loop_processing_idle_message() < 0 {
                break;
            }
            self.dispatcher.initialize();
            trace!("graph starting");

            loop {
                self.dispatcher.request_values().expect("cannot request");

                match self.dispatcher.step() {
                    Ok(cause) => match self.loop_processing_running_message(&cause) {
                        -1 => break 'main,
                        1 => break,
                        _ => (),
                    },
                    _ => {
                        break;
                    }
                }
            }
            trace!("graph stopping");
            self.dispatcher.terminate();
        }
    }

    fn loop_processing_idle_message(&mut self) -> i8 {
        'idle: loop {
            if let Some(data) = self.receiver.recv_all() {
                for (command, message) in data {
                    trace!("received command {:?}", command);
                    match command {
                        Command::State(StateCommand::Kill) => {
                            message.send_value(Ok(""));
                            return -1;
                        }
                        Command::State(StateCommand::Start) => {
                            message.send_value(Ok(""));
                            break 'idle;
                        }
                        Command::State(StateCommand::Status) => message.send_value(Ok("Idle")),
                        Command::State(StateCommand::Stop) => {
                            message.send_value(Ok(""));
                        }
                        Command::Node(node_command) => {
                            self.dispatcher
                                .dispatch_node_command(&node_command, message);
                        }
                        Command::Structure(structure_command) => {
                            self.dispatcher
                                .dispatch_structure_command(*structure_command, message);
                        }
                        Command::Param(param_command) => {
                            self.dispatcher.dispatch_param_command(
                                param_command,
                                message,
                                &StepResult::Done,
                            );
                        }
                    }
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
        0
    }

    fn loop_processing_running_message(&mut self, cause: &StepResult) -> i8 {
        if let Some(data) = self.receiver.recv_all() {
            for (command, message) in data {
                trace!("received command {:?}", command);
                match command {
                    Command::State(StateCommand::Kill) => {
                        message.send_value(Ok(""));
                        return -1;
                    }
                    Command::State(StateCommand::Stop) => {
                        message.send_value(Ok(""));
                        return 1;
                    }
                    Command::State(StateCommand::Status) => message.send_value(Ok("Running")),
                    Command::State(StateCommand::Start) => {
                        message.send_value(Ok(""));
                    }
                    Command::Node(node_command) => {
                        self.dispatcher
                            .dispatch_node_command(&node_command, message);
                    }
                    Command::Structure(structure_command) => {
                        self.dispatcher
                            .dispatch_structure_command(*structure_command, message);
                    }

                    Command::Param(param_command) => {
                        self.dispatcher
                            .dispatch_param_command(param_command, message, cause);
                    }
                }
            }
        }
        0
    }
}

/// A view into a specific graph within the system, allowing interaction with its state and nodes.
pub struct GraphView<'a> {
    senders: &'a IndexMap<GraphId, BlockSender<(Command, Response)>>,
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

    /// Kill the graph processing.
    ///
    /// # Errors
    ///
    /// Returns an error if the receiver cannot process the request.
    pub fn kill(&self) -> Result<Value> {
        self.command(Command::State(StateCommand::Kill))
    }

    /// Start the graph processing.
    ///
    /// # Errors
    ///
    /// Returns an error if the receiver cannot process the request.
    pub fn start(&self) -> Result<Value> {
        self.command(Command::State(StateCommand::Start))
    }

    /// Stop the graph processing.
    ///
    /// # Errors
    ///
    /// Returns an error if the receiver cannot process the request.
    pub fn stop(&self) -> Result<Value> {
        self.command(Command::State(StateCommand::Stop))
    }

    /// Get the status of the graph.
    ///
    /// # Errors
    ///
    /// Returns an error if the receiver cannot process the request.
    pub fn status(&self) -> Result<Value> {
        self.command(Command::State(StateCommand::Status))
    }

    /// List the nodes in the graph.
    ///
    /// # Errors
    ///
    /// Returns an error if the receiver cannot process the request.
    pub fn list_nodes(&self) -> Result<Value> {
        self.command(Command::Node(NodeCommand::ListNodes))
    }

    /// List the inputs of a node
    ///
    /// # Errors
    /// If the receiver cannot proress the request
    pub fn list_inputs(&self, node_id: NodeId) -> Result<Value> {
        self.command(Command::Node(NodeCommand::ListInputs(node_id)))
    }

    /// List the outputs of a node
    ///
    /// # Errors
    /// If the receiver cannot proress the request
    pub fn list_outputs(&self, node_id: NodeId) -> Result<Value> {
        self.command(Command::Node(NodeCommand::ListOutputs(node_id)))
    }

    /// Remove a node
    ///
    /// # Errors
    /// If the receiver cannot proress the request
    pub fn remove_node(&self, node_id: NodeId) -> Result<Value> {
        self.command(Command::Node(NodeCommand::RemoveNode(node_id)))
    }

    /// Dump a node param
    ///
    /// # Errors
    /// If the receiver cannot proress the request
    pub fn dump(&self, data: Vec<(NodeId, ParamId)>) -> Result<Value> {
        self.command(Command::Param(ParamCommand::Dump(data)))
    }

    /// Assign a node param
    ///
    /// # Errors
    /// If the receiver cannot proress the request
    pub fn assign(&self, data: Vec<(NodeId, ParamId, Value)>) -> Result<Value> {
        self.command(Command::Param(ParamCommand::Assign(data)))
    }

    /// Load a value into a node
    ///
    /// # Errors
    /// If the receiver cannot proress the request
    pub fn load(&self, data: Vec<(NodeId, ParamId, Value)>) -> Result<Value> {
        self.command(Command::Param(ParamCommand::Load(data)))
    }

    /// List the links
    ///
    /// # Errors
    /// If the receiver cannot proress the request
    pub fn list_links(&self) -> Result<Value> {
        self.command(Command::Structure(Box::new(StructureCommand::ListLinks)))
    }

    /// Add a link
    ///
    /// # Errors
    /// If the receiver cannot proress the request
    pub fn add_link(&self, src: (NodeId, ParamId), dst: (NodeId, ParamId)) -> Result<Value> {
        self.command(Command::Structure(Box::new(StructureCommand::AddLink((
            src, dst,
        )))))
    }

    /// List receivers in the graph
    ///
    /// # Errors
    /// If the receiver cannot proress the request
    pub fn list_receivers(&self) -> Result<Value> {
        self.command(Command::Structure(Box::new(
            StructureCommand::ListReceiver(self.senders.clone()),
        )))
    }

    /// Add a receiver
    ///
    /// # Errors
    /// If the receiver cannot proress the request
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

    /// Remove a receiver
    ///
    /// # Errors
    /// If the receiver cannot proress the request
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
}

#[derive(Debug)]
enum StructureCommand {
    /// List links
    ListLinks,
    /// Link nodes
    AddLink(((NodeId, ParamId), (NodeId, ParamId))),
    /// List receivers
    ListReceiver(IndexMap<GraphId, BlockSender<(Command, Response)>>),
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
            Self::Value(value) => write!(f, "{value:?}"),
            Self::Prop(_generic_owned_prop) => write!(f, "GenericOwnedProp"),
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
        let (sender, receiver) = bounded::<ResponseResult>();

        (Self { sender }, receiver)
    }

    /// Set the response and consume the message
    fn send_value<T: Serialize>(self, resp: Result<T>) {
        let resp = resp
            .and_then(|v| Value::load(&v))
            .map(Internal::Value)
            .map_err(|r| r.to_string());

        let _ = self.sender.send(resp);
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
type ExternalDestination = (BlockSender<InternalCommand>, Destination);

struct LinkToExternal {
    kind: LinkKind,
    sender: BlockSender<InternalCommand>,
    src: Vec<Destination>,
    dst: Vec<Destination>,
    response: Response,
    receiver: Receiver<ResponseResult>,
    requested: bool,
}

impl LinkToExternal {
    fn new_link(
        kind: LinkKind,
        external_sender: BlockSender<InternalCommand>,
        external_destinations: Vec<Destination>,
        destinations: Vec<Destination>,
    ) -> Self {
        let (message, receiver) = Response::new();

        Self {
            kind,
            sender: external_sender,
            src: external_destinations,
            dst: destinations,
            response: message,
            receiver,
            requested: false,
        }
    }
    fn same_sender(&self, external_sender: &BlockSender<InternalCommand>) -> bool {
        self.sender.same_channel(external_sender)
    }

    fn add_destination(&mut self, external_destination: Destination, destination: Destination) {
        self.src.push(external_destination);
        self.dst.push(destination);
    }

    fn remove_destination(&mut self, external_destination: Destination, destination: Destination) {
        self.src.retain(|n| *n != external_destination);
        self.dst.retain(|n| *n != destination);
    }

    const fn is_empty(&self) -> bool {
        self.dst.is_empty()
    }

    fn send_request(&mut self) -> Result<()> {
        if self.requested {
            Ok(())
        } else {
            self.sender
                .send((
                    Command::Param(ParamCommand::OwnedDump(self.src.clone())),
                    self.response.clone(),
                ))
                .inspect(|()| self.requested = true)
        }
    }

    fn iter_response(&mut self) -> Result<impl Iterator<Item = (&Destination, GenericOwnedProp)>> {
        let response = if self.kind == LinkKind::Wait {
            self.requested = false;
            self.receiver.recv()?.map(|r| match r {
                Internal::Value(_) => unreachable!(),
                Internal::Prop(v) => v,
            })?
        } else if let Some(p) = self.receiver.try_recv()? {
            self.requested = false;
            match p? {
                Internal::Value(_) => unreachable!(),
                Internal::Prop(v) => v,
            }
        } else {
            vec![]
        };

        Ok(self.dst.iter().zip(response))
    }

    fn iter_destinations(&self) -> impl Iterator<Item = (&Destination, &Destination)> {
        self.src.iter().zip(self.dst.iter())
    }
}

/// Link between an external and an internal resourcev
type LinksToExternal = Vec<LinkToExternal>;

struct Dispatcher {
    /// The graph
    graph: Graph,
    /// Requests between graphs
    requests: LinksToExternal,
    /// Queue
    queue: Vec<(ParamCommand, Response)>,
}

impl Dispatcher {
    fn initialize(&mut self) {
        self.graph.initialize().expect("cannot initialize");
    }

    fn step(&mut self) -> Result<StepResult> {
        self.graph.step()
    }

    fn terminate(&mut self) {
        self.graph.terminate().expect("cannot terminate");
    }

    fn dispatch_node_command(&mut self, node_command: &NodeCommand, message: Response) {
        match node_command {
            NodeCommand::ListNodes => message.send_value(Ok(self.graph.list_nodes())),
            NodeCommand::ListInputs(node) => {
                message.send_value(self.graph.get_node(*node).map(Transformer::input_names));
            }
            NodeCommand::ListOutputs(node) => {
                message.send_value(self.graph.get_node(*node).map(Transformer::output_names));
            }
            NodeCommand::RemoveNode(node) => message.send_value(self.graph.remove(*node)),
        }
    }

    fn common_dispatch_command(&mut self, command: ParamCommand, message: Response) {
        match command {
            ParamCommand::Assign(vec) => {
                let p: Result<Vec<_>> = vec
                    .into_iter()
                    .map(|(n, p, v)| self.graph.get_node_mut(n).and_then(|n| n.assign(p, v)))
                    .collect();
                message.send_value(p);
            }
            ParamCommand::Load(vec) => {
                let p: Result<Vec<_>> = vec
                    .into_iter()
                    .map(|(n, p, v)| self.graph.get_node_mut(n).and_then(|n| n.load(p, v)))
                    .collect();
                message.send_value(p);
            }
            _ => unreachable!(),
        }
    }

    /// Dispatch a command to the graph
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

                message.send_value(Ok(()));
            }
            StructureCommand::ListReceiver(senders) => {
                let senders: Vec<_> = self
                    .requests
                    .iter()
                    .flat_map(|link| {
                        let g = senders
                            .iter()
                            .find(|(_, sender)| link.same_sender(sender))
                            .map(|(g, _)| *g)
                            .unwrap();

                        link.iter_destinations()
                            .map(|((n, p), (n2, p2))| ((g, *n, *p), (*n2, *p2)))
                            .collect::<Vec<_>>()
                    })
                    .collect();
                message.send_value(Ok(senders));
            }

            StructureCommand::AddReceiver((
                (external_sender, external_destination),
                destination,
            )) => {
                if let Some(link) = self
                    .requests
                    .iter_mut()
                    .find(|link| link.same_sender(&external_sender))
                {
                    link.add_destination(external_destination, destination);
                } else {
                    let link = LinkToExternal::new_link(
                        LinkKind::Wait,
                        external_sender,
                        vec![external_destination],
                        vec![destination],
                    );
                    self.requests.push(link);
                }
                message.send_value(Ok(()));
            }
            StructureCommand::RemoveReceiver((
                (external_sender, external_destination),
                destination,
            )) => {
                if let Some(link) = self
                    .requests
                    .iter_mut()
                    .find(|link| link.same_sender(&external_sender))
                {
                    link.remove_destination(external_destination, destination);
                }

                self.requests.retain(|link| !link.is_empty());
                message.send_value(Ok(()));
            }
        }
    }

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
                for (command, message) in queue {
                    self.dispatch_param_command_on_done(command, message);
                }
                self.dispatch_param_command_on_done(command, message);
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
                message.send_value(dump);
            }
            ParamCommand::OwnedDump(vec) => {
                let dump: Result<Vec<_>> = vec
                    .into_iter()
                    .map(|(node, param)| {
                        self.graph.get_node(node).and_then(|n| n.dump_owned(param))
                    })
                    .collect();

                message.send_prop(dump);
            }
            command => self.common_dispatch_command(command, message),
        }
    }

    fn dispatch_param_command_on_skip(&mut self, command: ParamCommand, message: Response) {
        match command {
            ParamCommand::Dump(_) | ParamCommand::OwnedDump(_) => {
                self.queue.push((command, message));
            }
            command => self.common_dispatch_command(command, message),
        }
    }

    fn request_values(&mut self) -> Result<()> {
        trace!("requesting values start");
        for link in &mut self.requests {
            link.send_request()?;
        }
        for link in &mut self.requests {
            for ((node_id, param_id), response) in link.iter_response()? {
                self.graph
                    .get_node_mut(*node_id)
                    .and_then(|n| n.assign_owned(*param_id, response))?;
            }
        }
        trace!("requesting values end");

        Ok(())
    }
}
