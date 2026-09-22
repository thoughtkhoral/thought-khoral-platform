FROM docker.io/library/rust:1.90.0-bookworm AS build

WORKDIR /source
COPY thought-khoral-agent-gateway/Cargo.toml thought-khoral-agent-gateway/Cargo.lock ./
COPY thought-khoral-agent-gateway/src/ src/
COPY thought-khoral-agent-gateway/vendor/ vendor/
RUN cargo build --locked --release --features reference-agent-server --bin thought-khoral-reference-agent

FROM docker.io/library/debian:bookworm-slim

RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10002 thought-khoral \
    && useradd --no-create-home --uid 10002 --gid 10002 thought-khoral
COPY --from=build /source/target/release/thought-khoral-reference-agent /usr/local/bin/thought-khoral-reference-agent

USER 10002:10002
ENTRYPOINT ["/usr/local/bin/thought-khoral-reference-agent"]
