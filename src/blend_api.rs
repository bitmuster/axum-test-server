use axum;
use axum::debug_handler;
use axum::{
    extract::{Path, State},
    response,
    response::IntoResponse,
    Json,
};
use hyper::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

/// In-memory stuff store
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

/// Todo operation errors
#[derive(Serialize, Deserialize, ToSchema)]
enum StuffError {
    /// Already exists conflict.
    #[schema(example = "Item already exists")]
    Conflict(String),
    /// Not found by id.
    #[schema(example = "id = 1")]
    NotFound(String),
    /// Operation unauthorized
    #[schema(example = "missing api key")]
    Unauthorized(String),
}

pub(super) fn router() -> OpenApiRouter {
    let store = Arc::new(Mutex::new(Storage::new()));
    OpenApiRouter::new()
        .routes(routes!(convert_xml))
        .routes(routes!(upload_to_blend))
        .routes(routes!(blend_files))
        .routes(routes!(list_to_blend))
        .with_state(store)
}

/// convert
#[axum::debug_handler]
#[utoipa::path(
        post,
        path = "/xml",
        tag = "blend",
        responses(
            (status = 200, description = "Call blend_result_parse_from_str_to_str", body = String),
            // (status = 201, description = "Todo item created successfully", body = String),
            // (status = 409, description = "Todo already exists", body = String),
        ),
        request_body(content = String, description = "Xml as string request", content_type = "text/xml"),
        security(
            ("api_key" = [])
        ),
    )]
async fn convert_xml(
    State(_store): State<Arc<Store>>,
    //string : Query<String>
    //Json(val): Json<Val>,
    //body: String,
    headers: HeaderMap,
    string: String, //json : Json<String>
) -> impl IntoResponse {
    match check_api_key(headers) {
        Ok(_) => (),
        Err(error) => return error.into_response(),
    }
    blend_result::parse_from_str_to_str(&string)
        .unwrap()
        .to_string()
        .into_response()
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
        security(
            ("api_key" = [])
        ),
    )]
#[debug_handler]
async fn upload_to_blend(
    Path(name): Path<String>,
    State(store): State<Arc<Store>>,
    headers: HeaderMap,
    data: String,
) -> response::Response {
    match check_api_key(headers) {
        Ok(_) => (),
        Err(error) => return error.into_response(),
    }
    let mut state = store.lock().await;
    // println!("The Request {:?}", data);
    // println!("The Request Data {:?}", data.len());
    state.blend_storage.push((name, data));
    ().into_response()
}

/// List files
#[utoipa::path(
        get,
        path = "/list",
        tag = "blend",
        responses(
            (status = 200, description = "List files", body = String),
        ),
        security(
            ("api_key" = [])
        ),
    )]
async fn list_to_blend(State(store): State<Arc<Store>>, headers: HeaderMap) -> response::Response {
    match check_api_key(headers) {
        Ok(_) => (),
        Err(error) => return error.into_response(),
    }
    let state = store.lock().await;
    let files = state
        .blend_storage
        .iter()
        .map(|x| x.0.clone())
        .collect::<Vec<String>>();
    format!("{:?}", files).into_response()
}

/// blend
#[utoipa::path(
        get,
        path = "/blend",
        tag = "blend",
        responses(
            (status = 200, description = "Call blend_results::blend",
                 content_type = "application/octet-stream"),
            (status = 400, description = "Errror" ),
        ),
        security(
            ("api_key" = [])
        ),
    )]
async fn blend_files(State(store): State<Arc<Store>>, headers: HeaderMap) -> response::Response {
    let mut state = store.lock().await;
    match check_api_key(headers) {
        Ok(_) => (),
        Err(error) => return error.into_response(),
    }
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
            return error.to_string().as_bytes().to_owned().into_response();
        }
    };
    let result = match mrl.export_to_ods() {
        Ok(x) => x,
        Err(error) => {
            debug!("Error while exporing");
            return error.to_string().as_bytes().to_owned().into_response();
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

    result.into_response()
    //Json(results)
}

// normally you should create a middleware for this but this is sufficient for sake of example.
fn check_api_key(headers: HeaderMap) -> Result<(), (StatusCode, Json<StuffError>)> {
    let key = match env::var("API_KEY") {
        Ok(key) => key,
        Err(e) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(StuffError::Unauthorized(String::from(format!(
                    "no api key: {e}"
                )))),
            ))
        }
    };
    match headers.get("theapikey") {
        Some(header) => {
            if *header == *key {
                Ok(())
            } else {
                Err((
                    StatusCode::UNAUTHORIZED,
                    Json(StuffError::Unauthorized(String::from("incorrect api key"))),
                ))
            }
        }
        None => Err((
            StatusCode::UNAUTHORIZED,
            Json(StuffError::Unauthorized(String::from(
                "missing api key in request",
            ))),
        )),
    }
}
