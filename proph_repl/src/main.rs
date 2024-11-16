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

fn system_load(
    status: Rc<Mutex<Status>>,
    filename: String,
) -> Result<CommandStatus, anyhow::Error> {
    let mut configuration = String::new();

    File::open(filename)?.read_to_string(&mut configuration)?;
    let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;

    let mut registry = Registry::default();
    for lib in &status.libraries {
        registry
            .load_library(lib)
            .map_err(|x| anyhow!(x.to_string()))?;
    }

    let repr = SystemRepr::from_json(&configuration).map_err(|x| anyhow!(x.to_string()))?;
    let system = repr
        .to_system(&registry)
        .map_err(|x| anyhow!(x.to_string()))?;

    status.system = system;
    println!("loaded:");
    for name in status.system.graphs() {
        println!("-> {name:}");
    }

    Ok(CommandStatus::Done)
}

fn graph_list(status: Rc<Mutex<Status>>) -> Result<CommandStatus, anyhow::Error> {
    let status = status.lock().or(Err(anyhow!("cannot lock")))?;

    for name in status.system.graphs() {
        println!("-> {name:}");
    }
    Ok(CommandStatus::Done)
}

fn graph_start(status: Rc<Mutex<Status>>, graph: String) -> Result<CommandStatus, anyhow::Error> {
    let status = status.lock().or(Err(anyhow!("cannot lock")))?;
    let graph = serde_json::to_string(&graph).map_err(|x| anyhow!(x))?;
    let graph_id: GraphId = serde_json::from_str(&graph).map_err(|x| anyhow!(x))?;
    let result = status.system.command(graph_id, RCommand::Start);
    println!("{result:?}");

    Ok(CommandStatus::Done)
}

fn graph_stop(status: Rc<Mutex<Status>>, graph: String) -> Result<CommandStatus, anyhow::Error> {
    let status = status.lock().or(Err(anyhow!("cannot lock")))?;
    let graph = serde_json::to_string(&graph).map_err(|x| anyhow!(x))?;
    let graph_id: GraphId = serde_json::from_str(&graph).map_err(|x| anyhow!(x))?;
    let result = status.system.command(graph_id, RCommand::Stop);

    println!("{result:?}");

    Ok(CommandStatus::Done)
}

fn graph_status(status: Rc<Mutex<Status>>, graph: String) -> Result<CommandStatus, anyhow::Error> {
    let status = status.lock().or(Err(anyhow!("cannot lock")))?;
    let graph = serde_json::to_string(&graph).map_err(|x| anyhow!(x))?;
    let graph_id: GraphId = serde_json::from_str(&graph).map_err(|x| anyhow!(x))?;
    let result = status.system.command(graph_id, RCommand::Status);

    println!(">> {result:?}");

    Ok(CommandStatus::Done)
}

fn library_list(status: Rc<Mutex<Status>>) -> Result<CommandStatus, anyhow::Error> {
    let status = status.lock().or(Err(anyhow!("cannot lock")))?;
    for lib in &status.libraries {
        println!("{lib}");
    }
    Ok(CommandStatus::Done)
}

fn library_add(status: Rc<Mutex<Status>>, library: String) -> Result<CommandStatus, anyhow::Error> {
    let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;
    let mut registry = Registry::default();
    registry
        .load_library(&library)
        .or(Err(anyhow!("cannot load library")))?;

    for f in registry.list_factories() {
        println!("{f}");
    }

    status.libraries.insert(library);

    Ok(CommandStatus::Done)
}

fn library_remove(
    status: Rc<Mutex<Status>>,
    library: String,
) -> Result<CommandStatus, anyhow::Error> {
    let mut status = status.lock().or(Err(anyhow!("cannot lock")))?;
    status.libraries.remove(&library);
    println!("removed");
    Ok(CommandStatus::Done)
}

fn library_inspect(library: String) -> Result<CommandStatus, anyhow::Error> {
    let mut registry = Registry::default();
    registry
        .load_library(&library)
        .or(Err(anyhow!("cannot load library")))?;

    for f in registry.list_factories() {
        println!("{f}");
    }

    Ok(CommandStatus::Done)
}

fn node_list(status: Rc<Mutex<Status>>, graph: String) -> Result<CommandStatus, anyhow::Error> {
    let graph = serde_json::to_string(&graph).map_err(|x| anyhow!(x))?;
    let graph_id: GraphId = serde_json::from_str(&graph).map_err(|x| anyhow!(x))?;
    let status = status.lock().or(Err(anyhow!("cannot lock")))?;
    let result = status
        .system
        .command(graph_id, RCommand::ListNodes)
        .and_then(|x| x.dump::<Vec<NodeId>>())
        .map_err(|x| anyhow!(x.to_string()))?;

    println!("{result:?}");
    Ok(CommandStatus::Done)
}

fn node_inputs(
    status: Rc<Mutex<Status>>,
    graph: String,
    node: String,
) -> Result<CommandStatus, anyhow::Error> {
    let node = serde_json::to_string(&node).map_err(|x| anyhow!(x))?;
    let node_id: NodeId = serde_json::from_str(&node).map_err(|x| anyhow!(x))?;
    let graph = serde_json::to_string(&graph).map_err(|x| anyhow!(x))?;
    let graph_id: GraphId = serde_json::from_str(&graph).map_err(|x| anyhow!(x))?;
    let status = status.lock().or(Err(anyhow!("cannot lock")))?;
    let result = status
        .system
        .command(graph_id, RCommand::ListInputs(node_id))
        .and_then(|val| val.dump::<Vec<ParamId>>())
        .map_err(|x| anyhow!(x.to_string()))?;

    println!("{result:?}");
    Ok(CommandStatus::Done)
}

fn node_outputs(
    status: Rc<Mutex<Status>>,
    graph: String,
    node: String,
) -> Result<CommandStatus, anyhow::Error> {
    let node = serde_json::to_string(&node).map_err(|x| anyhow!(x))?;
    let node_id: NodeId = serde_json::from_str(&node).map_err(|x| anyhow!(x))?;
    let graph = serde_json::to_string(&graph).map_err(|x| anyhow!(x))?;
    let graph_id: GraphId = serde_json::from_str(&graph).map_err(|x| anyhow!(x))?;
    let status = status.lock().or(Err(anyhow!("cannot lock")))?;
    let result = status
        .system
        .command(graph_id, RCommand::ListOutputs(node_id))
        .and_then(|val| val.dump::<Vec<NodeId>>())
        .map_err(|x| anyhow!(x.to_string()))?;

    println!("{result:?}");
    Ok(CommandStatus::Done)
}

fn node_dump(
    status: Rc<Mutex<Status>>,
    graph: String,
    node: String,
    param: String,
) -> Result<CommandStatus, anyhow::Error> {
    let node = serde_json::to_string(&node).map_err(|x| anyhow!(x))?;
    let node: NodeId = serde_json::from_str(&node).map_err(|x| anyhow!(x))?;
    let param = serde_json::to_string(&param).map_err(|x| anyhow!(x))?;
    let param: ParamId = serde_json::from_str(&param).map_err(|x| anyhow!(x))?;
    let graph = serde_json::to_string(&graph).map_err(|x| anyhow!(x))?;
    let graph_id: GraphId = serde_json::from_str(&graph).map_err(|x| anyhow!(x))?;

    let status = status.lock().or(Err(anyhow!("cannot lock")))?;
    let result = status
        .system
        .command(graph_id, RCommand::MultiDump(vec![(node, param)]));

    println!("{result:?}");

    Ok(CommandStatus::Done)
}

fn node_load(
    status: Rc<Mutex<Status>>,
    graph: String,
    node: String,
    param: String,
    value: String,
) -> Result<CommandStatus, anyhow::Error> {
    let node = serde_json::to_string(&node).map_err(|x| anyhow!(x))?;
    let node: NodeId = serde_json::from_str(&node).map_err(|x| anyhow!(x))?;
    let param = serde_json::to_string(&param).map_err(|x| anyhow!(x))?;
    let param: ParamId = serde_json::from_str(&param).map_err(|x| anyhow!(x))?;
    let graph = serde_json::to_string(&graph).map_err(|x| anyhow!(x))?;
    let graph_id: GraphId = serde_json::from_str(&graph).map_err(|x| anyhow!(x))?;
    let value: Value = serde_json::from_str(&value).map_err(|x| anyhow!(x))?;

    let status = status.lock().or(Err(anyhow!("cannot lock")))?;
    let result = status
        .system
        .command(graph_id, RCommand::MultiLoad(vec![(node, param, value)]));

    println!("{result:?}");

    Ok(CommandStatus::Done)
}

fn system_load_command(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Load a system from a configuration",
        (filename: String) =>   |filename| system_load(status.clone(), filename)
    }
}

fn graph_list_command(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "List graphs",
        () => || graph_list(status.clone())
    }
}

fn graph_start_command(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Start a graph",
        (graph: String) => |graph: String| graph_start(status.clone(), graph)
    }
}

fn graph_stop_command(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Step a graph",
        (graph: String) => |graph: String| graph_stop(status.clone(), graph)
    }
}

fn graph_status_command(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Step a graph",
        (graph: String) => |graph: String| graph_status(status.clone(), graph)
    }
}

fn library_list_command(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "List libraries",
        () => || library_list(status.clone())
    }
}

fn library_add_command(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Add a library",
        (library: String) => |library: String| library_add(status.clone(), library)
    }
}

fn library_remove_command(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Remove a library",
        (library: String) => |library| library_remove(status.clone(), library)
    }
}

fn library_inspect_command() -> Command<'static> {
    command! {
        "Inspect a library",
        (library: String) => |library:String| library_inspect(library)
    }
}

fn node_list_command(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "List graph nodes",
        (graph: String) => |graph: String| node_list(status.clone(), graph)
    }
}

fn node_inputs_command(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "List inputs of a graph node",
        (graph: String, node: String) => |graph:String, node: String| node_inputs(status.clone(), graph, node)
    }
}

fn node_outputs_command(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "List outputs of a graph node",
        (graph: String, node: String) => |graph:String, node:String| node_outputs(status.clone(), graph, node)
    }
}

fn node_dump_command(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Dump a input/output of a graph node",
        (graph: String, node: String, param: String) => |graph: String, node:String, param:String| node_dump(status.clone(), graph, node, param)
    }
}

fn node_load_command(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Load to an input/output of a graph node",
        (graph: String, node: String, param: String, value: String) => |graph: String, node:String, param:String, value: String| node_load(status.clone(), graph, node, param, value)
    }
}

fn main() -> Result<(), &'static str> {
    let matches = clap::Command::new("rep")
        .about("start a proph repl")
        .version("0.0.1")
        .arg_required_else_help(true)
        .author("Treere")
        .arg(clap::Arg::new("library").short('l'))
        .get_matches();

    let status = Rc::new(Mutex::new(Status::default()));

    if let Some(library) = matches.get_one::<String>("library") {
        library_add(status.clone(), library.clone()).or(Err("Cannot load library"))?;
    }

    Repl::builder()
        .with_filename_completion(true)
        .add("system_load", system_load_command(status.clone()))
        .add("graph_list", graph_list_command(status.clone()))
        .add("graph_start", graph_start_command(status.clone()))
        .add("graph_stop", graph_stop_command(status.clone()))
        .add("graph_status", graph_status_command(status.clone()))
        .add("library_list", library_list_command(status.clone()))
        .add("library_add", library_add_command(status.clone()))
        .add("library_remove", library_remove_command(status.clone()))
        .add("library_inspect", library_inspect_command())
        .add("node_list", node_list_command(status.clone()))
        .add("node_inputs", node_inputs_command(status.clone()))
        .add("node_outputs", node_outputs_command(status.clone()))
        .add("node_dump", node_dump_command(status.clone()))
        .add("node_load", node_load_command(status.clone()))
        .add(
            "exit",
            command! { "Exit program", () => || Ok(CommandStatus::Quit) },
        )
        .build()
        .or(Err("Failed to create repl"))
        .and_then(|mut repl| repl.run().or(Err("Critical REPL error")))
}
