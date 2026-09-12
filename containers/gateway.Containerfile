FROM docker.io/library/rust:1.90.0-bookworm AS build

WORKDIR /source
COPY n2n-room-gateway/Cargo.toml n2n-room-gateway/Cargo.lock ./
COPY n2n-room-gateway/src/ src/
COPY n2n-room-gateway/contracts/ contracts/
RUN cargo build --locked --release

FROM docker.io/library/debian:bookworm-slim

RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 n2n \
    && useradd --no-create-home --uid 10001 --gid 10001 n2n
COPY --from=build /source/target/release/n2n-room-gateway /usr/local/bin/n2n-room-gateway
COPY --chmod=0555 n2n-platform/containers/gateway-entrypoint.sh /usr/local/bin/gateway-entrypoint

USER 10001:10001
EXPOSE 8080
ENTRYPOINT ["gateway-entrypoint"]
