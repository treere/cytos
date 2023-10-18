use super::graph::Graph;

use std::sync::mpsc::{channel, Sender};
use std::thread::{self, JoinHandle};

pub enum Command {
    Start,
    Stop,
}

pub enum Response {
    A,
}

struct Message {
    command: Command,
    resp: Sender<Response>,
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
                while let Ok(Message { command, .. }) = receiver.recv() {
                    match command {
                        Command::Start => break,
                        Command::Stop => todo!(),
                    }
                }

                graph.initialize().unwrap();
                'outer: loop {
                    while let Ok(message) = receiver.try_recv() {
                        match message.command {
                            Command::Start => (),
                            Command::Stop => break 'outer,
                        };

                        message.resp.send(Response::A).unwrap();
                    }

                    if let Err(_) = graph.step() {
                        break;
                    }
                }
                graph.terminate().unwrap();
            }),
            sender,
        }
    }

    pub fn join(self) -> () {
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
