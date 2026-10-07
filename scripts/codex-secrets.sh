#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
set -eu
# Adapter for producers that accept environment credentials. No value is printed.
THOUGHT_KHORAL_CODEX_CATALOG_BRIDGE_SECRET=$(cat /run/secrets/codex-catalog-bridge)
export THOUGHT_KHORAL_CODEX_CATALOG_BRIDGE_SECRET
case "${1:-}" in
  broker) exec /usr/local/bin/thought-khoral-gateway-entrypoint ;;
  mediator)
    THOUGHT_KHORAL_CODEX_INVOCATION_SECRET=$(cat /run/secrets/codex-invocation)
    export THOUGHT_KHORAL_CODEX_INVOCATION_SECRET
    exec /usr/local/bin/thought-khoral-agent-gateway ;;
  *) exit 1 ;;
esac
