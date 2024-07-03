use serde::Serialize;

use crate::loader::GraphRepr;

use super::graph::Graph;
use super::{Dumper, NodeId, ParamId, Result, Value};

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
    Listener(Vec<(NodeId, ParamId)>),
}

pub enum Response {
    Data(Value),
    Receiver(Receiver<Result<Response>>),
}

impl std::fmt::Debug for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Response::Data(data) => {
                write!(f, "*  {:?}", data)
            }
            Response::Receiver(_) => write!(f, "   receiver"),
        }
    }
}

struct Message {
    sender: Sender<Result<Response>>,
    resp: Option<Result<Response>>,
}

impl Message {
    fn set_resp<T: Serialize>(&mut self, resp: Result<T>) {
        self.resp = Some(resp.and_then(|v| Value::from_t(&v).map(Response::Data)));
    }

    fn set_listener(&mut self, recv: Receiver<Result<Response>>) {
        self.resp = Some(Ok(Response::Receiver(recv)));
    }
}

impl Drop for Message {
    fn drop(&mut self) {
        let null = Value::from_t(&()).unwrap();
        let resp = self.resp.take().unwrap_or(Ok(Response::Data(null)));
        self.sender.send(resp).expect("cannot send");
    }
}

struct Listener {
    sender: Sender<Result<Response>>,
    dumpers: Vec<Dumper>,
}

impl Listener {
    fn send<T: Serialize>(&self, data: Result<T>) -> Result<()> {
        self.sender
            .send(data.and_then(|v| Value::from_t(&v).map(Response::Data)))
            .or(Err("cannot send"))
    }
}

struct InternalRunner {
    listeners: Vec<Listener>,
    graph: Graph,
    receiver: Receiver<(Command, Message)>,
}

impl InternalRunner {
    fn run(mut self) {
        'main: loop {
            while let Ok((command, mut message)) = self.receiver.recv() {
                match command {
                    Command::Kill => break 'main,
                    Command::Start => break,
                    Command::Status => message.set_resp(Ok("Idle")),
                    _ => self.dispatch_command(command, &mut message),
                }
            }

            self.graph.initialize().expect("cannot initialize");
            'outer: loop {
                while let Ok((command, mut message)) = self.receiver.try_recv() {
                    match command {
                        Command::Kill => break 'main,
                        Command::Stop => break 'outer,
                        Command::Status => message.set_resp(Ok("Running")),
                        _ => self.dispatch_command(command, &mut message),
                    }
                }

                self.graph.step().expect("cannot step");
                self.notify_listeners();
            }
            self.graph.terminate().expect("cannot terminate");
        }
    }

    fn notify_listeners(&mut self) {
        self.listeners.retain(|l| {
            let data: Result<Vec<_>> = l.dumpers.iter().map(|x| x.dump()).collect();
            l.send(data).is_ok()
        });
    }

    fn dispatch_command(&mut self, command: Command, message: &mut Message) {
        match command {
            Command::ListNodes => message.set_resp(Ok(self.graph.list_nodes())),
            Command::ListInputs(node) => message.set_resp(self.graph.list_node_inputs(node)),
            Command::ListOutputs(node) => message.set_resp(self.graph.list_node_outputs(node)),
            Command::Dump(node, param) => {
                message.set_resp(self.graph.dumper_for((node, param)).and_then(|x| x.dump()))
            }
            Command::Load(node, param, value) => {
                message.set_resp(self.graph.load((node, param), value))
            }
            Command::Listener(nodes) => {
                let (s, r) = channel::<Result<Response>>();

                let dumpers: Result<Vec<_>> = nodes
                    .into_iter()
                    .map(|(n, p)| self.graph.dumper_for((n, p)))
                    .collect();

                match dumpers {
                    Ok(dumpers) => {
                        message.set_listener(r);
                        self.listeners.push(Listener { sender: s, dumpers })
                    }
                    Err(_) => message.set_resp(Err::<Value, _>("cannot load dumpers")),
                }
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
    pub fn new(repr: GraphRepr, reg: crate::loader::Registry) -> Self {
        let (sender, receiver) = channel::<(Command, Message)>();
        Self {
            thread: Some(thread::spawn(move || {
                InternalRunner {
                    listeners: Vec::new(),
                    graph: repr.build(&reg).expect("Cannot build graph"),
                    receiver,
                }
                .run()
            })),
            sender,
        }
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
