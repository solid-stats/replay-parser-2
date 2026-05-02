# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.95.0

FROM rust:${RUST_VERSION}-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo build --release -p parser-cli

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/replay-parser-2 /usr/local/bin/replay-parser-2

USER 65532:65532
ENTRYPOINT ["replay-parser-2"]
CMD ["worker"]
