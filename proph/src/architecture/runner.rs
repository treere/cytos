use serde::Serialize;

use crate::loader::{GraphRepr, Registry};

use super::graph::Graph;
use super::{GraphId, NodeId, ParamId, Result, Value};

use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{self, JoinHandle};

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

type ExternalReference = (GraphId, NodeId, ParamId);

type InternalReference = (NodeId, ParamId);

struct InternalRunner {
    graph: Graph,
    receiver: Receiver<(Command, Message)>,

    _external: Vec<(ExternalReference, InternalReference)>,
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

pub struct Runner {
    thread: Option<JoinHandle<()>>,
    sender: Sender<(Command, Message)>,
}

impl Runner {
    pub fn from_repr(repr: GraphRepr, reg: Registry) -> Result<(GraphId, Self)> {
        let (sender, receiver) = channel::<(Command, Message)>();
        Ok((
            repr.id()?,
            Self {
                thread: Some(thread::spawn(move || {
                    let graph = repr.build(&reg).expect("Cannot build graph");
                    InternalRunner { graph, receiver, _external: Vec::new() }.run();
                })),
                sender,
            },
        ))
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
