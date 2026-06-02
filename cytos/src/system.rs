//! System module
//!
//! A system is a set of graph linked together

use indexmap::IndexMap;
use tracing::trace;

use serde::Serialize;

use crate::NodeMetadata;
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

use super::BufferHandle;

use super::graph::Graph;
use super::graph::StepResult;

use super::GenericOwnedProp;
use super::{GraphId, NodeId, ParamId, Result, Value};

use std::collections::HashMap;
use std::thread;
use std::thread::{Builder, JoinHandle};
use std::time::Duration;

fn create_runner(
    repr: GraphRepr,
    buffer_links: HashMap<crate::NodeId, HashMap<crate::ParamId, String>>,
    buffer_registry: HashMap<String, BufferHandle>,
    receiver: BlockReceiver<InternalCommand>,
    senders: &IndexMap<GraphId, BlockSender<InternalCommand>>,
    registry: Registry,
    requests: Vec<&SystemLink>,
) -> Result<impl FnOnce() + use<>> {
    let requests = create_requests(senders, requests)?;

    Ok(move || {
        let mut graph = repr.into_graph(&registry).expect("Cannot build graph");

        // Wire buffer links
        for (node_id, links) in &buffer_links {
            if let Ok(node) = graph.get_node_mut(*node_id) {
                for (param_id, buffer_name) in links {
                    if let Some(prop) = node.get_prop_mut(*param_id)
                        && let Some(handle) = buffer_registry.get(buffer_name)
                    {
                        prop.link_buffer(handle.clone())
                            .expect("cannot link buffer");
                    }
                }
            }
        }

        Worker::new(graph, receiver, requests, registry).run();
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
        .ok_or_else(|| "missing sender".into())
}

impl SystemRepr {
    /// Convert a system representation into a System
    ///
    /// # Errors
    ///
    /// Will return `Errors` when cannot create a runner
    pub fn to_system(self, registry: &Registry) -> Result<System> {
        // Create buffer handles from buffer configurations
        let buffer_registry: HashMap<String, BufferHandle> = self
            .buffers
            .into_iter()
            .map(|(name, repr)| (name, BufferHandle::new(repr.capacity)))
            .collect();

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
                // Extract buffer links from this graph's node representations
                let buffer_links: HashMap<crate::NodeId, HashMap<crate::ParamId, String>> = repr
                    .nodes
                    .iter()
                    .map(|internal| (internal.name, internal.node.buffer_links.clone()))
                    .filter(|(_, links)| !links.is_empty())
                    .collect();

                let requests = self.requests.iter().filter(|l| l.dst.0 == id).collect();

                create_runner(
                    repr,
                    buffer_links,
                    buffer_registry.clone(),
                    receiver,
                    &senders,
                    registry.clone(),
                    requests,
                )
                .and_then(|thread| -> Result<(GraphId, JoinHandle<()>)> {
                    Builder::new()
                        .name(id.to_string())
                        .spawn(thread)
                        .map(|thread| (id, thread))
                        .map_err(std::convert::Into::into)
                })
            })
            .collect::<Result<IndexMap<_, _>>>()?;

        Ok(System {
            registry: registry.clone(),
            runners,
            senders,
            buffer_registry,
        })
    }
}

/// A system that manages multiple graphs running concurrently in separate threads.
///
/// Each graph is executed in its own thread and can communicate via links defined in the system representation.
#[derive(Default)]
pub struct System {
    /// The registry of available factories
    registry: Registry,
    /// Runners where there is a runner per graph
    runners: IndexMap<GraphId, JoinHandle<()>>,
    /// Senders to communicate to graphs
    senders: IndexMap<GraphId, BlockSender<(Command, Response)>>,
    /// Named buffer handles for cross-graph communication
    buffer_registry: HashMap<String, BufferHandle>,
}

impl System {
    /// Iterator on graph names
    pub fn graphs(&self) -> impl Iterator<Item = &GraphId> {
        self.runners.keys()
    }

    /// Get metadata for a factory
    pub fn get_factory_metadata(&self, name: &str) -> Option<&NodeMetadata> {
        self.registry.get_metadata(name)
    }

    /// Get a reference to the registry
    pub const fn registry(&self) -> &Registry {
        &self.registry
    }

    /// Load a library into the registry
    ///
    /// # Errors
    ///
    /// Will return `Err` if the library cannot be loaded
    pub fn load_library(&mut self, file: &str) -> Result<()> {
        self.registry.load_library(file)
    }

    /// Returns buffer statistics for a named buffer.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the buffer.
    ///
    /// # Returns
    ///
    /// A tuple of `(current_len, capacity)`.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the buffer name is not found.
    pub fn buffer_stats(&self, name: &str) -> Result<(usize, usize)> {
        self.buffer_registry
            .get(name)
            .map(|handle| (handle.len(), handle.capacity()))
            .ok_or_else(|| format!("buffer '{name}' not found").into())
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
        registry: Registry,
    ) -> Self {
        Self {
            receiver,
            dispatcher: Dispatcher {
                graph,
                requests,
                queue: Vec::default(),
                registry,
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

    /// Describe a node instance
    ///
    /// # Errors
    /// If the receiver cannot process the request
    pub fn describe_node(&self, node_id: NodeId) -> Result<Value> {
        self.command(Command::Node(NodeCommand::DescribeNode(node_id)))
    }

    /// Describe a factory type
    ///
    /// # Errors
    /// If the receiver cannot process the request
    pub fn describe_factory(&self, factory_name: String) -> Result<Value> {
        self.command(Command::Node(NodeCommand::DescribeFactory(factory_name)))
    }

    /// Remove a node
    ///
    /// # Errors
    /// If the receiver cannot process the request
    pub fn remove_node(&self, node_id: NodeId) -> Result<Value> {
        self.command(Command::Node(NodeCommand::RemoveNode(node_id)))
    }

    /// Dump a node param
    ///
    /// # Errors
    /// If the receiver cannot process the request
    pub fn dump(&self, data: Vec<(NodeId, ParamId)>) -> Result<Value> {
        self.command(Command::Param(ParamCommand::Dump(data)))
    }

    /// Assign a node param
    ///
    /// # Errors
    /// If the receiver cannot process the request
    pub fn assign(&self, data: Vec<(NodeId, ParamId, Value)>) -> Result<Value> {
        self.command(Command::Param(ParamCommand::Assign(data)))
    }

    /// Load a value into a node
    ///
    /// # Errors
    /// If the receiver cannot process the request
    pub fn load(&self, data: Vec<(NodeId, ParamId, Value)>) -> Result<Value> {
        self.command(Command::Param(ParamCommand::Load(data)))
    }

    /// List the links
    ///
    /// # Errors
    /// If the receiver cannot process the request
    pub fn list_links(&self) -> Result<Value> {
        self.command(Command::Structure(Box::new(StructureCommand::ListLinks)))
    }

    /// Add a link
    ///
    /// # Errors
    /// If the receiver cannot process the request
    pub fn add_link(&self, src: (NodeId, ParamId), dst: (NodeId, ParamId)) -> Result<Value> {
        self.command(Command::Structure(Box::new(StructureCommand::AddLink((
            src, dst,
        )))))
    }

    /// List receivers in the graph
    ///
    /// # Errors
    /// If the receiver cannot process the request
    pub fn list_receivers(&self) -> Result<Value> {
        self.command(Command::Structure(Box::new(
            StructureCommand::ListReceiver(self.senders.clone()),
        )))
    }

    /// Add a receiver
    ///
    /// # Errors
    /// If the receiver cannot process the request
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
    /// If the receiver cannot process the request
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
    /// Remove a node
    RemoveNode(NodeId),
    /// Describe a node instance
    DescribeNode(NodeId),
    /// Describe a factory type
    DescribeFactory(String),
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
    /// Registry for factory metadata
    registry: Registry,
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
            NodeCommand::RemoveNode(node) => message.send_value(self.graph.remove(*node)),
            NodeCommand::DescribeNode(node) => {
                message.send_value(self.graph.get_node_metadata(*node, &self.registry).cloned());
            }
            NodeCommand::DescribeFactory(name) => {
                message.send_value(Ok(self.registry.get_metadata(name).cloned()));
            }
        }
    }

    fn common_dispatch_command(&mut self, command: ParamCommand, message: Response) {
        match command {
            ParamCommand::Assign(vec) => {
                let p: Result<Vec<_>> = vec
                    .into_iter()
                    .map(|(n, p, v)| {
                        self.graph
                            .get_node_mut(n)
                            .and_then(|n| n.get_prop_mut(p).unwrap().assign(v))
                    })
                    .collect();
                message.send_value(p);
            }
            ParamCommand::Load(vec) => {
                let p: Result<Vec<_>> = vec
                    .into_iter()
                    .map(|(n, p, v)| {
                        self.graph
                            .get_node_mut(n)
                            .and_then(|n| n.get_prop_mut(p).unwrap().load(v))
                    })
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
                let s = (*s).get_prop(src.1).unwrap().as_generic();
                self.graph
                    .get_node_mut(dst.0)
                    .unwrap()
                    .get_prop_mut(dst.1)
                    .unwrap()
                    .link(s)
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
                    .map(|(node, param)| {
                        self.graph
                            .get_node(node)
                            .and_then(|n| n.get_prop(param).unwrap().dump())
                    })
                    .collect();
                message.send_value(dump);
            }
            ParamCommand::OwnedDump(vec) => {
                let dump: Result<Vec<_>> = vec
                    .into_iter()
                    .map(|(node, param)| {
                        self.graph
                            .get_node(node)
                            .map(|n| n.get_prop(param).unwrap().as_owned())
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
                    .and_then(|n| n.get_prop_mut(*param_id).unwrap().assign_owned(response))?;
            }
        }
        trace!("requesting values end");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::Registry;
    use crate::props::BufferProp;
    use crate::repr::{BufferRepr, GraphRepr, InternalNodeRepr, NodeRepr, OnError};
    use crate::test::{Constant, Empty};
    use crate::{
        MetadataProvider, NodeMetadata, ParamDirection, ParamInfo, PropInspector, Stepper,
    };
    use std::collections::HashMap;

    fn create_test_registry() -> Registry {
        let mut registry = Registry::default();
        registry.add("Empty", Empty::default);
        registry.add("Constant", Constant::default);
        registry
    }

    fn create_single_node_graph_repr(node_type: &str, node_name: NodeId) -> GraphRepr {
        GraphRepr {
            links: vec![],
            nodes: vec![InternalNodeRepr {
                name: node_name,
                node: NodeRepr {
                    typ: node_type.to_string(),
                    ..Default::default()
                },
                on_error: OnError::Continue,
            }],
        }
    }

    #[test]
    fn test_system_creation_single_graph() {
        let registry = create_test_registry();
        let graph_repr = create_single_node_graph_repr("Empty", NodeId(0));

        let mut system_repr = SystemRepr {
            graphs: HashMap::new(),
            requests: vec![],
            ..Default::default()
        };
        system_repr.graphs.insert(GraphId(0), graph_repr);

        let system = system_repr.to_system(&registry);
        assert!(system.is_ok());

        let system = system.unwrap();
        let graphs: Vec<_> = system.graphs().collect();
        assert_eq!(graphs.len(), 1);
        assert_eq!(graphs[0], &GraphId(0));
    }

    #[test]
    fn test_system_creation_multiple_graphs() {
        let registry = create_test_registry();
        let graph1_repr = create_single_node_graph_repr("Empty", NodeId(0));
        let graph2_repr = create_single_node_graph_repr("Constant", NodeId(1));

        let mut system_repr = SystemRepr {
            graphs: HashMap::new(),
            requests: vec![],
            ..Default::default()
        };
        system_repr.graphs.insert(GraphId(0), graph1_repr);
        system_repr.graphs.insert(GraphId(1), graph2_repr);

        let system = system_repr.to_system(&registry);
        assert!(system.is_ok());

        let system = system.unwrap();
        assert_eq!(system.graphs().count(), 2);
    }

    #[test]
    fn test_system_empty_graphs() {
        let registry = create_test_registry();
        let system_repr = SystemRepr {
            graphs: HashMap::new(),
            requests: vec![],
            ..Default::default()
        };

        let system = system_repr.to_system(&registry);
        assert!(system.is_ok());

        let system = system.unwrap();
        assert_eq!(system.graphs().count(), 0);
    }

    #[test]
    fn test_system_graph_view_creation() {
        let registry = create_test_registry();
        let graph_repr = create_single_node_graph_repr("Empty", NodeId(0));

        let mut system_repr = SystemRepr {
            graphs: HashMap::new(),
            requests: vec![],
            ..Default::default()
        };
        system_repr.graphs.insert(GraphId(0), graph_repr);

        let system = system_repr.to_system(&registry).unwrap();

        // Should be able to get a graph view for existing graph
        let view = system.graph(GraphId(0));
        assert!(view.is_ok());

        // Should fail for non-existent graph
        let view = system.graph(GraphId(999));
        assert!(view.is_err());
    }

    #[test]
    fn test_system_factory_metadata() {
        let registry = create_test_registry();
        let graph_repr = create_single_node_graph_repr("Empty", NodeId(0));

        let mut system_repr = SystemRepr {
            graphs: HashMap::new(),
            requests: vec![],
            ..Default::default()
        };
        system_repr.graphs.insert(GraphId(0), graph_repr);

        let system = system_repr.to_system(&registry).unwrap();

        // Should be able to get metadata for registered factories
        let metadata = system.get_factory_metadata("Empty");
        assert!(metadata.is_some());
        assert_eq!(metadata.unwrap().name, "Empty");

        // Should return None for unknown factories
        let metadata = system.get_factory_metadata("Unknown");
        assert!(metadata.is_none());
    }

    #[test]
    fn test_system_registry_access() {
        let registry = create_test_registry();
        let graph_repr = create_single_node_graph_repr("Empty", NodeId(0));

        let mut system_repr = SystemRepr {
            graphs: HashMap::new(),
            requests: vec![],
            ..Default::default()
        };
        system_repr.graphs.insert(GraphId(0), graph_repr);

        let system = system_repr.to_system(&registry).unwrap();

        // Should be able to access the registry
        let registry_ref = system.registry();
        assert!(registry_ref.get_metadata("Empty").is_some());
    }

    #[test]
    fn test_system_drop_cleanup() {
        let registry = create_test_registry();
        let graph_repr = create_single_node_graph_repr("Empty", NodeId(0));

        let mut system_repr = SystemRepr {
            graphs: HashMap::new(),
            requests: vec![],
            ..Default::default()
        };
        system_repr.graphs.insert(GraphId(0), graph_repr);

        {
            let system = system_repr.to_system(&registry).unwrap();
            // System should be able to run briefly
            let view = system.graph(GraphId(0)).unwrap();
            let _ = view.status();
            // System will be dropped here
        }
        // If we get here without panicking, cleanup worked
    }

    #[test]
    fn test_response_creation() {
        let (response, receiver) = Response::new();

        // Test sending a value
        response.send_value(Ok(42i32));

        let result = receiver.recv().unwrap();
        match result {
            Ok(Internal::Value(v)) => {
                let value: i32 = v.dump().unwrap();
                assert_eq!(value, 42);
            }
            _ => panic!("Expected Value"),
        }
    }

    #[test]
    fn test_response_error() {
        let (response, receiver) = Response::new();

        // Test sending an error
        let err: crate::Result<i32> = Err("test error".into());
        response.send_value(err);

        let result = receiver.recv().unwrap();
        assert!(result.is_err());
    }

    #[test]
    fn test_create_requests_empty() {
        let senders: IndexMap<GraphId, BlockSender<InternalCommand>> = IndexMap::new();
        let requests: Vec<&SystemLink> = vec![];

        let result = create_requests(&senders, requests);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_system_repr_from_json_valid() {
        let json = r#"{
            "graphs": {
                "0": {
                    "nodes": [
                        {
                            "name": "0",
                            "type": "TestNode"
                        }
                    ],
                    "links": []
                }
            },
            "requests": []
        }"#;

        let result = SystemRepr::from_json(json);
        assert!(result.is_ok());

        let system_repr = result.unwrap();
        assert_eq!(system_repr.graphs.len(), 1);
        assert!(system_repr.graphs.contains_key(&GraphId(0)));
    }

    #[test]
    fn test_system_repr_from_json_invalid() {
        let json = r"{ invalid json }";
        let result = SystemRepr::from_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_system_repr_from_json_empty() {
        let json = r#"{"graphs": {}, "requests": []}"#;
        let result = SystemRepr::from_json(json);
        assert!(result.is_ok());

        let system_repr = result.unwrap();
        assert!(system_repr.graphs.is_empty());
    }

    // Test node types for buffer integration test
    struct BufferProducer {
        output: BufferProp<i32>,
        count: i32,
        max: i32,
        done: bool,
    }

    impl BufferProducer {
        fn new(max: i32) -> Self {
            Self {
                output: BufferProp::new(0),
                count: 0,
                max,
                done: false,
            }
        }
    }

    impl Stepper for BufferProducer {
        fn step(&mut self) -> crate::Result<()> {
            if !self.done {
                for i in self.count..self.max {
                    *self.output = i;
                    self.output.push()?;
                }
                self.count = self.max;
                self.done = true;
            }
            Ok(())
        }
    }

    impl PropInspector for BufferProducer {
        fn get_prop(&self, val: ParamId) -> Option<&dyn crate::props::GenericPropInterface> {
            match val {
                ParamId(0) => Some(&self.output),
                _ => None,
            }
        }

        fn get_prop_mut(
            &mut self,
            val: ParamId,
        ) -> Option<&mut dyn crate::props::GenericPropInterface> {
            match val {
                ParamId(0) => Some(&mut self.output),
                _ => None,
            }
        }

        fn metadata(&self) -> &NodeMetadata {
            use std::sync::OnceLock;
            static METADATA: OnceLock<NodeMetadata> = OnceLock::new();
            METADATA.get_or_init(<Self as MetadataProvider>::metadata)
        }
    }

    impl MetadataProvider for BufferProducer {
        fn metadata() -> NodeMetadata {
            NodeMetadata {
                name: "BufferProducer".to_string(),
                description: "Test buffer producer".to_string(),
                params: vec![ParamInfo {
                    id: ParamId(0),
                    name: "output".to_string(),
                    description: "Output buffer prop".to_string(),
                    directions: vec![ParamDirection::Output],
                    type_name: "BufferProp<i32>".to_string(),
                }],
            }
        }
    }

    struct BufferConsumer {
        input: BufferProp<i32>,
    }

    impl BufferConsumer {
        fn new() -> Self {
            Self {
                input: BufferProp::new(0),
            }
        }
    }

    impl Stepper for BufferConsumer {
        fn step(&mut self) -> crate::Result<()> {
            // Pop in a loop to drain as much as possible per step
            loop {
                if !self.input.pop()? {
                    break;
                }
            }
            Ok(())
        }
    }

    impl PropInspector for BufferConsumer {
        fn get_prop(&self, val: ParamId) -> Option<&dyn crate::props::GenericPropInterface> {
            match val {
                ParamId(0) => Some(&self.input),
                _ => None,
            }
        }

        fn get_prop_mut(
            &mut self,
            val: ParamId,
        ) -> Option<&mut dyn crate::props::GenericPropInterface> {
            match val {
                ParamId(0) => Some(&mut self.input),
                _ => None,
            }
        }

        fn metadata(&self) -> &NodeMetadata {
            use std::sync::OnceLock;
            static METADATA: OnceLock<NodeMetadata> = OnceLock::new();
            METADATA.get_or_init(<Self as MetadataProvider>::metadata)
        }
    }

    impl MetadataProvider for BufferConsumer {
        fn metadata() -> NodeMetadata {
            NodeMetadata {
                name: "BufferConsumer".to_string(),
                description: "Test buffer consumer".to_string(),
                params: vec![ParamInfo {
                    id: ParamId(0),
                    name: "input".to_string(),
                    description: "Input buffer prop".to_string(),
                    directions: vec![ParamDirection::Input],
                    type_name: "BufferProp<i32>".to_string(),
                }],
            }
        }
    }

    #[test]
    fn test_integration_two_graphs_shared_buffer() {
        let mut registry = create_test_registry();
        registry.add("BufferProducer", || BufferProducer::new(100));
        registry.add("BufferConsumer", BufferConsumer::new);

        let mut system_repr = SystemRepr {
            graphs: HashMap::new(),
            requests: vec![],
            ..Default::default()
        };
        system_repr
            .buffers
            .insert("shared".to_string(), BufferRepr { capacity: 100 });

        // Producer graph (graph 0)
        let mut producer_props = HashMap::new();
        producer_props.insert(ParamId(0), crate::Value::load(&0i32).unwrap());
        let mut producer_buffer_links = HashMap::new();
        producer_buffer_links.insert(ParamId(0), "shared".to_string());

        let producer_node = InternalNodeRepr {
            name: NodeId(0),
            node: NodeRepr {
                typ: "BufferProducer".to_string(),
                props: producer_props,
                buffer_links: producer_buffer_links,
            },
            on_error: OnError::Continue,
        };
        system_repr.graphs.insert(
            GraphId(0),
            GraphRepr {
                nodes: vec![producer_node],
                links: vec![],
            },
        );

        // Consumer graph (graph 1)
        let mut consumer_props = HashMap::new();
        consumer_props.insert(ParamId(0), crate::Value::load(&0i32).unwrap());
        let mut consumer_buffer_links = HashMap::new();
        consumer_buffer_links.insert(ParamId(0), "shared".to_string());

        let consumer_node = InternalNodeRepr {
            name: NodeId(0),
            node: NodeRepr {
                typ: "BufferConsumer".to_string(),
                props: consumer_props,
                buffer_links: consumer_buffer_links,
            },
            on_error: OnError::Continue,
        };
        system_repr.graphs.insert(
            GraphId(1),
            GraphRepr {
                nodes: vec![consumer_node],
                links: vec![],
            },
        );

        // Build the system
        let system = system_repr.to_system(&registry).unwrap();

        // Buffer stats before running
        let (len, cap) = system.buffer_stats("shared").unwrap();
        assert_eq!(len, 0);
        assert_eq!(cap, 100);

        // Start both graphs
        system.graph(GraphId(0)).unwrap().start().unwrap();
        system.graph(GraphId(1)).unwrap().start().unwrap();

        // Let them run — producer pushes 100 values, consumer drains them
        std::thread::sleep(Duration::from_millis(500));

        // Stop both
        system.graph(GraphId(0)).unwrap().stop().unwrap();
        system.graph(GraphId(1)).unwrap().stop().unwrap();

        // Give threads time to terminate
        std::thread::sleep(Duration::from_millis(100));

        // All 100 values should have been consumed
        let (len, _) = system.buffer_stats("shared").unwrap();
        assert_eq!(len, 0);
    }
}
