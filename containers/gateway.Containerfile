FROM docker.io/library/rust:1.90.0-bookworm AS build

WORKDIR /source
COPY thought-khoral-room-gateway/Cargo.toml thought-khoral-room-gateway/Cargo.lock ./
COPY thought-khoral-room-gateway/src/ src/
COPY thought-khoral-room-gateway/contracts/ contracts/
RUN cargo build --locked --release

FROM docker.io/library/debian:bookworm-slim

RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 thought-khoral \
    && useradd --no-create-home --uid 10001 --gid 10001 thought-khoral
COPY --from=build /source/target/release/thought-khoral-room-gateway /usr/local/bin/thought-khoral-room-gateway
COPY --chmod=0555 thought-khoral-platform/containers/thought-khoral-gateway-entrypoint.sh /usr/local/bin/thought-khoral-gateway-entrypoint

USER 10001:10001
EXPOSE 8080
ENTRYPOINT ["thought-khoral-gateway-entrypoint"]
