use std::sync::Arc;

use crate::TODO_TAG;
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use hyper::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

/// In-memory todo store
type Store = Mutex<Storage>;

#[derive(Clone)]
struct Storage {
    todo_storage: Vec<Todo>,
}

impl Storage {
    fn new() -> Self {
        Storage {
            todo_storage: Vec::default(),
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
        .routes(routes!(list_todos, create_todo))
        .routes(routes!(search_todos))
        .routes(routes!(mark_done, delete_todo))
        .with_state(store)
}

/// List all Todo items
///
/// List all Todo items from in-memory storage.
#[utoipa::path(
        get,
        path = "",
        tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [Todo])
        ),
        security(
            ("api_key" = [])
        ),
    )]
async fn list_todos(State(store): State<Arc<Store>>) -> Json<Vec<Todo>> {
    let todos = store.lock().await.clone();

    Json(todos.todo_storage)
}

/// Todo search query
#[derive(Deserialize, IntoParams)]
struct TodoSearchQuery {
    /// Search by value. Search is incase sensitive.
    value: String,
    /// Search by `done` status.
    done: bool,
}

/// Search Todos by query params.
///
/// Search `Todo`s by query params and return matching `Todo`s.
#[utoipa::path(
        get,
        path = "/search",
        tag = TODO_TAG,
        params(
            TodoSearchQuery
        ),
        responses(
            (status = 200, description = "List matching todos by query", body = [Todo])
        ),
        security(
            ("api_key" = [])
        ),
    )]
async fn search_todos(
    State(store): State<Arc<Store>>,
    query: Query<TodoSearchQuery>,
) -> Json<Vec<Todo>> {
    Json(
        store
            .lock()
            .await
            .todo_storage
            .iter()
            .filter(|todo| {
                todo.value.to_lowercase() == query.value.to_lowercase() && todo.done == query.done
            })
            .cloned()
            .collect(),
    )
}

/// Create new Todo
///
/// Tries to create a new Todo item to in-memory storage or fails with 409 conflict if already exists.
#[utoipa::path(
        post,
        path = "",
        tag = TODO_TAG,
        responses(
            (status = 201, description = "Todo item created successfully", body = Todo),
            (status = 409, description = "Todo already exists", body = TodoError)
        ),
        security(
            ("api_key" = [])
        ),
    )]
async fn create_todo(State(store): State<Arc<Store>>, Json(todo): Json<Todo>) -> impl IntoResponse {
    let mut todos = store.lock().await;

    todos
        .todo_storage
        .iter_mut()
        .find(|existing_todo| existing_todo.id == todo.id)
        .map(|found| {
            (
                StatusCode::CONFLICT,
                Json(TodoError::Conflict(format!(
                    "todo already exists: {}",
                    found.id
                ))),
            )
                .into_response()
        })
        .unwrap_or_else(|| {
            todos.todo_storage.push(todo.clone());

            (StatusCode::CREATED, Json(todo)).into_response()
        })
}

/// Mark Todo item done by id
///
/// Mark Todo item done by given id. Return only status 200 on success or 404 if Todo is not found.
#[utoipa::path(
        put,
        path = "/{id}",
        tag = TODO_TAG,
        responses(
            (status = 200, description = "Todo marked done successfully"),
            (status = 404, description = "Todo not found")
        ),
        params(
            ("id" = i32, Path, description = "Todo database id")
        ),
        security(
            ("api_key" = [])
        ),
    )]
async fn mark_done(
    Path(id): Path<i32>,
    State(store): State<Arc<Store>>,
    headers: HeaderMap,
) -> StatusCode {
    match check_api_key(false, headers) {
        Ok(_) => (),
        Err(_) => return StatusCode::UNAUTHORIZED,
    }

    let mut todos = store.lock().await;

    todos
        .todo_storage
        .iter_mut()
        .find(|todo| todo.id == id)
        .map(|todo| {
            todo.done = true;
            StatusCode::OK
        })
        .unwrap_or(StatusCode::NOT_FOUND)
}

/// Delete Todo item by id
///
/// Delete Todo item from in-memory storage by id. Returns either 200 success of 404 with TodoError if Todo is not found.
#[utoipa::path(
        delete,
        path = "/{id}",
        tag = TODO_TAG,
        responses(
            (status = 200, description = "Todo marked done successfully"),
            (status = 401, description = "Unauthorized to delete Todo", body = TodoError, example = json!(TodoError::Unauthorized(String::from("missing api key")))),
            (status = 404, description = "Todo not found", body = TodoError, example = json!(TodoError::NotFound(String::from("id = 1"))))
        ),
        params(
            ("id" = i32, Path, description = "Todo database id")
        ),
        security(
            ("api_key" = [])
        ),
    )]
async fn delete_todo(
    Path(id): Path<i32>,
    State(store): State<Arc<Store>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    match check_api_key(true, headers) {
        Ok(_) => (),
        Err(error) => return error.into_response(),
    }

    let mut state = store.lock().await;
    let todos = &mut state.todo_storage;

    let len = todos.len();

    todos.retain(|todo| todo.id != id);

    if todos.len() != len {
        StatusCode::OK.into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(TodoError::NotFound(format!("id = {id}"))),
        )
            .into_response()
    }
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
