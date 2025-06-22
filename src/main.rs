use std::net::{Ipv4Addr, SocketAddr};

use std::io::Error;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_axum::router::OpenApiRouter;
//use utoipa_rapidoc::RapiDoc;
//use utoipa_redoc::{Redoc, Servable};
//use utoipa_scalar::{Scalar, Servable as ScalarServable};
use axum_server::tls_openssl::OpenSSLConfig;
use tracing_subscriber::EnvFilter;
use utoipa_swagger_ui::SwaggerUi;

const TODO_TAG: &str = "todo";

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        // This allows you to use, e.g., `RUST_LOG=info` or `RUST_LOG=debug`
        .with_env_filter(
            EnvFilter::try_from_default_env()
                //.or_else(|_| EnvFilter::try_new("axum-test-server=trace,tower_http=warn"))
                .or_else(|_| EnvFilter::try_new("axum-test-server=trace,tower_http=warn"))
                .unwrap(),
        )
        .init();

    #[derive(OpenApi)]
    #[openapi(
        modifiers(&SecurityAddon),
        tags(
            (name = "blend", description = "Robotframework result blender"),
            (name = "stuff", description = "Various tests"),
            (name = TODO_TAG, description = "Todo items management API"),
        )
    )]
    struct ApiDoc;

    struct SecurityAddon;

    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            if let Some(components) = openapi.components.as_mut() {
                components.add_security_scheme(
                    "api_key",
                    SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("todo_apikey"))),
                )
            }
        }
    }

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api/v1/todos", todo::router())
        .split_for_parts();

    let router =
        router.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api.clone()));
    // .merge(Redoc::with_url("/redoc", api.clone()))
    // There is no need to create `RapiDoc::with_openapi` because the OpenApi is served
    // via SwaggerUi instead we only make rapidoc to point to the existing doc.
    // .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
    // Alternative to above
    // .merge(RapiDoc::with_openapi("/api-docs/openapi2.json", api).path("/rapidoc"))
    //.merge(Scalar::with_url("/scalar", api));

    //let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8080));
    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 44001));

    let config = OpenSSLConfig::from_pem_file("./cert.pem", "./key.pem").unwrap();

    axum_server::bind_openssl(address, config)
        .serve(router.into_make_service())
        .await
        .unwrap();

    /*
    let listener = TcpListener::bind(&address).await?;
    axum::serve(listener, router.into_make_service()).await*/
    Ok(())
}

mod todo {
    use std::sync::Arc;

    use crate::TODO_TAG;
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
    use utoipa::{IntoParams, ToSchema};
    use utoipa_axum::{router::OpenApiRouter, routes};

    /// In-memory todo store
    type Store = Mutex<Storage>;

    #[derive(Clone)]
    struct Storage {
        todo_storage: Vec<Todo>,
        blend_storage: Vec<(String, String)>,
    }

    impl Storage {
        fn new() -> Self {
            Storage {
                todo_storage: Vec::default(),
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
        )
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

    /// Stuff
    #[utoipa::path(
        get,
        path = "/stuff/{mul}",
        tag = "stuff",
        responses(
            (status = 200, description = "Stuff successfully"),
            (status = 404, description = "Stuff not found")
        ),
        params(
            ("mul" = u32, Path, description = "Multron")
        ),
        security(
            (), // <-- make optional authentication
            ("api_key" = [])
        )
    )]
    async fn do_stuff(
        Path(mul): Path<u32>,
        State(_store): State<Arc<Store>>,
        _headers: HeaderMap,
    ) -> Json<String> {
        Json(String::from("Stuff").repeat(mul as usize))
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
            (status = 200, description = "List matching todos by query", body = String),
            (status = 400, description = "Errror", body = String),
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
            Err(error) => return error.to_string(),
        };
        let result = match mrl.export_to_ods() {
            Ok(x) => x,
            Err(error) => return error.to_string(),
        };
        let result = match String::from_utf8(result) {
            Ok(x) => x,
            Err(error) => return error.to_string(),
        };
        println!("The Reponse has len {}", result.len());
        state.blend_storage = Vec::new();
        result
        //Json(results)
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
        )
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
                    todo.value.to_lowercase() == query.value.to_lowercase()
                        && todo.done == query.done
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
        )
    )]
    async fn create_todo(
        State(store): State<Arc<Store>>,
        Json(todo): Json<Todo>,
    ) -> impl IntoResponse {
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
            (), // <-- make optional authentication
            ("api_key" = [])
        )
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
        )
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
        match headers.get("todo_apikey") {
            Some(header) if header != "utoipa-rocks" => Err((
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
}
