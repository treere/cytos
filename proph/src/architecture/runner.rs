use super::graph::Graph;

use std::sync::mpsc::{channel, Sender};
use std::thread::{self, JoinHandle};

pub enum Command {
    Start,
    Stop,
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
}

struct Message {
    command: Command,
    sender: Sender<Response>,
    resp: Option<Response>,
}

impl Message {
    fn command(&self) -> &Command {
        &self.command
    }

    fn set_resp(&mut self, resp: Response) {
        self.resp = Some(resp);
    }
}

impl Drop for Message {
    fn drop(&mut self) {
        let resp = self.resp.take().unwrap_or(Response::None);
        self.sender.send(resp).expect("cannot send");
    }
}

pub struct Runner {
    graph: JoinHandle<()>,
    sender: Sender<Message>,
}

impl Runner {
    pub fn new(mut graph: Graph) -> Self {
        let (sender, receiver) = channel::<Message>();
        Self {
            graph: thread::spawn(move || loop {
                while let Ok(mut message) = receiver.recv() {
                    match message.command() {
                        Command::Start => {
                            message.set_resp(Response::Ok);
                            break;
                        }
                        Command::Stop => (),
                        Command::ListNodes => message.set_resp(Response::List(graph.list_nodes())),
                        Command::ListInputs(node) => message.set_resp(Response::List(
                            graph.list_node_inputs(node.to_string()).unwrap(),
                        )),
                        Command::ListOutputs(node) => message.set_resp(Response::List(
                            graph.list_node_outputs(node.to_string()).unwrap(),
                        )),
                        Command::Dump(node, param) => message.set_resp(Response::Data(
                            graph.dump((node.to_string(), param.to_string())).unwrap(),
                        )),
                    }
                }

                graph.initialize().unwrap();
                'outer: loop {
                    while let Ok(mut message) = receiver.try_recv() {
                        match message.command() {
                            Command::Start => (),
                            Command::Stop => {
                                message.set_resp(Response::Ok);
                                break 'outer;
                            }
                            Command::ListNodes => {
                                message.set_resp(Response::List(graph.list_nodes()))
                            }
                            Command::ListInputs(node) => message.set_resp(Response::List(
                                graph.list_node_inputs(node.to_string()).unwrap(),
                            )),
                            Command::ListOutputs(node) => message.set_resp(Response::List(
                                graph.list_node_outputs(node.to_string()).unwrap(),
                            )),
                            Command::Dump(node, param) => message.set_resp(Response::Data(
                                graph.dump((node.to_string(), param.to_string())).unwrap(),
                            )),
                        };
                    }

                    if graph.step().is_err() {
                        break;
                    }
                }
                graph.terminate().unwrap();
            }),
            sender,
        }
    }

    pub fn join(self) {
        self.graph.join().unwrap()
    }

    pub fn command(&mut self, command: Command) -> Result<Response, &'static str> {
        let (sender, receiver) = channel::<Response>();

        let _ = self.sender.send(Message {
            command,
            sender,
            resp: None,
        });
        receiver.recv().map_err(|_| "Error unwrapping")
    }
}
