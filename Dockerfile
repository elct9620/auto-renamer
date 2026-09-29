# syntax=docker/dockerfile:1
# Keep in sync with rust-toolchain.toml
ARG RUST_VERSION=1.98.0

FROM rust:${RUST_VERSION}-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app

# Build dependencies first so they stay cached until the manifest changes.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs \
    && cargo build --release --locked \
    && rm -rf src target/release/auto-renamer target/release/deps/auto_renamer*

COPY src ./src
RUN cargo build --release --locked

# Statically linked (musl) binary needs no runtime OS.
FROM scratch
COPY --from=builder /app/target/release/auto-renamer /auto-renamer
ENTRYPOINT ["/auto-renamer"]
