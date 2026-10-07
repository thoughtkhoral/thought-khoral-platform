# SPDX-License-Identifier: Apache-2.0
FROM docker.io/library/debian:bookworm-slim@sha256:3783cc01769c7b2b1b83a5c5ad96c815348e28ed7da68e2e3687004faa906251
COPY thought-khoral-platform/proxy/debian-packages-aarch64.lock /opt/codex/debian-packages.lock
RUN test "$(dpkg --print-architecture)" = arm64 \
    && apt-get update \
    && xargs apt-get install --yes --no-install-recommends --no-upgrade < /opt/codex/debian-packages.lock \
    && rm -rf /var/lib/apt/lists/*
COPY thought-khoral-platform/proxy/codex-provider.py thought-khoral-platform/proxy/codex-provider.conf /opt/codex/
COPY thought-khoral-platform/scripts/codex-egress.sh thought-khoral-platform/scripts/codex-egress.py /opt/codex/
COPY thought-khoral-platform/proxy/THIRD_PARTY_NOTICES.md /usr/share/doc/thought-khoral-codex-proxy/THIRD_PARTY_NOTICES.md
USER 10004:10004
ENTRYPOINT ["python3", "/opt/codex/codex-provider.py"]
