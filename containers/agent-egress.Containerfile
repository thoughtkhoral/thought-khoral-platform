FROM docker.io/library/debian:bookworm-slim

RUN apt-get update \
    && apt-get install --yes --no-install-recommends iptables libc-bin curl \
    && rm -rf /var/lib/apt/lists/*
COPY thought-khoral-platform/scripts/agent-egress.sh /usr/local/bin/agent-egress
ENTRYPOINT ["/bin/sh", "/usr/local/bin/agent-egress"]
