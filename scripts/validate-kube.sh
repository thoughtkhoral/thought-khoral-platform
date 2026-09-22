#!/bin/sh
set -eu

platform_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
kube_dir="$platform_dir/kube"
manifests='namespace postgres keycloak gateway ui agent-gateway reference-agent'
validated_manifests=''

fail() {
  printf 'validate-kube: %b\n' "$*" >&2
  exit 1
}

command -v podman >/dev/null 2>&1 || fail 'podman is required'

cleanup() {
  [ -z "$validated_manifests" ] || \
    podman kube down $validated_manifests >/dev/null 2>&1 || true
}
trap cleanup EXIT INT TERM

for manifest in $manifests; do
  path="$kube_dir/$manifest.yaml"
  [ -f "$path" ] || fail "missing $path"
done

legacy_identifiers=$(awk '
  function previous_name_is(pattern) {
    return previous ~ ("^[[:space:]]+-[[:space:]]+name:[[:space:]]+(" pattern ")[[:space:]]*$")
  }
  {
    lower = tolower($0)
    if (index(lower, "n2n") != 0) {
      allowed = 0
      if ($0 ~ /^[[:space:]]+value:[[:space:]]+(n2n\.room\.v1|n2n_role)[[:space:]]*$/) allowed = 1
      if ($0 ~ /^[[:space:]]+value:[[:space:]]+n2n[[:space:]]*$/ &&
          previous_name_is("POSTGRES_DB|POSTGRES_USER|KC_DB_USERNAME")) allowed = 1
      if ($0 ~ /^[[:space:]]+value:[[:space:]]+n2n-admin-dev-only[[:space:]]*$/ &&
          previous_name_is("KC_BOOTSTRAP_ADMIN_PASSWORD")) allowed = 1
      if ($0 ~ /^[[:space:]]+value:[[:space:]]+n2n-dev-only[[:space:]]*$/ &&
          previous_name_is("POSTGRES_PASSWORD|KC_DB_PASSWORD")) allowed = 1
      if ($0 ~ /^[[:space:]]+value:[[:space:]]+postgres:\/\/n2n:n2n-dev-only@thought-khoral-postgres:5432\/n2n[[:space:]]*$/ &&
          previous_name_is("DATABASE_URL")) allowed = 1
      if ($0 ~ /^[[:space:]]+command:[[:space:]]+\[pg_isready, -U, n2n, -d, n2n\][[:space:]]*$/) allowed = 1
      if (!allowed) print FILENAME ":" FNR ":" $0
    }
    previous = $0
  }
' "$kube_dir"/*.yaml)
[ -z "$legacy_identifiers" ] || \
  fail "active legacy N2N Kubernetes identifiers are present:\n$legacy_identifiers"

# Namespace and Service resources are Kubernetes API objects that Podman does
# not create. Workload manifests keep their Podman-playable Deployment first.
require_agent_gateway_isolation() {
  agent_manifest="$kube_dir/agent-gateway.yaml"
  reference_manifest="$kube_dir/reference-agent.yaml"
  container_block() {
    container_name=$1
    awk -v container_name="$container_name" '
      $0 == "        - name: " container_name { printing = 1 }
      printing && $0 ~ /^        - name:/ && $0 != "        - name: " container_name { exit }
      printing { print }
    ' "$agent_manifest"
  }
  environment_value() {
    container=$1
    variable_name=$2
    printf '%s\n' "$container" | awk -v variable_name="$variable_name" '
      $0 ~ "^[[:space:]]+- name: " variable_name "$" { reading_value = 1; next }
      reading_value && $1 == "value:" { print $2; exit }
    '
  }
  tmp_volume_name() {
    container=$1
    printf '%s\n' "$container" | awk '
      /^[[:space:]]+volumeMounts:/ { reading_mounts = 1; next }
      reading_mounts && /^[[:space:]]+- name:/ { volume_name = $3; next }
      reading_mounts && $1 == "mountPath:" && $2 == "/tmp" { print volume_name; exit }
    '
  }
  memory_backed_volume() {
    volume_name=$1
    awk -v volume_name="$volume_name" '
      $0 == "      volumes:" { reading_volumes = 1; next }
      reading_volumes && $0 == "        - name: " volume_name { selected = 1; found = 1; next }
      selected && /^        - name:/ { selected = 0 }
      selected && $1 == "medium:" && $2 == "Memory" { memory = 1 }
      END { exit !(found && memory) }
    ' "$agent_manifest"
  }
  grep -Fq 'name: thought-khoral-agent-gateway' "$agent_manifest" || \
    fail 'missing thought-khoral-agent-gateway Kubernetes workload identity'
  grep -Fq 'name: thought-khoral-reference-agent' "$agent_manifest" || \
    fail 'reference agent must be the agent-gateway pod sidecar'
  grep -Fq 'image: localhost/thought-khoral-agent-gateway:dev' "$agent_manifest" || \
    fail 'agent gateway Kubernetes image identity does not match Compose'
  grep -Fq 'image: localhost/thought-khoral-reference-agent:dev' "$agent_manifest" || \
    fail 'reference agent Kubernetes image identity does not match Compose'
  grep -Fq 'value: http://127.0.0.1:9090/.well-known/agent-card.json' "$agent_manifest" || \
    fail 'agent gateway Kubernetes Card endpoint is not the pinned loopback endpoint'
  grep -Fq 'value: http://thought-khoral-room-gateway:8080/' "$agent_manifest" || \
    fail 'agent gateway Kubernetes room authority does not match Compose'
  grep -Fq 'value: http://thought-khoral-keycloak:8080/realms/thought-khoral/protocol/openid-connect/token' "$agent_manifest" || \
    fail 'agent gateway Kubernetes Keycloak token authority does not match Compose'
  grep -Fq 'endpoint: http://127.0.0.1:9090/.well-known/agent-card.json' "$reference_manifest" || \
    fail 'reference-agent boundary declaration drifted from the pinned Card endpoint'

  reference_agent=$(container_block thought-khoral-reference-agent)
  agent_gateway=$(container_block thought-khoral-agent-gateway)
  if printf '%s\n' "$reference_agent" | grep -Fq 'name: THOUGHT_KHORAL_AGENT_GATEWAY_CLIENT_SECRET'; then
    fail 'reference agent must not receive the Keycloak client credential'
  fi
  printf '%s\n' "$reference_agent" | grep -Fq 'name: THOUGHT_KHORAL_REFERENCE_AGENT_INBOUND_SECRET' || \
    fail 'reference agent must receive the distinct A2A inbound secret'
  printf '%s\n' "$agent_gateway" | grep -Fq 'name: THOUGHT_KHORAL_AGENT_GATEWAY_CLIENT_SECRET' || \
    fail 'agent gateway must receive the Keycloak client credential'
  printf '%s\n' "$agent_gateway" | grep -Fq 'name: THOUGHT_KHORAL_REFERENCE_AGENT_INBOUND_SECRET' || \
    fail 'agent gateway must receive the A2A inbound secret'
  reference_inbound=$(environment_value "$reference_agent" THOUGHT_KHORAL_REFERENCE_AGENT_INBOUND_SECRET)
  gateway_inbound=$(environment_value "$agent_gateway" THOUGHT_KHORAL_REFERENCE_AGENT_INBOUND_SECRET)
  gateway_client=$(environment_value "$agent_gateway" THOUGHT_KHORAL_AGENT_GATEWAY_CLIENT_SECRET)
  [ -n "$reference_inbound" ] && [ "$reference_inbound" = "$gateway_inbound" ] || \
    fail 'agent gateway and reference agent must share one non-empty inbound secret'
  [ "$reference_inbound" != "$gateway_client" ] || \
    fail 'A2A inbound secret must be distinct from the Keycloak client credential'

  reference_tmp=$(tmp_volume_name "$reference_agent")
  gateway_tmp=$(tmp_volume_name "$agent_gateway")
  [ -n "$reference_tmp" ] && [ -n "$gateway_tmp" ] || \
    fail 'each agent container must mount a writable /tmp volume'
  [ "$reference_tmp" != "$gateway_tmp" ] || \
    fail 'agent containers must mount distinct /tmp volumes'
  memory_backed_volume "$reference_tmp" || \
    fail 'agent /tmp volumes must use memory-backed emptyDir'
  memory_backed_volume "$gateway_tmp" || \
    fail 'agent /tmp volumes must use memory-backed emptyDir'

  if grep -Eq 'hostPort:|DATABASE_URL|kind: Service' "$agent_manifest" "$reference_manifest"; then
    fail 'agent workloads must not publish a host port, receive DATABASE_URL, or declare a routable Service'
  fi
  for required in 'runAsNonRoot: true' 'allowPrivilegeEscalation: false' \
    'readOnlyRootFilesystem: true'; do
    grep -Fq "$required" "$agent_manifest" || \
      fail "agent gateway Kubernetes workload is missing ${required}"
  done
}

require_agent_gateway_isolation

for manifest in postgres keycloak gateway ui agent-gateway; do
  path="$kube_dir/$manifest.yaml"
  podman play kube --replace --start=false "$path"
  validated_manifests="$validated_manifests $path"
done

printf 'validate-kube: all workload manifests were accepted by podman play kube\n'
