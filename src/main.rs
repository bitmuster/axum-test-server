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

mod blend_api;
mod stuff_api;
mod todo_api;

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
                    SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("theapikey"))),
                )
            }
        }
    }

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api/v1/stuff", stuff_api::router())
        .nest("/api/v1/blend", blend_api::router())
        .nest("/api/v1/todo", todo_api::router())
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
