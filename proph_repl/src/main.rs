use anyhow::anyhow;

use easy_repl::{command, Command, CommandStatus, Repl};
use proph::architecture::graph::Graph;
use proph::loader::{GraphRepr, Registry};

use proph_transformers::{
    AddValue, GrayScale, ImageDecoder, IncrementalGenerator, Mean, Print, Rscam,
};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};

fn load_registry() -> Registry {
    Registry::default()
        .add("IncrementalGenerator", IncrementalGenerator::default)
        .add("AddValue", AddValue::default)
        .add("Rscam", Rscam::default)
        .add("ImageDecoder", ImageDecoder::default)
        .add("ImageGrayScale", GrayScale::default)
        .add("ImageMean", Mean::default)
        .add("PrintU64", Print::<u64>::default)
        .add("PrintF64", Print::<f64>::default)
}

#[derive(Default)]
struct Status {
    graphs: HashMap<String, Graph>,
}

fn list_command(s: Arc<Mutex<Status>>) -> Command<'static> {
    command! {
        "List graphs",
        () => ||{
            let mut status = s.lock().map_err(|_| anyhow!("cannot lock"))?;
            for k in status.graphs.keys() {
                println!("{}", k);
            }
            Ok(CommandStatus::Done)
        }
    }
}

fn load_command(status: Arc<Mutex<Status>>) -> Command<'static> {
    command! {
    "Load a configuration",
    (name: String, filename: String) =>   |name, filename| {
        let mut configuration = String::new();

        File::open(filename)?.read_to_string(&mut configuration)?;

        let loader = load_registry();
        let mut graph = GraphRepr::load(&configuration, &loader).map_err(|x| anyhow!(x))?;

        let mut status = status.lock().map_err(|_| anyhow!("cannot lock"))?;
        status.graphs.insert(name, graph);

        Ok(CommandStatus::Done)
    }}
}

fn initialize_command(s: Arc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Initialize a loaded graph",
        (name: String) => |name| {
            let mut status = s.lock().map_err(|_| anyhow!("cannot lock"))?;
            if let Some(graph) = status.graphs.get_mut(&name) {
                let _ = graph.initialize();
            }

            Ok(CommandStatus::Done)
        }
    }
}

fn step_command(s: Arc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Step a loaded graph",
        (name: String) => |name| {
            let mut status = s.lock().map_err(|_| anyhow!("cannot lock"))?;
            if let Some(graph) = status.graphs.get_mut(&name) {
                let _ = graph.step();
            }

            Ok(CommandStatus::Done)
        }
    }
}

fn terminate_command(s: Arc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Terminate a loaded graph",
        (name: String) => |name| {
            let mut status = s.lock().map_err(|_| anyhow!("cannot lock"))?;
            if let Some(graph) = status.graphs.get_mut(&name) {
                let _ = graph.terminate();
            }

            Ok(CommandStatus::Done)
        }
    }
}

fn nodes_command(s: Arc<Mutex<Status>>) -> Command<'static> {
    command! {
        "List graph nodes",
        (name: String) => |name| {
            let mut status = s.lock().map_err(|_| anyhow!("cannot lock"))?;
            if let Some(graph) = status.graphs.get_mut(&name) {
                for n in  graph.list_nodes() {
                    println!("{}", n);
                }
            }

            Ok(CommandStatus::Done)
        }
    }
}

fn node_inputs_command(s: Arc<Mutex<Status>>) -> Command<'static> {
    command! {
           "List input of a graph nodes",
           (name: String, node: String) => |name, node| {
               let mut status = s.lock().map_err(|_| anyhow!("cannot lock"))?;
               if let Some(graph) = status.graphs.get_mut(&name) {
                   for n in  graph.list_node_inputs(node) {
                       println!("{}", n);
                   }
               }

               Ok(CommandStatus::Done)
           }
    }
}

fn node_outputs_command(s: Arc<Mutex<Status>>) -> Command<'static> {
    command! {
        "List output of a graph nodes",
        (name: String, node: String) => |name, node| {
            let mut status = s.lock().map_err(|_| anyhow!("cannot lock"))?;
            if let Some(graph) = status.graphs.get_mut(&name) {
                for n in  graph.list_node_outputs(node) {
                    println!("{}", n);
                }
            }

            Ok(CommandStatus::Done)
        }
    }
}

fn dump_command(s: Arc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Dump param value",
        (name: String, node: String, param: String) => |name, node, param| {
            let mut status = s.lock().map_err(|_| anyhow!("cannot lock"))?;
            if let Some(graph) = status.graphs.get_mut(&name) {
                let dump = graph.dump((node, param)).map_err(|x| anyhow!(x))?;
                println!("{}", dump);
            }

            Ok(CommandStatus::Done)
        }
    }
}

fn main() -> Result<(), &'static str> {
    let status = Arc::new(Mutex::new(Status::default()));

    Repl::builder()
        .add("list", list_command(status.clone()))
        .add("load", load_command(status.clone()))
        .add("initialize", initialize_command(status.clone()))
        .add("step", step_command(status.clone()))
        .add("terminate", terminate_command(status.clone()))
        .add("nodes", nodes_command(status.clone()))
        .add("node_inputs", node_inputs_command(status.clone()))
        .add("node_outputs", node_outputs_command(status.clone()))
        .add("dump", dump_command(status.clone()))
        .build()
        .or(Err("Failed to create repl"))
        .and_then(|mut repl| repl.run().or(Err("Critical REPL error")))
}
