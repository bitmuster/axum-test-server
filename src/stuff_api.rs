use axum::{
    extract::{Path, Query},
    response::IntoResponse,
    Json,
};
use hyper::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

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
    OpenApiRouter::new()
        .routes(routes!(do_stuff))
        .routes(routes!(testquery))
}

/// Stuff
#[utoipa::path(
        get,
        path = "/stuff/{mul}",
        tag = "stuff",
        responses(
            (status = 200, description = "Stuff successfully", body = String),
            (status = 401, description = "Unauthorized", body = StuffError),
            // (status = 401, description = "Unauthorized", body = TodoError, example = json!(TodoError::Unauthorized(String::from("missing api key")))),
            (status = 404, description = "Stuff not found")
        ),
        params(
            ("mul" = u32, Path, description = "Multron")
        ),
        security(
            ("api_key" = [])
        ),
    )]
async fn do_stuff(Path(mul): Path<u32>, headers: HeaderMap) -> impl IntoResponse {
    match check_api_key(true, headers) {
        Ok(_) => (),
        Err(error) => return error.into_response(),
    }
    Json(String::from("Stuff").repeat(mul as usize)).into_response()
}

#[utoipa::path(
        get,
        path = "/stuff/testquery",
        tag = "stuff",
        params(
            ("name" = String, Query),
            // ("data" = String, Query),
        ),
        responses(
            (status = 200, description = "Called testquery"),
            (status = 400, description = "Whatever", body = String),
        ),
        security(
            ("api_key" = [])
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

// normally you should create a middleware for this but this is sufficient for sake of example.
fn check_api_key(
    require_api_key: bool,
    headers: HeaderMap,
) -> Result<(), (StatusCode, Json<StuffError>)> {
    match headers.get("theapikey") {
        Some(header) if header != "rocks" => Err((
            StatusCode::UNAUTHORIZED,
            Json(StuffError::Unauthorized(String::from("incorrect api key"))),
        )),
        None if require_api_key => Err((
            StatusCode::UNAUTHORIZED,
            Json(StuffError::Unauthorized(String::from("missing api key"))),
        )),
        _ => Ok(()),
    }
}
