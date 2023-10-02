# syntax=docker/dockerfile:1
FROM lukemathwalker/cargo-chef:latest AS chef
WORKDIR sns-quicknode

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /sns-quicknode/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN mkdir -p -m 0600 ~/.ssh && ssh-keyscan github.com >> ~/.ssh/known_hosts
RUN --mount=type=ssh cargo chef cook --release --recipe-path recipe.json
# Build application
COPY src ./src
COPY Cargo.toml ./Cargo.toml
COPY Cargo.lock ./Cargo.lock
RUN --mount=type=ssh CARGO_NET_GIT_FETCH_WITH_CLI=true cargo build --release

FROM debian:bookworm-slim AS base
RUN apt-get update
RUN apt-get install -y ca-certificates

FROM base AS runtime
WORKDIR sns-quicknode
RUN mkdir certs
COPY ./certs ./certs
COPY --from=builder /sns-quicknode/target/release/sns-quicknode /usr/local/bin
ENTRYPOINT ["/usr/local/bin/sns-quicknode"]