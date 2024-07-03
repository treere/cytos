use anyhow::anyhow;

use easy_repl::{command, Command, CommandStatus, Repl};

use proph::architecture::runner::{Command as RCommand, Response, Runner};
use proph::architecture::Value;
use proph::loader::{GraphRepr, Registry};

use proph::utils::{
    convert_val_to_nodeid_string, convert_val_to_paramid_string, string_to_nodeid,
    string_to_paramid,
};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;

use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::RecvTimeoutError;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

struct Listener {
    _handler: JoinHandle<()>,
    run: Arc<AtomicBool>,
    description: String,
}

impl Drop for Listener {
    fn drop(&mut self) {
        self.run.store(false, Ordering::Release);
    }
}

#[derive(Default)]
struct Status {
    graphs: HashMap<String, Runner>,
    listeners: HashMap<String, Listener>,
    libraries: HashSet<String>,
}

fn graph_list(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "List graphs",
        () => ||{
            let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;
            let names :Vec<_>= status.graphs.keys().collect();
            println!("{:?}", names);
            Ok(CommandStatus::Done)
        }
    }
}

fn graph_remove(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Remove a graph",
        (graph: String) => |graph| {
            let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;

            let runner = status.graphs.remove(&graph).ok_or(anyhow!("not found"))?;
            status.listeners.remove(&graph);
            drop(runner);

            println!("removed!");
            Ok(CommandStatus::Done)
        }
    }
}

fn graph_load(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
    "Load a graph from a configuration",
    (graph: String, filename: String) =>   |graph, filename| {
        let mut configuration = String::new();

        File::open(filename)?.read_to_string(&mut configuration)?;
        let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;

        let mut registry = Registry::default();
        for lib in status.libraries.iter() {
            registry.load_library(lib).or(Err(anyhow!("cannot load library")))?;
        }

        let repr = GraphRepr::from_json(&configuration).map_err(|x| anyhow!(x))?;
        let mut runner = Runner::new(repr,registry);

        status.graphs.insert(graph, runner);

        println!("loaded!");

        Ok(CommandStatus::Done)
    }}
}

fn graph_start(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Start a graph",
        (graph: String) => |graph| {
        let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;
        let result = status
            .graphs
            .get_mut(&graph)
            .ok_or(anyhow!("missing graph"))?
            .command(RCommand::Start);
        println!("{:?}", result);

        Ok(CommandStatus::Done)
    }}
}
fn graph_stop(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Step a graph",
        (graph: String) => |graph| {
            let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;
            let result = status
            .graphs
            .get_mut(&graph)
            .ok_or(anyhow!("missing graph"))?
            .command(RCommand::Stop);

            println!("{:?}", result);

            Ok(CommandStatus::Done)
        }
    }
}

fn graph_status(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Step a graph",
        (graph: String) => |graph| {
            let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;
            let result = status
            .graphs
            .get_mut(&graph)
            .ok_or(anyhow!("missing graph"))?
            .command(RCommand::Status);
            println!("{:?}", result);

            Ok(CommandStatus::Done)
        }
    }
}

fn library_list(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "List libraries",
        () => || {
            let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;
            for lib in status.libraries.iter() {
                println!("{}", lib);
            }
            Ok(CommandStatus::Done)
        }
    }
}

fn library_add(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Add a library",
        (library: String) => |library: String| {
            let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;
            let mut registry = Registry::default();
            registry.load_library(&library).or(Err(anyhow!("cannot load library")))?;

            for f in registry.list_factories() {
                println!("{}", f);
            }

            status.libraries.insert(library);
            println!("added");
            Ok(CommandStatus::Done)
        }
    }
}

fn library_remove(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Remove a library",
        (library: String) => |library| {
            let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;
            status.libraries.remove(&library);
            println!("removed");
            Ok(CommandStatus::Done)
        }
    }
}

fn library_inspect() -> Command<'static> {
    command! {
        "Inspect a library",
        (library: String) => |library:String| {

            let mut registry = Registry::default();
            registry.load_library(&library).or(Err(anyhow!("cannot load library")))?;

            for f in registry.list_factories() {
                println!("{}", f);
            }

            Ok(CommandStatus::Done)
        }
    }
}

fn node_list(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "List graph nodes",
        (graph: String) => |graph| {
            let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;
            let result = status
                .graphs
                .get_mut(&graph)
                .ok_or(anyhow!("missing graph"))?
            .command(RCommand::ListNodes);

            if let Ok(Response::Data(val))  = result {
                let result  = convert_val_to_nodeid_string(val).map_err(|x|anyhow!(x))?;
                println!("{:?}", result);
                Ok(CommandStatus::Done)
            }
            else {
                Err(anyhow!("Invalid response"))
            }
        }
    }
}

fn node_inputs(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "List inputs of a graph node",
        (graph: String, node: String) => |graph, node| {
            let node = string_to_nodeid(node).map_err(|x| anyhow!(x))?;
            let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;
            let result = status
            .graphs
            .get_mut(&graph)
            .ok_or(anyhow!("missing graph"))?
            .command(RCommand::ListInputs(node));

            if let Ok(Response::Data(val))  = result {
                let result = convert_val_to_paramid_string(val).map_err(|x| anyhow!(x))?;
                println!("{:?}", result);
                Ok(CommandStatus::Done)
            }
            else {
                Err(anyhow!("Invalid response"))
            }
        }
    }
}

fn node_outputs(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "List outputs of a graph node",
        (graph: String, node: String) => |graph, node| {
            let node = string_to_nodeid(node).map_err(|x| anyhow!(x))?;
            let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;
            let result = status
            .graphs
            .get_mut(&graph)
            .ok_or(anyhow!("missing graph"))?
            .command(RCommand::ListOutputs(node));

            if let Ok(Response::Data(val))  = result {
                let result  = convert_val_to_nodeid_string(val).map_err(|x|anyhow!(x))?;
                println!("{:?}", result);
                Ok(CommandStatus::Done)
            }
            else {
                Err(anyhow!("Invalid response"))
            }
        }
    }
}

fn node_dump(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Dump a input/output of a graph node",
        (graph: String, node: String, param: String) => |graph, node, param| {
            let node = string_to_nodeid(node).map_err(|x| anyhow!(x))?;
            let param = string_to_paramid(param).map_err(|x| anyhow!(x))?;
            let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;
            let result = status
            .graphs
            .get_mut(&graph)
            .ok_or(anyhow!("missing graph"))?
            .command(RCommand::Dump(node,param));

            println!("{:?}", result);

            Ok(CommandStatus::Done)
        }
    }
}

fn node_load(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Load to an input/output of a graph node",
        (graph: String, node: String, param: String, value: String) => |graph, node, param, value| {
            let node = string_to_nodeid(node).map_err(|x| anyhow!(x))?;
            let param = string_to_paramid(param).map_err(|x| anyhow!(x))?;
            let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;
            let value = Value::from_string(value).map_err(|x| anyhow!(x))?;

            let result = status
            .graphs
            .get_mut(&graph)
            .ok_or(anyhow!("missing graph"))?
            .command(RCommand::Load(node,param, value));

            println!("{:?}", result);

            Ok(CommandStatus::Done)
        }
    }
}

fn listener_list(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "List graphs",
        () => ||{
            let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;
            let names :Vec<_>= status.listeners.iter().map(|(k,v)| (k, &v.description)).collect();
            println!("{:?}", names);
            Ok(CommandStatus::Done)
        }
    }
}

fn listener_add(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Listen some nodes",
        (graph: String, description: String) => |graph, description: String| {
            let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;
            let nodes = description
            .split("|")
            .map(|g| {
                let p = g.split(":").collect::<Vec<_>>();
                if p.len() != 2 {
                    Err("Malformed str")
                }
                else {
                    let nodeid =  string_to_paramid(p[0]).map_err(|x| anyhow!(x)).unwrap();
                    let paramid =  string_to_paramid(p[1]).map_err(|x| anyhow!(x)).unwrap();
                    Ok((nodeid, paramid))
                }
            }).collect::<Result<Vec<_>,_>>().map_err(|x| anyhow!(x))?;

            let runner = status
            .graphs
            .get_mut(&graph)
            .ok_or(anyhow!("missing graph"))?;

            if let Ok(Response::Receiver(result)) = runner.command(RCommand::Listener(nodes)) {
                let run = Arc::new(AtomicBool::new(true));
                let r1 = run.clone();
                let handler = thread::spawn(move || {
                    while r1.load(Ordering::Relaxed) {
                        match result.recv_timeout(Duration::from_millis(10)) {
                            Ok(Ok(r)) => println!("{:?}", r),
                            Ok(Err(r)) => println!("{:?}", r),
                            Err(RecvTimeoutError::Timeout) => continue,
                            Err(RecvTimeoutError::Disconnected) => break
                        }
                    }
                });
                status.listeners.insert(graph, Listener{_handler: handler, run, description});
                println!("added");
                Ok(CommandStatus::Done)
            }
            else {
                Err(anyhow!("Invalid return value"))
            }

        }
    }
}

fn listener_remove(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Listen some nodes",
        (graph: String) => |graph| {
            let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;
            status.listeners.remove(&graph).ok_or(anyhow!("not found"))?;
            println!("removed");
            Ok(CommandStatus::Done)
        }
    }
}

fn main() -> Result<(), &'static str> {
    let status = Rc::new(Mutex::new(Status::default()));

    Repl::builder()
        .with_filename_completion(true)
        .add("graph_list", graph_list(status.clone()))
        .add("graph_load", graph_load(status.clone()))
        .add("graph_remove", graph_remove(status.clone()))
        .add("graph_start", graph_start(status.clone()))
        .add("graph_stop", graph_stop(status.clone()))
        .add("graph_status", graph_status(status.clone()))
        .add("library_list", library_list(status.clone()))
        .add("library_add", library_add(status.clone()))
        .add("library_remove", library_remove(status.clone()))
        .add("library_inspect", library_inspect())
        .add("node_list", node_list(status.clone()))
        .add("node_inputs", node_inputs(status.clone()))
        .add("node_outputs", node_outputs(status.clone()))
        .add("node_dump", node_dump(status.clone()))
        .add("node_load", node_load(status.clone()))
        .add("listener_list", listener_list(status.clone()))
        .add("listener_add", listener_add(status.clone()))
        .add("listener_remove", listener_remove(status.clone()))
        .add(
            "exit",
            command! { "Exit program", () => || Ok(CommandStatus::Quit) },
        )
        .build()
        .or(Err("Failed to create repl"))
        .and_then(|mut repl| repl.run().or(Err("Critical REPL error")))
}
