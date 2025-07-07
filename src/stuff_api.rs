use std::sync::Arc;

use axum::debug_handler;
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use blend_result;
use hyper::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::debug;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

/// In-memory todo store
type Store = Mutex<Storage>;

#[derive(Clone)]
struct Storage {
    blend_storage: Vec<(String, String)>,
}

impl Storage {
    fn new() -> Self {
        Storage {
            blend_storage: Vec::default(),
        }
    }
}

/// Item to do.
#[derive(Serialize, Deserialize, ToSchema, Clone)]
struct Todo {
    id: i32,
    #[schema(example = "Buy groceries")]
    value: String,
    done: bool,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
struct Val {
    #[schema(example = "Buy groceries")]
    value: String,
}

/// Todo operation errors
#[derive(Serialize, Deserialize, ToSchema)]
enum TodoError {
    /// Todo already exists conflict.
    #[schema(example = "Todo already exists")]
    Conflict(String),
    /// Todo not found by id.
    #[schema(example = "id = 1")]
    NotFound(String),
    /// Todo operation unauthorized
    #[schema(example = "missing api key")]
    Unauthorized(String),
}

pub(super) fn router() -> OpenApiRouter {
    let store = Arc::new(Mutex::new(Storage::new()));
    OpenApiRouter::new()
        .routes(routes!(do_stuff))
        .routes(routes!(convert_xml))
        .routes(routes!(upload_to_blend))
        .routes(routes!(testquery))
        .routes(routes!(blend_files))
        .routes(routes!(list_to_blend))
        .with_state(store)
}

/// Stuff
#[utoipa::path(
        get,
        path = "/stuff/{mul}",
        tag = "stuff",
        responses(
            (status = 200, description = "Stuff successfully", body = String),
            (status = 401, description = "Unauthorized", body = TodoError),
            // (status = 401, description = "Unauthorized", body = TodoError, example = json!(TodoError::Unauthorized(String::from("missing api key")))),
            (status = 404, description = "Stuff not found")
        ),
        params(
            ("mul" = u32, Path, description = "Multron")
        ),
        security(
            ("api_key" = [])
        )
    )]
async fn do_stuff(
    Path(mul): Path<u32>,
    State(_store): State<Arc<Store>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    match check_api_key(true, headers) {
        Ok(_) => (),
        Err(error) => return error.into_response(),
    }
    Json(String::from("Stuff").repeat(mul as usize)).into_response()
}

/// Upload file to blend
/// does not work yet
#[utoipa::path(
        get,
        path = "/stuff/testquery",
        tag = "stuff",
        params(
            ("name" = String, Query),
            // ("data" = String, Query),
        ),
        responses(
            (status = 200, description = "File uploadedFile uploaded"),
            (status = 400, description = "Whatever", body = String),
        ),
    )]
async fn testquery(
    // Query(name): Query<String>,
    name: Query<String>,
    // Query((name, data)): Query<(String, String)>,
    // Query(data): Query<String>,
    //State(store): State<Arc<Store>>,
    // name: String,
    // data: String,
) {
    //let mut state = store.lock().await;
    println!("The Request {:?}", name);
    // println!("The Request Data {:?}", data.len());
    // state.blend_storage.push((name, data));
}

/// convert
#[utoipa::path(
        post,
        path = "/xml",
        tag = "blend",
        responses(
            (status = 200, description = "List matching todos by query", body = String),
            (status = 201, description = "Todo item created successfully", body = String),
            (status = 409, description = "Todo already exists", body = String),
        ),
        request_body(content = String, description = "Xml as string request", content_type = "text/xml"),
    )]
async fn convert_xml(
    State(_store): State<Arc<Store>>,
    //string : Query<String>
    //Json(val): Json<Val>,
    //body: String,
    string: String, //json : Json<String>
) -> String {
    //) -> Json<String> {
    //let string : String = Json(json);
    //let string : String = val.value;
    // println!("The Request {}", string);
    let results = blend_result::parse_from_str_to_str(&string).unwrap();
    // println!("The Reponse {}", results);
    results
    //Json(results)
}

/// Upload file to blend
#[utoipa::path(
        post,
        path = "/upload/{name}",
        tag = "blend",
        responses(
            (status = 200, description = "File uploaded"),
        ),
        params(
            ("name" = String, Path, description = "Filename")
        ),
        request_body(content = String, description = "Xml as string request",
             content_type = "text/xml"),
    )]
#[debug_handler]
async fn upload_to_blend(
    Path(name): Path<String>,
    State(store): State<Arc<Store>>,
    data: String,
) -> impl IntoResponse {
    let mut state = store.lock().await;
    // println!("The Request {:?}", data);
    // println!("The Request Data {:?}", data.len());
    state.blend_storage.push((name, data));
}

/// List files
#[utoipa::path(
        get,
        path = "/list",
        tag = "blend",
        responses(
            (status = 200, description = "Files", body = String),
        ),
    )]
async fn list_to_blend(State(store): State<Arc<Store>>) -> impl IntoResponse {
    let state = store.lock().await;
    let files = state
        .blend_storage
        .iter()
        .map(|x| x.0.clone())
        .collect::<Vec<String>>();
    format!("{:?}", files)
}

/// blend
#[utoipa::path(
        get,
        path = "/blend",
        tag = "blend",
        responses(
            (status = 200, description = "List matching todos by query",
                 content_type = "application/octet-stream"),
            (status = 400, description = "Errror" ),
        ),
    )]
async fn blend_files(State(store): State<Arc<Store>>) -> impl IntoResponse {
    let mut state = store.lock().await;
    // println!("The Request {}", string);
    let files = state
        .blend_storage
        .iter()
        .map(|x| x.0.clone())
        .collect::<Vec<String>>();
    let data = state
        .blend_storage
        .iter()
        .map(|x| x.1.clone())
        .collect::<Vec<String>>();
    let mrl = match blend_result::blend_results::blend(&data, &files, 5) {
        Ok(x) => x,
        Err(error) => {
            debug!("Error while blending");
            return error.to_string().as_bytes().to_owned();
        }
    };
    let result = match mrl.export_to_ods() {
        Ok(x) => x,
        Err(error) => {
            debug!("Error while exporing");
            return error.to_string().as_bytes().to_owned();
        }
    };
    // let result = match String::from_utf8(result) {
    //     Ok(x) => x,
    //     Err(error) => {
    //         debug!("Error while converting");
    //         return error.to_string();
    //     }
    // };
    debug!("The Reponse has len {}", result.len());
    state.blend_storage = Vec::new();

    result
    //Json(results)
}

// normally you should create a middleware for this but this is sufficient for sake of example.
fn check_api_key(
    require_api_key: bool,
    headers: HeaderMap,
) -> Result<(), (StatusCode, Json<TodoError>)> {
    match headers.get("theapikey") {
        Some(header) if header != "rocks" => Err((
            StatusCode::UNAUTHORIZED,
            Json(TodoError::Unauthorized(String::from("incorrect api key"))),
        )),
        None if require_api_key => Err((
            StatusCode::UNAUTHORIZED,
            Json(TodoError::Unauthorized(String::from("missing api key"))),
        )),
        _ => Ok(()),
    }
}
