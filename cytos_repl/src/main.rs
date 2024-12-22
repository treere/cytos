use anyhow::anyhow;
use easy_repl::{command, Command, CommandStatus, Repl};

use cytos::loader::Registry;
use cytos::repr::SystemRepr;
use cytos::{id_number_to_string, id_string_to_number, NodeId, ParamId, System, Value};

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

fn graph_start(
    status: Rc<Mutex<Status>>,
    graph_id: String,
) -> Result<CommandStatus, anyhow::Error> {
    let status = status.lock().or(Err(anyhow!("cannot lock")))?;
    let result = status.system.graph(graph_id.into()).unwrap().start();
    println!("{result:?}");

    Ok(CommandStatus::Done)
}

fn graph_stop(status: Rc<Mutex<Status>>, graph_id: String) -> Result<CommandStatus, anyhow::Error> {
    let status = status.lock().or(Err(anyhow!("cannot lock")))?;
    let result = status.system.graph(graph_id.into()).unwrap().stop();

    println!("{result:?}");

    Ok(CommandStatus::Done)
}

fn graph_status(
    status: Rc<Mutex<Status>>,
    graph_id: String,
) -> Result<CommandStatus, anyhow::Error> {
    let status = status.lock().or(Err(anyhow!("cannot lock")))?;
    let result = status
        .system
        .graph(graph_id.into())
        .unwrap()
        .status()
        .unwrap();

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

fn node_list(status: Rc<Mutex<Status>>, graph_id: String) -> Result<CommandStatus, anyhow::Error> {
    let status = status.lock().or(Err(anyhow!("cannot lock")))?;
    let result = status
        .system
        .graph(graph_id.into())
        .unwrap()
        .list_nodes()
        .and_then(|x| x.dump::<Vec<NodeId>>())
        .map_err(|x| anyhow!(x.to_string()))?;

    println!("{result:?}");
    Ok(CommandStatus::Done)
}

fn node_inputs(
    status: Rc<Mutex<Status>>,
    graph_id: String,
    node_id: String,
) -> Result<CommandStatus, anyhow::Error> {
    let status = status.lock().or(Err(anyhow!("cannot lock")))?;
    let result = status
        .system
        .graph(graph_id.into())
        .unwrap()
        .list_inputs(node_id.into())
        .and_then(|val| val.dump::<Vec<ParamId>>())
        .map_err(|x| anyhow!(x.to_string()))?;

    println!("{result:?}");
    Ok(CommandStatus::Done)
}

fn node_outputs(
    status: Rc<Mutex<Status>>,
    graph_id: String,
    node_id: String,
) -> Result<CommandStatus, anyhow::Error> {
    let status = status.lock().or(Err(anyhow!("cannot lock")))?;
    let result = status
        .system
        .graph(graph_id.into())
        .unwrap()
        .list_outputs(node_id.into())
        .and_then(|val| val.dump::<Vec<NodeId>>())
        .map_err(|x| anyhow!(x.to_string()))?;

    println!("{result:?}");
    Ok(CommandStatus::Done)
}

fn node_remove(
    status: Rc<Mutex<Status>>,
    graph_id: String,
    node_id: String,
) -> Result<CommandStatus, anyhow::Error> {
    let status = status.lock().or(Err(anyhow!("cannot lock")))?;
    let result = status
        .system
        .graph(graph_id.into())
        .unwrap()
        .remove_node(node_id.into());

    println!("{result:?}");

    Ok(CommandStatus::Done)
}

fn node_dump(
    status: Rc<Mutex<Status>>,
    graph_id: String,
    node_id: String,
    param_id: String,
) -> Result<CommandStatus, anyhow::Error> {
    let status = status.lock().or(Err(anyhow!("cannot lock")))?;
    let result = status
        .system
        .graph(graph_id.into())
        .unwrap()
        .dump(vec![(node_id.into(), param_id.into())]);

    println!("{result:?}");

    Ok(CommandStatus::Done)
}

fn node_load(
    status: Rc<Mutex<Status>>,
    graph_id: String,
    node_id: String,
    param_id: String,
    value: String,
) -> Result<CommandStatus, anyhow::Error> {
    let value: Value = serde_json::from_str(&value).map_err(|x| anyhow!(x))?;

    let status = status.lock().or(Err(anyhow!("cannot lock")))?;
    let result = status.system.graph(graph_id.into()).unwrap().load(vec![(
        node_id.into(),
        param_id.into(),
        value,
    )]);

    println!("{result:?}");

    Ok(CommandStatus::Done)
}

fn node_assign(
    status: Rc<Mutex<Status>>,
    graph_id: String,
    node_id: String,
    param_id: String,
    value: String,
) -> Result<CommandStatus, anyhow::Error> {
    let value: Value = serde_json::from_str(&value).map_err(|x| anyhow!(x))?;

    let status = status.lock().or(Err(anyhow!("cannot lock")))?;
    let result = status.system.graph(graph_id.into()).unwrap().assign(vec![(
        node_id.into(),
        param_id.into(),
        value,
    )]);

    println!("{result:?}");

    Ok(CommandStatus::Done)
}

fn link_list(status: Rc<Mutex<Status>>, graph_id: String) -> Result<CommandStatus, anyhow::Error> {
    let status = status.lock().or(Err(anyhow!("cannot lock")))?;
    let l = status.system.graph(graph_id.into()).unwrap().list_links();

    println!("{:?}", l);

    Ok(CommandStatus::Done)
}

fn link_nodes(
    status: Rc<Mutex<Status>>,
    graph_id: String,
    (src_node, src_param): (String, String),
    (dst_node, dst_param): (String, String),
) -> Result<CommandStatus, anyhow::Error> {
    let status = status.lock().or(Err(anyhow!("cannot lock")))?;
    status
        .system
        .graph(graph_id.into())
        .unwrap()
        .add_link(
            (src_node.into(), src_param.into()),
            (dst_node.into(), dst_param.into()),
        )
        .unwrap();

    Ok(CommandStatus::Done)
}

fn receiver_list(
    status: Rc<Mutex<Status>>,
    graph_id: String,
) -> Result<CommandStatus, anyhow::Error> {
    let status = status.lock().or(Err(anyhow!("cannot lock")))?;
    let l = status
        .system
        .graph(graph_id.into())
        .unwrap()
        .list_receivers();

    println!("{:?}", l);

    Ok(CommandStatus::Done)
}

fn receiver_add(
    status: Rc<Mutex<Status>>,
    (src_graph, src_node, src_param): (String, String, String),
    (dst_graph, dst_node, dst_param): (String, String, String),
) -> Result<CommandStatus, anyhow::Error> {
    let status = status.lock().or(Err(anyhow!("cannot lock")))?;
    status
        .system
        .graph(src_graph.into())
        .unwrap()
        .add_receiver(
            (dst_graph.into(), dst_node.into(), dst_param.into()),
            (src_node.into(), src_param.into()),
        )
        .unwrap();

    Ok(CommandStatus::Done)
}

fn receiver_remove(
    status: Rc<Mutex<Status>>,
    (src_graph, src_node, src_param): (String, String, String),
    (dst_graph, dst_node, dst_param): (String, String, String),
) -> Result<CommandStatus, anyhow::Error> {
    let status = status.lock().or(Err(anyhow!("cannot lock")))?;
    status
        .system
        .graph(src_graph.into())
        .unwrap()
        .remove_receiver(
            (dst_graph.into(), dst_node.into(), dst_param.into()),
            (src_node.into(), src_param.into()),
        )
        .unwrap();

    Ok(CommandStatus::Done)
}

fn system_load_command(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Load a system from a configuration",
        (filename: String) => |filename| system_load(status.clone(), filename)
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

fn node_remove_command(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Remove a node from a graph",
        (graph: String, node: String) => |graph: String, node:String| node_remove(status.clone(), graph, node)
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

fn node_assign_command(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Assign to an input/output of a graph node",
        (graph: String, node: String, param: String, value: String) => |graph: String, node:String, param:String, value: String| node_assign(status.clone(), graph, node, param, value)
    }
}

fn link_nodes_command(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Link two nodes of the same graph",
        (graph: String, src_node: String, src_param: String, dst_node: String, dst_param: String) => |graph: String, src_node:String, src_param:String, dst_node: String, dst_param: String| link_nodes(status.clone(), graph, (src_node, src_param), (dst_node, dst_param))
    }
}

fn link_list_command(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Link two nodes of the same graph",
        (graph: String) => |graph: String| link_list(status.clone(), graph)
    }
}

fn receiver_list_command(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "List receivers",
        (graph: String) => |graph: String| receiver_list(status.clone(), graph)
    }
}

fn receiver_add_command(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Add a receiver to a graph",
        (src_graph: String, src_node: String, src_param: String, dst_graph: String, dst_node: String, dst_param: String) => |src_graph: String, src_node:String, src_param:String, dst_graph: String, dst_node: String, dst_param: String| receiver_add(status.clone(), (src_graph, src_node, src_param), (dst_graph, dst_node, dst_param))
    }
}

fn receiver_remove_command(status: Rc<Mutex<Status>>) -> Command<'static> {
    command! {
        "Remove a receiver to a graph",
        (src_graph: String, src_node: String, src_param: String, dst_graph: String, dst_node: String, dst_param: String) => |src_graph: String, src_node:String, src_param:String, dst_graph: String, dst_node: String, dst_param: String| receiver_remove(status.clone(), (src_graph, src_node, src_param), (dst_graph, dst_node, dst_param))
    }
}

fn main() -> Result<(), &'static str> {
    env_logger::init();

    let matches = clap::Command::new("rep")
        .about("start a cytos repl")
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
        .add("node_remove", node_remove_command(status.clone()))
        .add("node_dump", node_dump_command(status.clone()))
        .add("node_load", node_load_command(status.clone()))
        .add("node_assign", node_assign_command(status.clone()))
        .add("link_nodes", link_nodes_command(status.clone()))
        .add("link_list", link_list_command(status.clone()))
        .add("receiver_list", receiver_list_command(status.clone()))
        .add("receiver_add", receiver_add_command(status.clone()))
        .add("receiver_remove", receiver_remove_command(status.clone()))
        .add(
            "id_s2n",
            command! {
            "Convert id from string to number",
            (s: String) => |s: String| {
                println!("-> {}",id_string_to_number(&s).map_err(|x| anyhow!(x.to_string()))?);
                Ok(CommandStatus::Done)
            }},
        )
        .add(
            "id_n2s",
            command! {
            "Convert id from number to string",
            (s: u64) => |s: u64| {
                println!("-> {}",id_number_to_string(s).map_err(|x| anyhow!(x.to_string()))?);
                Ok(CommandStatus::Done)
            }},
        )
        .add(
            "exit",
            command! { "Exit program", () => || Ok(CommandStatus::Quit) },
        )
        .build()
        .or(Err("Failed to create repl"))
        .and_then(|mut repl| repl.run().or(Err("Critical REPL error")))
}
