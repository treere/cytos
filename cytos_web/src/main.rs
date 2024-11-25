use std::io::Read;

use std::{fs::File, sync::Arc};

use axum::extract::{Path, State};
use axum::Json;
use axum::{
    routing::{get, post},
    Router,
};
use cytos::architecture::system::SystemRepr;
use cytos::{architecture::System, loader::Registry};
use serde_json::{json, Value};

type WebSystem = Arc<System>;

#[tokio::main]
async fn main() {
    let matches = clap::Command::new("web")
        .about("start a cytos web")
        .version("0.0.1")
        .arg_required_else_help(true)
        .author("Treere")
        .arg(clap::Arg::new("library").short('l').required(true))
        .arg(clap::Arg::new("config").short('c').required(true))
        .get_matches();

    let library = matches.get_one::<String>("library").unwrap();

    let mut registry = Registry::default();
    registry.load_library(library).unwrap();

    let filename = matches.get_one::<String>("config").unwrap();
    let mut configuration = String::new();

    File::open(filename)
        .unwrap()
        .read_to_string(&mut configuration)
        .unwrap();
    let repr = SystemRepr::from_json(&configuration).unwrap();

    let system = repr.to_system(&registry).unwrap();

    let shared_state = Arc::new(system);

    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/graphs", get(graphs_list))
        .route("/graphs/:id", get(graph_status))
        .route("/graphs/:graph_id/start", post(graph_start))
        .route("/graphs/:graph_id/stop", post(graph_stop))
        .route("/graphs/:graph_id/nodes", get(node_list))
        .route("/graphs/:graph_id/nodes/:node_id/inputs", get(node_inputs))
        .route(
            "/graphs/:graph_id/nodes/:node_id/outputs",
            get(node_outputs),
        )
        .route(
            "/graphs/:graph_id/nodes/:node_id/params/:param_id/load",
            post(node_param_load),
        )
        .route(
            "/graphs/:graph_id/nodes/:node_id/params/:param_id/assign",
            post(node_param_assign),
        )
        .route(
            "/graphs/:graph_id/nodes/:node_id/params/:param_id/dump",
            get(node_param_dump),
        )
        .with_state(shared_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello World!"
}
async fn graphs_list(State(system): State<WebSystem>) -> Json<Value> {
    let graphs: Vec<_> = system.graphs().cloned().collect();
    Json(json!(graphs))
}
async fn graph_status(
    Path(graph_id): Path<String>,
    State(system): State<WebSystem>,
) -> Json<Value> {
    let result = system.graph(graph_id.into()).unwrap().status().unwrap();
    Json(json!(result))
}
async fn graph_start(Path(graph_id): Path<String>, State(system): State<WebSystem>) -> Json<Value> {
    let result = system.graph(graph_id.into()).unwrap().start().unwrap();
    Json(json!(result))
}
async fn graph_stop(Path(graph_id): Path<String>, State(system): State<WebSystem>) -> Json<Value> {
    let result = system.graph(graph_id.into()).unwrap().stop().unwrap();
    Json(json!(result))
}
async fn node_list(Path(graph_id): Path<String>, State(system): State<WebSystem>) -> Json<Value> {
    let result = system.graph(graph_id.into()).unwrap().list_nodes().unwrap();
    Json(json!(result))
}
async fn node_inputs(
    Path((graph_id, node_id)): Path<(String, String)>,
    State(system): State<WebSystem>,
) -> Json<Value> {
    let result = system
        .graph(graph_id.into())
        .unwrap()
        .list_inputs(node_id.into())
        .unwrap();
    Json(json!(result))
}
async fn node_outputs(
    Path((graph_id, node_id)): Path<(String, String)>,
    State(system): State<WebSystem>,
) -> Json<Value> {
    let result = system
        .graph(graph_id.into())
        .unwrap()
        .list_outputs(node_id.into())
        .unwrap();
    Json(json!(result))
}
async fn node_param_dump(
    Path((graph_id, node_id, param_id)): Path<(String, String, String)>,
    State(system): State<WebSystem>,
) -> Json<Value> {
    let result = system
        .graph(graph_id.into())
        .unwrap()
        .dump(vec![(node_id.into(), param_id.into())])
        .unwrap();
    Json(json!(result))
}
async fn node_param_load(
    Path((graph_id, node_id, param_id)): Path<(String, String, String)>,
    State(system): State<WebSystem>,
    Json(value): Json<cytos::architecture::Value>,
) -> Json<Value> {
    let result = system
        .graph(graph_id.into())
        .unwrap()
        .load(vec![(node_id.into(), param_id.into(), value)])
        .unwrap();
    Json(json!(result))
}

async fn node_param_assign(
    Path((graph_id, node_id, param_id)): Path<(String, String, String)>,
    State(system): State<WebSystem>,
    Json(value): Json<cytos::architecture::Value>,
) -> Json<Value> {
    let result = system
        .graph(graph_id.into())
        .unwrap()
        .assign(vec![(node_id.into(), param_id.into(), value)])
        .unwrap();
    Json(json!(result))
}
