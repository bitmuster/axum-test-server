
FROM docker.io/library/rust

WORKDIR /home/hektor/Repos/axum-test-server
COPY . .
COPY cert.pem .
COPY key.pem .

RUN cargo install --path .

CMD ["axum-test-server"]
