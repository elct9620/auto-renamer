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

# Linux to run the tests on: real inotify, and a second filesystem for cross-device moves.
# The source is mounted rather than copied, see docker-compose.yml.
FROM rust:${RUST_VERSION} AS test
WORKDIR /app
CMD ["cargo", "test", "--locked"]

# Statically linked (musl) binary needs no runtime OS.
FROM scratch
COPY --from=builder /app/target/release/auto-renamer /auto-renamer
ENTRYPOINT ["/auto-renamer"]
