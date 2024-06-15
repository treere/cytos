use crate::loader::GraphRepr;

use super::graph::Graph;
use super::{Dumper, Result, Value};

use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{self, JoinHandle};

pub enum Command {
    Start,
    Stop,
    Status,
    ListNodes,
    ListInputs(String),
    ListOutputs(String),
    Dump(String, String),
    Load(String, String, Value),
    Listener(Vec<(String, String)>),
}

pub enum Response {
    Data(Result<Value>),
    Receiver(Receiver<Response>),
}

impl std::fmt::Debug for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Response::Data(Ok(data)) => {
                write!(f, "*  {}", serde_json::to_string_pretty(data).unwrap())
            }
            Response::Data(Err(reason)) => write!(f, "!: {:?}", reason),
            Response::Receiver(_) => write!(f, "   receiver"),
        }
    }
}

struct Message {
    sender: Sender<Response>,
    resp: Option<Response>,
}

impl Message {
    fn set_resp(&mut self, resp: Result<impl Into<Value>>) {
        self.resp = Some(Response::Data(resp.map(|v| v.into())));
    }

    fn set_listener(&mut self, recv: Receiver<Response>) {
        self.resp = Some(Response::Receiver(recv));
    }
}

impl Drop for Message {
    fn drop(&mut self) {
        let resp = self.resp.take().unwrap_or(Response::Data(Ok(Value::Null)));
        self.sender.send(resp).expect("cannot send");
    }
}

struct Listener {
    sender: Sender<Response>,
    dumpers: Vec<Dumper>,
}

impl Listener {
    fn send(&self, data: Result<impl Into<Value>>) -> Result<()> {
        self.sender
            .send(Response::Data(data.map(|v| v.into())))
            .map_err(|_| "cannot send")
    }
}

pub struct Runner {
    thread: JoinHandle<()>,
    sender: Sender<(Command, Message)>,
}

impl Runner {
    pub fn new(repr: GraphRepr, reg: crate::loader::Registry) -> Self {
        let (sender, receiver) = channel::<(Command, Message)>();
        Self {
            thread: thread::spawn(move || {
                let mut listeners: Vec<Listener> = Vec::new();
                {
                    let mut graph = repr.build(&reg).expect("Cannot build graph");
                    {
                        loop {
                            while let Ok((command, mut message)) = receiver.recv() {
                                match command {
                                    Command::Start => break,
                                    Command::Status => message.set_resp(Ok("Idle")),
                                    Command::Listener(nodes) => {
                                        let (s, r) = channel::<Response>();
                                        message.set_listener(r);

                                        let dumpers = nodes
                                            .iter()
                                            .map(|(n, p)| graph.dump((n, p)).unwrap())
                                            .collect();
                                        listeners.push(Listener { sender: s, dumpers })
                                    }
                                    _ => {
                                        Self::dispatch_command(command, &mut message, &mut graph);
                                    }
                                }
                            }

                            graph.initialize().expect("cannot initialize");
                            'outer: loop {
                                while let Ok((command, mut message)) = receiver.try_recv() {
                                    match command {
                                        Command::Stop => break 'outer,
                                        Command::Status => message.set_resp(Ok("Running")),
                                        Command::Listener(nodes) => {
                                            let (s, r) = channel::<Response>();
                                            message.set_listener(r);

                                            let dumpers = nodes
                                                .iter()
                                                .map(|(n, p)| graph.dump((n, p)).unwrap())
                                                .collect();
                                            listeners.push(Listener { sender: s, dumpers })
                                        }

                                        _ => {
                                            Self::dispatch_command(
                                                command,
                                                &mut message,
                                                &mut graph,
                                            );
                                        }
                                    }
                                }

                                graph.step().expect("cannot step");
                                for l in &listeners {
                                    let data: Result<Vec<_>> =
                                        l.dumpers.iter().map(|x| x.dump()).collect();
                                    l.send(data).expect("cannot sent to listener");
                                }
                            }
                            graph.terminate().expect("cannot terminate");
                        }
                    }
                };
            }),
            sender,
        }
    }

    pub fn join(self) {
        self.thread.join().expect("Cannot join");
    }

    pub fn command(&mut self, command: Command) -> Response {
        let (sender, receiver) = channel::<Response>();

        match self.sender.send((command, Message { sender, resp: None })) {
            Ok(()) => receiver
                .recv()
                .unwrap_or(Response::Data(Err("Error unwrapping"))),
            Err(_) => Response::Data(Err("Cannot send")),
        }
    }

    fn dispatch_command(command: Command, message: &mut Message, graph: &mut Graph) {
        match command {
            Command::Start => (),
            Command::Stop => (),
            Command::Status => (),
            Command::Listener(_) => (),
            Command::ListNodes => message.set_resp(Ok(graph.list_nodes())),
            Command::ListInputs(node) => message.set_resp(graph.list_node_inputs(&node)),
            Command::ListOutputs(node) => message.set_resp(graph.list_node_outputs(&node)),
            Command::Dump(node, param) => {
                message.set_resp(graph.dump((&node, &param)).and_then(|x| x.dump()))
            }
            Command::Load(node, param, value) => {
                message.set_resp(graph.load((&node, &param), value))
            }
        }
    }
}
