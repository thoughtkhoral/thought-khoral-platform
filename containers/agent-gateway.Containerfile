FROM docker.io/library/rust:1.90.0-bookworm AS build

WORKDIR /source
COPY thought-khoral-agent-gateway/Cargo.toml thought-khoral-agent-gateway/Cargo.lock ./
COPY thought-khoral-agent-gateway/src/ src/
RUN cargo build --locked --release --bin thought-khoral-agent-gateway

FROM docker.io/library/debian:bookworm-slim

RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 thought-khoral \
    && useradd --no-create-home --uid 10001 --gid 10001 thought-khoral
COPY --from=build /source/target/release/thought-khoral-agent-gateway /usr/local/bin/thought-khoral-agent-gateway

USER 10001:10001
ENTRYPOINT ["/usr/local/bin/thought-khoral-agent-gateway"]
