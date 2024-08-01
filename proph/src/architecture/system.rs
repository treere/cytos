use crossbeam::channel::bounded;
use crossbeam::channel::unbounded;
use crossbeam::channel::Receiver;
use crossbeam::channel::Sender;
use serde::Deserialize;
use serde::Serialize;

use crate::loader::Registry;

use super::graph::{Graph, GraphRepr, LinkSource};

use super::{GraphId, NodeId, ParamId, Result, Value};

use std::collections::HashMap;
use std::thread::{Builder, JoinHandle};

#[derive(Deserialize, Debug)]
pub struct SystemRepr {
    #[serde(default)]
    pub graphs: Vec<GraphRepr>,
}

impl SystemRepr {
    pub fn from_json(file: &str) -> Result<Self> {
        serde_json::from_str(file).map_err(|v| format!("cannot read file: {}", v).into())
    }
}

#[derive(Default)]
pub struct System {
    runners: Vec<(GraphId, Runner)>,
}

impl System {
    pub fn command(&mut self, graph: GraphId, command: Command) -> Result<Response> {
        let (_, g) = self
            .runners
            .iter_mut()
            .find(|x| x.0 == graph)
            .ok_or("not found")?;
        g.command(command)
    }

    pub fn from_repr(repr: SystemRepr, loader: &Registry) -> Result<Self> {
        let channels: HashMap<_, _> = repr
            .graphs
            .iter()
            .map(|x| {
                let (sender, receiver) = unbounded::<(Command, Message)>();
                (x.name, (sender, Some(receiver)))
            })
            .collect();

        let senders: HashMap<GraphId, Sender<(Command, Message)>> =
            channels.iter().map(|(k, (s, _))| (*k, s.clone())).collect();

        let mut receivers: HashMap<GraphId, Option<Receiver<(Command, Message)>>> =
            channels.into_iter().map(|(k, (_, r))| (k, r)).collect();

        let v = repr
            .graphs
            .into_iter()
            .map(|graph_repr| {
                load_runner(graph_repr, &mut receivers, loader.clone(), senders.clone())
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self { runners: v })
    }

    pub fn keys(&self) -> impl Iterator<Item = &GraphId> {
        self.runners.iter().map(|(v, _)| v)
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

pub enum Command {
    Kill,
    Start,
    Stop,
    Status,
    ListNodes,
    ListInputs(NodeId),
    ListOutputs(NodeId),
    Dump(NodeId, ParamId),
    Load(NodeId, ParamId, Value),
}

type ResponseResult = std::result::Result<Response, String>;

#[derive(Debug)]
pub struct Response(pub Value);

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
            .and_then(|v| Value::load(&v).map(Response))
            .map_err(|r| r.to_string());
        self.sender.send(resp).expect("cannot send");
    }
}

type ExternalReference = (Sender<(Command, Message)>, NodeId, ParamId);

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

                self.external.iter().for_each(|((s, n0, p0), (n1, p1))| {
                    let (message, receiver) = Message::new();

                    let r = match s.send((Command::Dump(*n0, *p0), message)) {
                        Ok(()) => receiver
                            .recv()
                            .unwrap_or(Ok(Response(Value::load(&()).unwrap()))),
                        Err(_) => Err("Cannot send".into()),
                    };
                    let r = r.unwrap();
                    self.graph.load((*n1, *p1), r.0).unwrap();
                });

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
                        LinkSource::External(g, n, p) => ((senders[g].clone(), *n, *p), x.dst),
                    })
                    .collect::<Vec<_>>();

                repr.links
                    .retain(|x| matches!(x.src, LinkSource::Internal(_, _)));
                let graph = Graph::try_from_repr(repr, &reg).expect("Cannot build graph");

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

    pub fn command(&mut self, command: Command) -> Result<Response> {
        let (message, receiver) = Message::new();

        match self.sender.send((command, message)) {
            Ok(()) => receiver
                .recv()
                .unwrap_or(Ok(Response(Value::load(&()).unwrap())))
                .map_err(|r| r.into()),
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
