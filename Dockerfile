
FROM debian:bookworm-slim

RUN apt update
RUN apt install libssl3

COPY target/release/axum-test-server .
COPY cert.pem .
COPY key.pem .

RUN pwd
RUN ls -lah

CMD ["/axum-test-server"]

