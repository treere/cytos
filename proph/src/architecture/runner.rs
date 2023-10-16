use super::graph::Graph;

use std::sync::mpsc::{channel, Sender};
use std::thread::{self, JoinHandle};

pub enum Command {}

pub enum Response {
    A,
}

struct Message {
    command: Command,
    resp: Sender<Response>,
}

impl Command {
    fn execute(self, _graph: &mut Graph) {}
}

pub struct Runner {
    graph: JoinHandle<Graph>,
    sender: Sender<Message>,
}

impl Runner {
    pub fn run(&mut self, mut graph: Graph) -> Self {
        let (sender, receiver) = channel::<Message>();
        Self {
            graph: thread::spawn(move || {
                graph.initialize().unwrap();
                loop {
                    while let Ok(message) = receiver.try_recv() {
                        message.command.execute(&mut graph);
                        message.resp.send(Response::A).unwrap();
                    }

                    if let Err(_) = graph.step() {
                        break;
                    }
                }
                graph.terminate().unwrap();
                graph
            }),
            sender,
        }
    }

    pub fn join(self) -> Graph {
        self.graph.join().unwrap()
    }

    pub fn command(&mut self, command: Command) {
        let (sender, receiver) = channel::<Response>();

        let _ = self.sender.send(Message {
            command,
            resp: sender,
        });
        receiver.recv().unwrap();
    }
}
