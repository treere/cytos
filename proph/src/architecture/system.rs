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
        serde_json::from_str(file).or(Err("cannot read file"))
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

#[derive(Debug)]
pub struct Response(pub Value);

pub struct Message {
    sender: Sender<Result<Response>>,
    resp: Option<Result<Response>>,
}

impl Message {
    fn new(sender: Sender<Result<Response>>) -> Self {
        Self { sender, resp: None }
    }

    fn set<T: Serialize>(&mut self, resp: Result<T>) {
        self.resp = Some(resp.and_then(|v| Value::from_t(&v).map(Response)));
    }
}

impl Drop for Message {
    fn drop(&mut self) {
        let null = Value::from_t(&()).unwrap();
        let resp = self.resp.take().unwrap_or(Ok(Response(null)));
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
            while let Ok((command, mut message)) = self.receiver.recv() {
                match command {
                    Command::Kill => break 'main,
                    Command::Start => break,
                    Command::Status => message.set(Ok("Idle")),
                    _ => self.dispatch_command(command, &mut message),
                }
            }

            self.graph.initialize().expect("cannot initialize");
            'outer: loop {
                while let Ok((command, mut message)) = self.receiver.try_recv() {
                    match command {
                        Command::Kill => break 'main,
                        Command::Stop => break 'outer,
                        Command::Status => message.set(Ok("Running")),
                        _ => self.dispatch_command(command, &mut message),
                    }
                }

                self.external.iter().for_each(|((s, n0, p0), (n1, p1))| {
                    let (sender, receiver) = bounded::<Result<Response>>(0);

                    let r = match s.send((Command::Dump(*n0, *p0), Message::new(sender))) {
                        Ok(()) => receiver.recv().unwrap_or(Err("Error unwrapping")),
                        Err(_) => Err("Cannot send"),
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

    fn dispatch_command(&mut self, command: Command, message: &mut Message) {
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
        let (sender, receiver) = bounded::<Result<Response>>(0);

        match self.sender.send((command, Message::new(sender))) {
            Ok(()) => receiver.recv().unwrap_or(Err("Error unwrapping")),
            Err(_) => Err("Cannot send"),
        }
    }
}

impl Drop for Runner {
    fn drop(&mut self) {
        if let Some(t) = self.thread.take() {
            let (sender, _receiver) = bounded::<Result<Response>>(0);

            self.sender
                .send((Command::Kill, Message::new(sender)))
                .expect("cannot send");

            t.join().expect("cannot join");
        }
    }
}
