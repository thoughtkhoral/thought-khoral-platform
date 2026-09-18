FROM docker.io/library/rust:1.90.0-bookworm AS build

WORKDIR /source
COPY thought-khoral-memory-engine/Cargo.toml thought-khoral-memory-engine/Cargo.lock ./
COPY thought-khoral-memory-engine/src/ src/
RUN cargo build --locked --release

FROM docker.io/library/debian:bookworm-slim

RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 thought-khoral \
    && useradd --no-create-home --uid 10001 --gid 10001 thought-khoral
COPY --from=build /source/target/release/thought-khoral-memory-engine /usr/local/bin/thought-khoral-memory-engine

USER 10001:10001
EXPOSE 43121
ENTRYPOINT ["/usr/local/bin/thought-khoral-memory-engine"]
