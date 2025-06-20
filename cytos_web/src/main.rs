use std::error::Error;
use std::io::Read;

use std::{fs::File, sync::Arc};

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::delete;
use axum::Json;
use axum::{
    routing::{get, post},
    Router,
};
use cytos::repr::SystemRepr;
use cytos::{loader::Registry, System};
use serde_json::{json, Value};
use tower_http::trace::TraceLayer;

type WebSystem = Arc<System>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

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
        .route("/graphs/{id}", get(graph_status))
        .route("/graphs/{graph_id}/start", post(graph_start))
        .route("/graphs/{graph_id}/stop", post(graph_stop))
        .route("/graphs/{graph_id}/nodes", get(node_list))
        .route("/graphs/{graph_id}/links", get(link_list))
        .route("/graphs/{graph_id}/receivers", get(receivers_list))
        .route("/graphs/{graph_id}/receivers", post(receivers_create))
        .route("/graphs/{graph_id}/receivers", delete(receivers_delete))
        .route(
            "/graphs/{graph_id}/nodes/{node_id}/inputs",
            get(node_inputs),
        )
        .route(
            "/graphs/{graph_id}/nodes/{node_id}/outputs",
            get(node_outputs),
        )
        .route("/graphs/{graph_id}/nodes/link", post(node_link))
        .route(
            "/graphs/{graph_id}/nodes/{node_id}/params/{param_id}/load",
            post(node_param_load),
        )
        .route(
            "/graphs/{graph_id}/nodes/{node_id}/params/{param_id}/assign",
            post(node_param_assign),
        )
        .route(
            "/graphs/{graph_id}/nodes/{node_id}/params/{param_id}/dump",
            get(node_param_dump),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(shared_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello World!"
}

struct WebError(String);

impl From<Box<dyn Error>> for WebError {
    fn from(value: Box<dyn Error>) -> Self {
        Self(value.to_string())
    }
}

impl IntoResponse for WebError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0).into_response()
    }
}
async fn graphs_list(State(system): State<WebSystem>) -> Json<Value> {
    let graphs: Vec<_> = system.graphs().cloned().collect();
    Json(json!(graphs))
}
async fn graph_status(
    Path(graph_id): Path<String>,
    State(system): State<WebSystem>,
) -> Result<Json<Value>, WebError> {
    let result = system.graph(graph_id.into())?.status()?;
    Ok(Json(json!(result)))
}
async fn graph_start(
    Path(graph_id): Path<String>,
    State(system): State<WebSystem>,
) -> Result<Json<Value>, WebError> {
    let result = system.graph(graph_id.into())?.start()?;
    Ok(Json(json!(result)))
}
async fn graph_stop(
    Path(graph_id): Path<String>,
    State(system): State<WebSystem>,
) -> Result<Json<Value>, WebError> {
    let result = system.graph(graph_id.into())?.stop()?;
    Ok(Json(json!(result)))
}
async fn node_list(
    Path(graph_id): Path<String>,
    State(system): State<WebSystem>,
) -> Result<Json<Value>, WebError> {
    let result = system.graph(graph_id.into())?.list_nodes()?;
    Ok(Json(json!(result)))
}
async fn link_list(
    Path(graph_id): Path<String>,
    State(system): State<WebSystem>,
) -> Result<Json<Value>, WebError> {
    let result = system.graph(graph_id.into())?.list_links()?;
    Ok(Json(json!(result)))
}
async fn receivers_list(
    Path(graph_id): Path<String>,
    State(system): State<WebSystem>,
) -> Result<Json<Value>, WebError> {
    let result = system.graph(graph_id.into())?.list_receivers()?;
    Ok(Json(json!(result)))
}

type ReceiverJson = Json<((String, String, String), (String, String))>;
async fn receivers_create(
    Path(graph_id): Path<String>,
    State(system): State<WebSystem>,
    Json((src, dst)): ReceiverJson,
) -> Result<Json<Value>, WebError> {
    let result = system.graph(graph_id.into())?.add_receiver(
        (src.0.into(), src.1.into(), src.2.into()),
        (dst.0.into(), dst.1.into()),
    )?;
    Ok(Json(json!(result)))
}
async fn receivers_delete(
    Path(graph_id): Path<String>,
    State(system): State<WebSystem>,
    Json((src, dst)): ReceiverJson,
) -> Result<Json<Value>, WebError> {
    let result = system.graph(graph_id.into())?.remove_receiver(
        (src.0.into(), src.1.into(), src.2.into()),
        (dst.0.into(), dst.1.into()),
    )?;
    Ok(Json(json!(result)))
}
async fn node_inputs(
    Path((graph_id, node_id)): Path<(String, String)>,
    State(system): State<WebSystem>,
) -> Result<Json<Value>, WebError> {
    let result = system.graph(graph_id.into())?.list_inputs(node_id.into())?;
    Ok(Json(json!(result)))
}
async fn node_outputs(
    Path((graph_id, node_id)): Path<(String, String)>,
    State(system): State<WebSystem>,
) -> Result<Json<Value>, WebError> {
    let result = system
        .graph(graph_id.into())?
        .list_outputs(node_id.into())?;
    Ok(Json(json!(result)))
}

async fn node_link(
    Path(graph_id): Path<String>,
    State(system): State<WebSystem>,
    Json(((src_node, src_param), (dst_node, dst_param))): Json<(
        (String, String),
        (String, String),
    )>,
) -> Result<Json<Value>, WebError> {
    let result = system.graph(graph_id.into())?.add_link(
        (src_node.into(), src_param.into()),
        (dst_node.into(), dst_param.into()),
    )?;

    Ok(Json(json!(result)))
}
async fn node_param_dump(
    Path((graph_id, node_id, param_id)): Path<(String, String, String)>,
    State(system): State<WebSystem>,
) -> Result<Json<Value>, WebError> {
    let result = system
        .graph(graph_id.into())?
        .dump(vec![(node_id.into(), param_id.into())])?;
    Ok(Json(json!(result)))
}
async fn node_param_load(
    Path((graph_id, node_id, param_id)): Path<(String, String, String)>,
    State(system): State<WebSystem>,
    Json(value): Json<cytos::Value>,
) -> Result<Json<Value>, WebError> {
    let result =
        system
            .graph(graph_id.into())?
            .load(vec![(node_id.into(), param_id.into(), value)])?;
    Ok(Json(json!(result)))
}

async fn node_param_assign(
    Path((graph_id, node_id, param_id)): Path<(String, String, String)>,
    State(system): State<WebSystem>,
    Json(value): Json<cytos::Value>,
) -> Result<Json<Value>, WebError> {
    let result =
        system
            .graph(graph_id.into())?
            .assign(vec![(node_id.into(), param_id.into(), value)])?;
    Ok(Json(json!(result)))
}
