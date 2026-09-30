# syntax=docker/dockerfile:1
# Keep in sync with rust-toolchain.toml
ARG RUST_VERSION=1.98.0

# The static busybox gives the image the `mv` a move between filesystems is handed to.
FROM busybox:1.38.0-musl AS busybox

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

# Linux to run the tests on: real inotify, and /dev/shm as a second filesystem for moves between mounts.
# The source is mounted rather than copied, see docker-compose.test.yml.
FROM rust:${RUST_VERSION} AS test
COPY --from=busybox /bin/busybox /usr/local/bin/mv
WORKDIR /app
CMD ["cargo", "test", "--locked"]

FROM scratch
COPY --from=busybox /bin/busybox /bin/mv
COPY --from=builder /app/target/release/auto-renamer /auto-renamer
ENV PATH=/bin
ENTRYPOINT ["/auto-renamer"]
