use serde::Serialize;

use crate::loader::Registry;

use super::graph::Graph;
use super::repr::{GraphRepr, LinkSource, SystemRepr};
use super::{GraphId, NodeId, ParamId, Result, Value};

use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{self, JoinHandle};

#[derive(Default)]
pub struct System {
    runners: Vec<(GraphId, Runner)>,
}

impl System {
    pub fn command(&mut self, graph: GraphId, command: Command) -> Result<Response> {
        let v = self
            .runners
            .iter_mut()
            .find(|x| x.0 == graph)
            .ok_or("not found")?;
        v.1.command(command)
    }

    pub fn from_repr(repr: SystemRepr, loader: &Registry) -> Result<Self> {
        let mut channels: HashMap<_, _> = repr
            .graphs
            .iter()
            .map(|x| {
                let id = GraphId::try_from(&x.name).unwrap();
                let (sender, receiver) = channel::<(Command, Message)>();
                (id, (sender, Some(receiver)))
            })
            .collect();

        let senders: HashMap<_, _> = channels.iter().map(|(k, (s, _))| (*k, s.clone())).collect();

        let v = repr
            .graphs
            .into_iter()
            .map(|x| {
                let id = GraphId::try_from(&x.name).unwrap();

                let (sender, receiver) = channels.get_mut(&id).unwrap();

                let receiver = receiver.take().unwrap();

                let runner = Runner::try_from_repr(
                    x,
                    loader.clone(),
                    (sender.clone(), receiver),
                    senders.clone(),
                )
                .unwrap();
                Ok((id, runner))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self { runners: v })
    }

    pub fn keys(&self) -> impl Iterator<Item = &GraphId> {
        self.runners.iter().map(|(v, _)| v)
    }
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
                    let (sender, receiver) = channel::<Result<Response>>();

                    let r = match s.send((Command::Dump(*n0, *p0), Message { sender, resp: None }))
                    {
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

impl Runner {
    pub fn try_from_repr(
        mut repr: GraphRepr,
        reg: Registry,
        (sender, receiver): (Sender<(Command, Message)>, Receiver<(Command, Message)>),
        senders: HashMap<GraphId, Sender<(Command, Message)>>,
    ) -> Result<Self> {
        Ok(Self {
            thread: Some(thread::spawn(move || {
                let external = repr
                    .links
                    .iter()
                    .filter(|x| matches!(x.src, LinkSource::External(_, _, _)))
                    .map(|x| match &x.src {
                        LinkSource::Internal(_, _) => unreachable!(),
                        LinkSource::External(g, n, p) => {
                            let g = GraphId::try_from(g.as_str()).unwrap();
                            let n = NodeId::try_from(n.as_str()).unwrap();
                            let p = ParamId::try_from(p.as_str()).unwrap();

                            let d0 = NodeId::try_from(&x.dst.0).unwrap();
                            let d1 = ParamId::try_from(&x.dst.1).unwrap();

                            ((senders[&g].clone(), n, p), (d0, d1))
                        }
                    })
                    .collect::<Vec<_>>();

                repr.links
                    .retain(|x| matches!(x.src, LinkSource::External(_, _, _)));
                let graph = Graph::try_from_repr(repr, &reg).expect("Cannot build graph");

                InternalRunner {
                    graph,
                    receiver,
                    external,
                }
                .run();
            })),
            sender,
        })
    }

    pub fn command(&mut self, command: Command) -> Result<Response> {
        let (sender, receiver) = channel::<Result<Response>>();

        match self.sender.send((command, Message { sender, resp: None })) {
            Ok(()) => receiver.recv().unwrap_or(Err("Error unwrapping")),
            Err(_) => Err("Cannot send"),
        }
    }
}

impl Drop for Runner {
    fn drop(&mut self) {
        if let Some(t) = self.thread.take() {
            let (sender, _receiver) = channel::<Result<Response>>();

            self.sender
                .send((Command::Kill, Message { sender, resp: None }))
                .expect("cannot send");

            t.join().expect("cannot join");
        }
    }
}
