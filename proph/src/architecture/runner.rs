use crate::loader::GraphRepr;

use super::graph::Graph;
use super::Value;

use std::sync::mpsc::{channel, Sender};
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
}

pub type Response = Result<Value, &'static str>;

struct Message {
    sender: Sender<Response>,
    resp: Option<Response>,
}

impl Message {
    fn set_resp(&mut self, resp: Result<impl Into<Value>, &'static str>) {
        self.resp = Some(resp.map(|v| v.into()));
    }
}

impl Drop for Message {
    fn drop(&mut self) {
        let resp = self.resp.take().unwrap_or(Ok(Value::Null));
        self.sender.send(resp).expect("cannot send");
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
                let mut graph = repr.build(&reg).expect("Cannot build graph");
                {
                    loop {
                        while let Ok((command, mut message)) = receiver.recv() {
                            match command {
                                Command::Start => break,
                                Command::Status => message.set_resp(Ok("Idle")),
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
                                    _ => {
                                        Self::dispatch_command(command, &mut message, &mut graph);
                                    }
                                }
                            }

                            graph.step().expect("cannot step")
                        }
                        graph.terminate().expect("cannot terminate");
                    }
                }
            }),
            sender,
        }
    }

    pub fn join(self) {
        self.thread.join().expect("Cannot join");
    }

    pub fn command(&mut self, command: Command) -> Response {
        let (sender, receiver) = channel::<Response>();

        let _ = self.sender.send((command, Message { sender, resp: None }));
        receiver.recv().unwrap_or(Response::Err("Error unwrapping"))
    }

    fn dispatch_command(command: Command, message: &mut Message, graph: &mut Graph) {
        match command {
            Command::Start => (),
            Command::Stop => (),
            Command::Status => (),
            Command::ListNodes => message.set_resp(Ok(graph.list_nodes())),
            Command::ListInputs(node) => message.set_resp(graph.list_node_inputs(&node)),
            Command::ListOutputs(node) => message.set_resp(graph.list_node_outputs(&node)),
            Command::Dump(node, param) => message.set_resp(graph.dump((&node, &param))),
            Command::Load(node, param, value) => {
                message.set_resp(graph.load((&node, &param), value))
            }
        }
    }
}
