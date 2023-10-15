use clap::Command;
use easy_repl::{command, CommandStatus, Repl};
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

struct Status {
    graphs: HashMap<String, Graph>,
}

fn main() -> Result<(), String> {
    let status = Arc::new(Mutex::new(Status {
        graphs: HashMap::new(),
    }));

    let s = status.clone();
    let load_command = command! {
    "Load a configuration",
    (name: String, filename: String) =>   |name, filename| {
        let loader = load_registry();
        let mut status = s.lock().expect("cannot lock");
        let mut configuration = String::new();

        File::open(filename)?.read_to_string(&mut configuration)?;

        let mut graph = GraphRepr::load(&configuration, &loader).expect("a");

        status.graphs.insert(name, graph);

        Ok(CommandStatus::Done)
    }};

    let s = status.clone();
    let initialize_command = command! {
        "Run a loaded graph",
        (name: String) => |name| {
            let mut status = s.lock().expect("cannot lock");
            if let Some(graph) = status.graphs.get_mut(&name) {
                let _ = graph.initialize();
            }

            Ok(CommandStatus::Done)
        }
    };

    let s = status.clone();
    let step_command = command! {
        "Step a loaded graph",
        (name: String) => |name| {
            let mut status = s.lock().expect("cannot lock");
            if let Some(graph) = status.graphs.get_mut(&name) {
                let _ = graph.step();
            }

            Ok(CommandStatus::Done)
        }
    };

    let s = status.clone();
    let nodes_command = command! {
        "List graph nodes",
        (name: String) => |name| {
            let mut status = s.lock().expect("cannot lock");
            if let Some(graph) = status.graphs.get_mut(&name) {
                for n in  graph.list_nodes() {
                    println!("{}", n);
                }
            }

            Ok(CommandStatus::Done)
        }
    };

    let s = status.clone();
    let node_inputs_command = command! {
        "List input of a graph nodes",
        (name: String, node: String) => |name, node| {
            let mut status = s.lock().expect("cannot lock");
            if let Some(graph) = status.graphs.get_mut(&name) {
                for n in  graph.list_node_inputs(node) {
                    println!("{}", n);
                }
            }

            Ok(CommandStatus::Done)
        }
    };

    let s = status.clone();
    let node_outputs_command = command! {
        "List output of a graph nodes",
        (name: String, node: String) => |name, node| {
            let mut status = s.lock().expect("cannot lock");
            if let Some(graph) = status.graphs.get_mut(&name) {
                for n in  graph.list_node_outputs(node) {
                    println!("{}", n);
                }
            }

            Ok(CommandStatus::Done)
        }
    };

    let _matches = Command::new("repl")
        .about("proph repl")
        .version("0.0.1")
        .author("Treere")
        .get_matches();

    Repl::builder()
        .add("load_config", load_command)
        .add("initialize_graph", initialize_command)
        .add("step_graph", step_command)
        .add("nodes_graph", nodes_command)
        .add("node_inputs_graph", node_inputs_command)
        .add("node_outputs_graph", node_outputs_command)
        .build()
        .expect("Failed to create repl")
        .run()
        .expect("Critical REPL error");

    Ok(())
}
