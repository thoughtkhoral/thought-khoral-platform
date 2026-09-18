#!/bin/sh
set -eu

platform_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
compose_file="$platform_dir/compose.yaml"
timeout_seconds=${THOUGHT_KHORAL_SMOKE_TIMEOUT_SECONDS:-180}
expected_services='thought-khoral-postgres
thought-khoral-keycloak
thought-khoral-room-gateway
thought-khoral-memory-engine
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

memory_engine_ready() {
  status=$(podman-compose -f "$compose_file" exec -T thought-khoral-memory-engine \
    curl --silent --output /dev/null --write-out '%{http_code}' \
    http://127.0.0.1:43121/healthz) || return 1
  [ "$status" = 200 ]
}

require_compose_identity() {
  actual_services=$(podman-compose -f "$compose_file" config --services) || \
    fail 'Compose configuration is invalid'
  [ "$actual_services" = "$expected_services" ] || \
    fail "Compose services do not use the ThoughtKhoral identities:\n$actual_services"

  compose_config=$(podman-compose -f "$compose_file" config) || \
    fail 'Compose configuration is invalid'

  printf '%s\n' "$compose_config" | grep -q 'name: n2n_postgres-data' || \
    fail 'ThoughtKhoral must reuse the external n2n_postgres-data compatibility volume'
  printf '%s\n' "$compose_config" | grep -q 'external: true' || \
    fail 'the compatibility database volume must be declared external'
  printf '%s\n' "$compose_config" | grep -q 'POSTGRES_PASSWORD: n2n-dev-only' || \
    fail 'the persisted PostgreSQL password semantics changed'
  printf '%s\n' "$compose_config" | grep -q 'KC_DB_PASSWORD: n2n-dev-only' || \
    fail 'the persisted Keycloak database password semantics changed'
  printf '%s\n' "$compose_config" | grep -q 'KC_BOOTSTRAP_ADMIN_PASSWORD: n2n-admin-dev-only' || \
    fail 'the persisted Keycloak administrator password semantics changed'
  printf '%s\n' "$compose_config" | \
    grep -q 'DATABASE_URL: postgres://n2n:n2n-dev-only@thought-khoral-postgres:5432/n2n' || \
    fail 'the gateway database compatibility URL changed'

  unexpected_n2n=$(printf '%s\n' "$compose_config" | \
    grep -Ei 'n2n[_-]' | \
    grep -Ev '^[[:space:]]+(KC_BOOTSTRAP_ADMIN_PASSWORD: n2n-admin-dev-only|KC_DB_PASSWORD: n2n-dev-only|POSTGRES_PASSWORD: n2n-dev-only|DATABASE_URL: postgres://n2n:n2n-dev-only@thought-khoral-postgres:5432/n2n|name: n2n_postgres-data)$' || true)
  [ -z "$unexpected_n2n" ] || \
    fail "Compose configuration contains an active legacy N2N identifier:\n$unexpected_n2n"

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
command -v node >/dev/null 2>&1 || fail 'Node.js is required'

node "$platform_dir/scripts/test-bootstrap.mjs"

require_compose_identity
retry PostgreSQL postgres_ready
retry 'Keycloak realm discovery' curl --fail --silent --show-error \
  http://127.0.0.1:8081/realms/thought-khoral/.well-known/openid-configuration
retry gateway gateway_ready
retry memory-engine memory_engine_ready
retry UI curl --fail --silent --show-error http://127.0.0.1:8082/
require_fresh_browser_entry || \
  fail 'UI entry responses permit a stale pre-migration browser bootstrap'
retry 'ThoughtKhoral browser title' require_thought_khoral_title
node "$platform_dir/scripts/browser-smoke.mjs"

printf 'smoke: all local services are ready\n'
