use anyhow::anyhow;

use easy_repl::{command, Command, CommandStatus, Repl};

use proph::architecture::load_value_from_string;
use proph::architecture::runner::{Command as RCommand, Response, Runner};
use proph::loader::{GraphRepr, Registry};

use proph_transformers::{
    AddValue, GrayScale, ImageDecoder, IncrementalGenerator, Mean, Print, Rscam, ZuneImageDecoder,
};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use std::rc::Rc;
use std::sync::Mutex;
use std::thread::{self, JoinHandle};

fn load_registry() -> Registry {
    Registry::default()
        .add("AddValue", AddValue::default)
        .add("ImageDecoder", ImageDecoder::default)
        .add("ImageGrayScale", GrayScale::default)
        .add("ImageMean", Mean::default)
        .add("IncrementalGenerator", IncrementalGenerator::default)
        .add("PrintF64", Print::<f64>::default)
        .add("PrintU64", Print::<u64>::default)
        .add("Rscam", Rscam::default)
        .add("ZuneImageDecoder", ZuneImageDecoder::default)
}

#[derive(Default)]
struct Status {
    graphs: HashMap<String, Runner>,
    listeners: HashMap<String, JoinHandle<()>>,
}

fn graph_list(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "List graphs",
        () => ||{
            let mut status = status.lock().map_err(|_| anyhow!("cannot lock"))?;
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
            let mut status = status.lock().map_err(|_| anyhow!("cannot lock"))?;

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

        let loader = load_registry();
        let repr = GraphRepr::from_json(&configuration).map_err(|x| anyhow!(x))?;
        let mut runner = Runner::new(repr,loader);

        let mut status = status.lock().map_err(|_| anyhow!("cannot lock"))?;
        status.graphs.insert(graph, runner);
        println!("loaded!");

        Ok(CommandStatus::Done)
    }}
}

fn graph_start(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Start a graph",
        (graph: String) => |graph| {
        let mut status = status.lock().map_err(|_| anyhow!("cannot lock"))?;
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
            let mut status = status.lock().map_err(|_| anyhow!("cannot lock"))?;
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
            let mut status = status.lock().map_err(|_| anyhow!("cannot lock"))?;
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

fn node_list(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "List graph nodes",
        (graph: String) => |graph| {
            let mut status = status.lock().map_err(|_| anyhow!("cannot lock"))?;
            let result = status
                .graphs
                .get_mut(&graph)
                .ok_or(anyhow!("missing graph"))?
                .command(RCommand::ListNodes);
            println!("{:?}", result);

            Ok(CommandStatus::Done)
        }
    }
}

fn node_inputs(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "List inputs of a graph node",
        (graph: String, node: String) => |graph, node| {
            let mut status = status.lock().map_err(|_| anyhow!("cannot lock"))?;
            let result = status
            .graphs
            .get_mut(&graph)
            .ok_or(anyhow!("missing graph"))?
            .command(RCommand::ListInputs(node));

            println!("{:?}", result);


            Ok(CommandStatus::Done)
        }
    }
}

fn node_outputs(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "List outputs of a graph node",
        (graph: String, node: String) => |graph, node| {
            let mut status = status.lock().map_err(|_| anyhow!("cannot lock"))?;
            let result = status
            .graphs
            .get_mut(&graph)
            .ok_or(anyhow!("missing graph"))?
            .command(RCommand::ListOutputs(node));

            println!("{:?}", result);

            Ok(CommandStatus::Done)
        }
    }
}

fn node_dump(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Dump a input/output of a graph node",
        (graph: String, node: String, param: String) => |graph, node, param| {
            let mut status = status.lock().map_err(|_| anyhow!("cannot lock"))?;
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
            let mut status = status.lock().map_err(|_| anyhow!("cannot lock"))?;
            let value = load_value_from_string(value).map_err(|_| anyhow!("cannot parse value"))?;

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
            let mut status = status.lock().map_err(|_| anyhow!("cannot lock"))?;
            let names :Vec<_>= status.listeners.keys().collect();
            println!("{:?}", names);
            Ok(CommandStatus::Done)
        }
    }
}

fn listener_add(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Listen some nodes",
        (graph: String, nodes: String) => |graph, nodes: String| {
            let mut status = status.lock().map_err(|_| anyhow!("cannot lock"))?;
            let nodes = nodes
            .split("|")
            .map(|g| {
                let p = g.split(":").collect::<Vec<_>>();
                if p.len() != 2 {
                    Err("Malformed str")
                }
                else {
                    Ok((p[0].to_owned(),p[1].to_owned()))
                }
            }).collect::<Result<Vec<_>,_>>().map_err(|_| anyhow!("Invalid string"))?;

            let runner = status
            .graphs
            .get_mut(&graph)
            .ok_or(anyhow!("missing graph"))?;

            if let Response::Receiver(result) = runner.command(RCommand::Listener(nodes)) {
                status.listeners.insert(graph, thread::spawn( move || {
                    loop {
                        let r = result.recv().expect("Cannot receive");
                        println!("{:?}", r);
                    }
                }));
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
            let mut status = status.lock().map_err(|_| anyhow!("cannot lock"))?;
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
        .add("node_list", node_list(status.clone()))
        .add("node_inputs", node_inputs(status.clone()))
        .add("node_outputs", node_outputs(status.clone()))
        .add("node_dump", node_dump(status.clone()))
        .add("node_load", node_load(status.clone()))
        .add("listener_list", listener_list(status.clone()))
        .add("listener_add", listener_add(status.clone()))
        .add("listener_remove", listener_remove(status.clone()))
        .build()
        .or(Err("Failed to create repl"))
        .and_then(|mut repl| repl.run().or(Err("Critical REPL error")))
}
