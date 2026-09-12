#!/bin/sh
set -eu

platform_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
compose_file="$platform_dir/compose.yaml"
timeout_seconds=${THOUGHT_KHORAL_SMOKE_TIMEOUT_SECONDS:-180}
expected_services='thought-khoral-postgres
thought-khoral-keycloak
thought-khoral-room-gateway
thought-khoral-workspace-ui'

fail() {
  printf 'smoke: %b\n' "$*" >&2
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
  podman-compose -f "$compose_file" exec -T thought-khoral-postgres \
    pg_isready -U n2n -d n2n
}

gateway_ready() {
  status=$(curl --silent --output /dev/null --write-out '%{http_code}' \
    http://127.0.0.1:8080/ws) || return 1
  [ "$status" = 400 ]
}

require_compose_identity() {
  actual_services=$(podman-compose -f "$compose_file" config --services) || \
    fail 'Compose configuration is invalid'
  [ "$actual_services" = "$expected_services" ] || \
    fail "Compose services do not use the ThoughtKhoral identities:\n$actual_services"

  if podman-compose -f "$compose_file" config | grep -Eq '(^|[/:_-])n2n[_-]'; then
    fail 'Compose configuration contains an active legacy N2N identifier'
  fi

  legacy_containers=$(podman ps --all \
    --filter label=io.podman.compose.project=n2n \
    --format '{{.Names}}' || true)
  [ -z "$legacy_containers" ] || \
    fail "legacy N2N Compose containers are present:\n$legacy_containers"
}

require_thought_khoral_title() {
  page=$(curl --fail --silent --show-error http://127.0.0.1:8082/) || return 1
  printf '%s\n' "$page" | grep -Eq \
    '<title>[[:space:]]*ThoughtKhoral workspace[[:space:]]*</title>'
}

require_fresh_browser_entry() {
  for path in / /thought-khoral-bootstrap.js; do
    headers=$(curl --fail --silent --show-error --head \
      "http://127.0.0.1:8082$path") || return 1
    printf '%s\n' "$headers" | \
      grep -Eiq '^Cache-Control:.*(no-cache|no-store|max-age=0)' || return 1
  done
}

[ -f "$compose_file" ] || fail "missing $compose_file"
command -v podman-compose >/dev/null 2>&1 || fail 'podman-compose is required'
command -v podman >/dev/null 2>&1 || fail 'podman is required'
command -v curl >/dev/null 2>&1 || fail 'curl is required'

require_compose_identity
retry PostgreSQL postgres_ready
retry 'Keycloak realm discovery' curl --fail --silent --show-error \
  http://127.0.0.1:8081/realms/thought-khoral/.well-known/openid-configuration
retry gateway gateway_ready
retry UI curl --fail --silent --show-error http://127.0.0.1:8082/
require_fresh_browser_entry || \
  fail 'UI entry responses permit a stale pre-migration browser bootstrap'
retry 'ThoughtKhoral browser title' require_thought_khoral_title

printf 'smoke: all local services are ready\n'
