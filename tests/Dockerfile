FROM rust:1.40-buster

WORKDIR /app

RUN cargo install cargo-watch && \
    rustup component add clippy

ENTRYPOINT cargo watch -c -x 'check' -x 'clippy' -x 'test'
