# todo-axum ~ utoipa with utoipa-swagger-ui, utoipa-redoc and utoipa-rapidoc example

This is a demo `axum` application with in-memory storage to manage Todo items. The API
demonstrates `utoipa` with `utoipa-swagger-ui` functionalities.

For security restricted endpoints the super secret API key is: `utoipa-rocks`.

Just run command below to run the demo application and browse to `http://localhost:8080/swagger-ui/`.

If you prefer Redoc just head to `http://localhost:8080/redoc` and view the Open API.

RapiDoc can be found from `http://localhost:8080/rapidoc`.

Scalar can be reached on `http://localhost:8080/scalar`.


Generate certificate:

    openssl req -x509 -nodes -newkey rsa:4096 -keyout key.pem -out cert.pem -sha256 -days 365


```bash
cargo run

RUST_LOG=trace cargo run
```

Open:

    https://localhost:44001/swagger-ui/

# Docs

https://crates.io/crates/tracing-subscriber
https://docs.rs/axum/0.8.3/axum/index.html
https://docs.rs/tower-service/0.3.3/tower_service/index.html
https://docs.rs/utoipa-swagger-ui/latest/utoipa_swagger_ui/index.html
https://docs.rs/utoipa-axum/latest/utoipa_axum/index.html
https://docs.rs/utoipa/latest/utoipa/
https://docs.rs/utoipa-gen/5.4.0/utoipa_gen/attr.path.html

