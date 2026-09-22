#!/bin/sh
set -eu

platform_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
compose_file="$platform_dir/compose.yaml"
timeout_seconds=${THOUGHT_KHORAL_SMOKE_TIMEOUT_SECONDS:-180}
expected_services='thought-khoral-postgres
thought-khoral-keycloak
thought-khoral-room-gateway
thought-khoral-memory-engine
thought-khoral-reference-agent
thought-khoral-agent-gateway
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

require_running_service() {
  service_name=$1
  container_id=$(podman ps --all \
    --filter "label=io.podman.compose.project=thought-khoral" \
    --filter "label=io.podman.compose.service=$service_name" \
    --format '{{.ID}}')
  [ -n "$container_id" ] || fail "${service_name} is not running; start the complete Compose stack first"
  status=$(podman inspect --format '{{.State.Status}}' "$container_id") || \
    fail "${service_name} container could not be inspected"
  [ "$status" = running ] || fail "${service_name} is ${status}, not running"
}

service_block() {
  service_name=$1
  compose_path=$2
  awk -v service_name="$service_name" '
    $0 == "  " service_name ":" { printing = 1 }
    printing && $0 ~ /^  [[:alnum:]_-]+:$/ && $0 != "  " service_name ":" { exit }
    printing { print }
  ' "$compose_path"
}

compose_environment_value() {
  service=$1
  key=$2
  printf '%s\n' "$service" | awk -v key="$key" '
    $1 == key ":" { print $2; exit }
  '
}

require_agent_gateway_isolation() {
  for service_name in thought-khoral-agent-gateway thought-khoral-reference-agent; do
    service=$(service_block "$service_name" "$compose_file")
    [ -n "$service" ] || fail "missing ${service_name} Compose definition"
    printf '%s\n' "$service" | grep -q 'read_only: true' || \
      fail "${service_name} must use a read-only root filesystem"
    printf '%s\n' "$service" | grep -q 'no-new-privileges:true' || \
      fail "${service_name} must prohibit privilege escalation"
    printf '%s\n' "$service" | grep -q 'tmpfs:' || \
      fail "${service_name} must declare a temporary filesystem"
    printf '%s\n' "$service" | grep -q 'user: "10001:10001"' || \
      fail "${service_name} must run as the dedicated non-root identity"
    if printf '%s\n' "$service" | grep -q '^    ports:'; then
      fail "${service_name} must not publish a host port"
    fi
    if printf '%s\n' "$service" | grep -q 'DATABASE_URL'; then
      fail "${service_name} must not receive DATABASE_URL"
    fi
  done

  agent_gateway=$(service_block thought-khoral-agent-gateway "$compose_file")
  reference_agent=$(service_block thought-khoral-reference-agent "$compose_file")
  printf '%s\n' "$agent_gateway" | \
    grep -q 'network_mode: service:thought-khoral-reference-agent' || \
    fail 'agent gateway must share only the reference-agent loopback namespace'
  printf '%s\n' "$agent_gateway" | \
    grep -q 'THOUGHT_KHORAL_REFERENCE_AGENT_CARD_URL: http://127.0.0.1:9090/.well-known/agent-card.json' || \
    fail 'agent gateway must use the pinned reference-agent Card endpoint'
  printf '%s\n' "$agent_gateway" | \
    grep -q 'THOUGHT_KHORAL_ROOM_GATEWAY_ORIGIN: http://thought-khoral-room-gateway:8080/' || \
    fail 'agent gateway must use the reviewed internal room-gateway authority'
  printf '%s\n' "$agent_gateway" | \
    grep -q 'THOUGHT_KHORAL_KEYCLOAK_TOKEN_URL: http://thought-khoral-keycloak:8080/realms/thought-khoral/protocol/openid-connect/token' || \
    fail 'agent gateway must use the reviewed internal Keycloak token authority'

  if printf '%s\n' "$reference_agent" | grep -q 'THOUGHT_KHORAL_AGENT_GATEWAY_CLIENT_SECRET'; then
    fail 'reference agent must not receive the Keycloak client credential'
  fi
  printf '%s\n' "$reference_agent" | grep -q 'THOUGHT_KHORAL_REFERENCE_AGENT_INBOUND_SECRET' || \
    fail 'reference agent must receive the distinct A2A inbound secret'
  printf '%s\n' "$agent_gateway" | grep -q 'THOUGHT_KHORAL_AGENT_GATEWAY_CLIENT_SECRET' || \
    fail 'agent gateway must receive the Keycloak client credential'
  printf '%s\n' "$agent_gateway" | grep -q 'THOUGHT_KHORAL_REFERENCE_AGENT_INBOUND_SECRET' || \
    fail 'agent gateway must receive the A2A inbound secret'

  reference_inbound=$(compose_environment_value "$reference_agent" THOUGHT_KHORAL_REFERENCE_AGENT_INBOUND_SECRET)
  gateway_inbound=$(compose_environment_value "$agent_gateway" THOUGHT_KHORAL_REFERENCE_AGENT_INBOUND_SECRET)
  gateway_client=$(compose_environment_value "$agent_gateway" THOUGHT_KHORAL_AGENT_GATEWAY_CLIENT_SECRET)
  [ -n "$reference_inbound" ] && [ "$reference_inbound" = "$gateway_inbound" ] || \
    fail 'agent gateway and reference agent must share one non-empty inbound secret'
  [ "$reference_inbound" != "$gateway_client" ] || \
    fail 'A2A inbound secret must be distinct from the Keycloak client credential'
}

require_compose_identity() {
  actual_services=$(podman-compose -f "$compose_file" config --services) || \
    fail 'Compose configuration is invalid'
  [ "$actual_services" = "$expected_services" ] || \
    fail "Compose services do not use the ThoughtKhoral identities:\n$actual_services"

  require_agent_gateway_isolation

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
require_running_service thought-khoral-reference-agent
require_running_service thought-khoral-agent-gateway
require_fresh_browser_entry || \
  fail 'UI entry responses permit a stale pre-migration browser bootstrap'
retry 'ThoughtKhoral browser title' require_thought_khoral_title
node "$platform_dir/scripts/browser-smoke.mjs"
node "$platform_dir/scripts/smoke-agent-gateway.mjs"

printf 'smoke: all local services are ready\n'
