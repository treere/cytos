use anyhow::anyhow;

use easy_repl::{command, Command, CommandStatus, Repl};

use proph::architecture::system::{Command as RCommand, SystemRepr};
use proph::architecture::{GraphId, NodeId, ParamId, System, Value};
use proph::loader::Registry;

use std::collections::HashSet;

use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::sync::Mutex;

#[derive(Default)]
struct Status {
    system: System,
    libraries: HashSet<String>,
}

fn system_load(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
    "Load a system from a configuration",
    (filename: String) =>   |filename| {
        let mut configuration = String::new();

        File::open(filename)?.read_to_string(&mut configuration)?;
        let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;

        let mut registry = Registry::default();
        for lib in &status.libraries {
            registry.load_library(lib).map_err(|x| anyhow!(x))?;
        }

        let repr = SystemRepr::from_json(&configuration).map_err(|x| anyhow!(x))?;
        let system = System::from_repr(repr, &registry).map_err(|x| anyhow!(x))?;

        status.system = system;
        println!("loaded!");
        Ok(CommandStatus::Done)
    }}
}

fn graph_list(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "List graphs",
        () => ||{
            let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;

            for name in status.system.keys() {
                println!("-> {name:}");
            }
            Ok(CommandStatus::Done)
        }
    }
}

fn graph_start(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Start a graph",
        (graph: String) => |graph: String| {
            let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;
            let graph_id = GraphId::try_from(&graph).map_err(|x| anyhow!(x))?;
            let result = status.system.command(graph_id, RCommand::Start);
            println!("{result:?}");

            Ok(CommandStatus::Done)
    }}
}
fn graph_stop(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Step a graph",
        (graph: String) => |graph: String| {
            let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;
            let graph_id = GraphId::try_from(&graph).map_err(|x| anyhow!(x))?;
            let result = status.system.command(graph_id, RCommand::Stop);

            println!("{result:?}");

            Ok(CommandStatus::Done)
        }
    }
}

fn graph_status(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Step a graph",
        (graph: String) => |graph: String| {
            let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;
            let graph_id = GraphId::try_from(&graph).map_err(|x| anyhow!(x))?;
            let result = status.system.command(graph_id, RCommand::Status);

            println!("{result:?}");

            Ok(CommandStatus::Done)
        }
    }
}

fn library_list(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "List libraries",
        () => || {
            let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;
            for lib in & status.libraries {
                println!("{lib}");
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
                println!("{f}");
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
                println!("{f}");
            }

            Ok(CommandStatus::Done)
        }
    }
}

fn node_list(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "List graph nodes",
        (graph: String) => |graph: String| {
            let graph_id = GraphId::try_from(&graph).map_err(|x| anyhow!(x))?;
            let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;
            let result = status.system.command(graph_id, RCommand::ListNodes).and_then(|x| x.0.convert::<Vec<NodeId>>()).map_err(|x|anyhow!(x))?;

            println!("{result:?}");
            Ok(CommandStatus::Done)
        }
    }
}

fn node_inputs(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "List inputs of a graph node",
        (graph: String, node: String) => |graph:String, node: String| {
            let node = NodeId::try_from(&node).map_err(|x| anyhow!(x))?;
            let graph_id = GraphId::try_from(&graph).map_err(|x| anyhow!(x))?;
            let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;
            let result = status.system.command(graph_id, RCommand::ListInputs(node))
            .and_then(|val| val.0.convert::<Vec<ParamId>>()).map_err(|x| anyhow!(x))?;

            println!("{result:?}");
            Ok(CommandStatus::Done)
        }
    }
}

fn node_outputs(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "List outputs of a graph node",
        (graph: String, node: String) => |graph:String, node:String| {
            let node = NodeId::try_from(&node).map_err(|x| anyhow!(x))?;
            let graph_id = GraphId::try_from(&graph).map_err(|x| anyhow!(x))?;
            let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;
            let result = status.system.command(graph_id, RCommand::ListOutputs(node))
                .and_then(|val| val.0.convert::<Vec<NodeId>>())
                .map_err(|x|anyhow!(x))?;

            println!("{result:?}");
            Ok(CommandStatus::Done)
        }
    }
}

fn node_dump(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Dump a input/output of a graph node",
        (graph: String, node: String, param: String) => |graph: String, node:String, param:String| {
            let node = NodeId::try_from(&node).map_err(|x| anyhow!(x))?;
            let param = ParamId::try_from(&param).map_err(|x| anyhow!(x))?;
            let graph_id = GraphId::try_from(&graph).map_err(|x| anyhow!(x))?;
            let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;
            let result = status.system.command(graph_id, RCommand::Dump(node,param));

            println!("{result:?}");

            Ok(CommandStatus::Done)
        }
    }
}

fn node_load(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Load to an input/output of a graph node",
        (graph: String, node: String, param: String, value: String) => |graph: String, node:String, param:String, value: String| {
            let node = NodeId::try_from(&node).map_err(|x| anyhow!(x))?;
            let param = ParamId::try_from(&param).map_err(|x| anyhow!(x))?;
            let value = Value::from_string(&value).map_err(|x| anyhow!(x))?;
            let graph_id = GraphId::try_from(&graph).map_err(|x| anyhow!(x))?;

            let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;
            let result = status.system.command(graph_id, RCommand::Load(node,param, value));

            println!("{result:?}");

            Ok(CommandStatus::Done)
        }
    }
}

fn main() -> Result<(), &'static str> {
    let status = Rc::new(Mutex::new(Status::default()));

    Repl::builder()
        .with_filename_completion(true)
        .add("system_load", system_load(status.clone()))
        .add("graph_list", graph_list(status.clone()))
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
        .add(
            "exit",
            command! { "Exit program", () => || Ok(CommandStatus::Quit) },
        )
        .build()
        .or(Err("Failed to create repl"))
        .and_then(|mut repl| repl.run().or(Err("Critical REPL error")))
}
