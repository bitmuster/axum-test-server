
# Axum Testserver for Robotframework Result Blender

utoipa with utoipa-swagger-ui example.
Based on: https://github.com/juhaku/utoipa/tree/master/examples/todo-axum

Experimental feature is to blend robotframework files.
See also : https://github.com/bitmuster/BlendResult .

# Generate certificate:

    openssl req -x509 -nodes -newkey rsa:4096 -keyout key.pem -out cert.pem -sha256 -days 365


```bash
cargo run

RUST_LOG=trace cargo run
```

Open:

    https://localhost:44001/swagger-ui/

# Docs

* https://crates.io/crates/tracing-subscriber
* https://docs.rs/axum/0.8.3/axum/index.html
* https://docs.rs/tower-service/0.3.3/tower_service/index.html
* https://docs.rs/utoipa-swagger-ui/latest/utoipa_swagger_ui/index.html
* https://docs.rs/utoipa-axum/latest/utoipa_axum/index.html
* https://docs.rs/utoipa/latest/utoipa/
* https://docs.rs/utoipa-gen/5.4.0/utoipa_gen/attr.path.html


# Podman Container

The container currently uses the prebuilt binary for debugging purposes.

    cargo build --release
    podman build .
    podman run -p 44001:44001 <id>
    https://localhost:44001/swagger-ui/
    https://<ip>:44001/swagger-ui/
