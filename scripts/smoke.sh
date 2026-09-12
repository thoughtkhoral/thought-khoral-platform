#!/bin/sh
set -eu

platform_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
compose_file="$platform_dir/compose.yaml"
timeout_seconds=${N2N_SMOKE_TIMEOUT_SECONDS:-180}

fail() {
  printf 'smoke: %s\n' "$*" >&2
  exit 1
}

retry() {
  description=$1
  shift
  attempts=0
  max_attempts=$((timeout_seconds / 2 + 1))
  while ! "$@" >/dev/null 2>&1; do
    attempts=$((attempts + 1))
    if [ "$attempts" -ge "$max_attempts" ]; then
      fail "$description did not become ready within ${timeout_seconds}s"
    fi
    sleep 2
  done
  printf 'smoke: %s is ready\n' "$description"
}

postgres_ready() {
  podman-compose -f "$compose_file" exec -T postgres \
    pg_isready -U n2n -d n2n
}

gateway_ready() {
  status=$(curl --silent --output /dev/null --write-out '%{http_code}' \
    http://127.0.0.1:8080/ws) || return 1
  [ "$status" = 400 ]
}

[ -f "$compose_file" ] || fail "missing $compose_file"
command -v podman-compose >/dev/null 2>&1 || fail 'podman-compose is required'
command -v curl >/dev/null 2>&1 || fail 'curl is required'

retry PostgreSQL postgres_ready
retry 'Keycloak realm discovery' curl --fail --silent --show-error \
  http://127.0.0.1:8081/realms/n2n/.well-known/openid-configuration
retry gateway gateway_ready
retry UI curl --fail --silent --show-error http://127.0.0.1:8082/

printf 'smoke: all local services are ready\n'
