use super::graph::Graph;

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
}

#[derive(Debug)]
pub enum Response {
    Ok,
    None,
    List(Vec<String>),
    Data(String),
    Error(&'static str),
}

impl From<String> for Response {
    fn from(value: String) -> Self {
        Response::Data(value)
    }
}

impl From<Vec<String>> for Response {
    fn from(value: Vec<String>) -> Self {
        Response::List(value)
    }
}

impl<T> From<Result<T, &'static str>> for Response
where
    T: Into<Response> + 'static,
{
    fn from(value: Result<T, &'static str>) -> Self {
        match value {
            Ok(x) => x.into(),
            Err(x) => Response::Error(x),
        }
    }
}

struct Message {
    sender: Sender<Response>,
    resp: Option<Response>,
}

impl Message {
    fn set_resp(&mut self, resp: impl Into<Response>) {
        self.resp = Some(resp.into());
    }
}

impl Drop for Message {
    fn drop(&mut self) {
        let resp = self.resp.take().unwrap_or(Response::None);
        self.sender.send(resp).expect("cannot send");
    }
}

pub struct Runner {
    thread: JoinHandle<()>,
    sender: Sender<(Command, Message)>,
}

impl Runner {
    pub fn new(mut graph: Graph) -> Self {
        let (sender, receiver) = channel::<(Command, Message)>();
        Self {
            thread: thread::spawn(move || loop {
                while let Ok((command, mut message)) = receiver.recv() {
                    match command {
                        Command::Start => {
                            message.set_resp(Response::Ok);
                            break;
                        }
                        Command::Status => message.set_resp("Idle".to_string()),
                        _ => {
                            Self::dispatch_command(command, &mut message, &graph);
                        }
                    }
                }

                graph.initialize().expect("cannot initialize");
                'outer: loop {
                    while let Ok((command, mut message)) = receiver.try_recv() {
                        match command {
                            Command::Stop => {
                                message.set_resp(Response::Ok);
                                break 'outer;
                            }
                            Command::Status => message.set_resp("Running".to_string()),
                            _ => {
                                Self::dispatch_command(command, &mut message, &graph);
                            }
                        }
                    }

                    graph.step().expect("cannot step")
                }
                graph.terminate().expect("cannot terminate");
            }),
            sender,
        }
    }

    pub fn join(self) {
        self.thread.join().expect("Cannot join");
    }

    pub fn command(&mut self, command: Command) -> Result<Response, &'static str> {
        let (sender, receiver) = channel::<Response>();

        let _ = self.sender.send((command, Message { sender, resp: None }));
        receiver.recv().map_err(|_| "Error unwrapping")
    }

    fn dispatch_command(command: Command, message: &mut Message, graph: &Graph) {
        match command {
            Command::Start => (),
            Command::Stop => (),
            Command::Status => (),
            Command::ListNodes => message.set_resp(graph.list_nodes()),
            Command::ListInputs(node) => message.set_resp(graph.list_node_inputs(&node)),
            Command::ListOutputs(node) => message.set_resp(graph.list_node_outputs(&node)),
            Command::Dump(node, param) => message.set_resp(graph.dump((&node, &param))),
        }
    }
}
