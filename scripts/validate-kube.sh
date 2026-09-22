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
  if grep -Eq 'hostPort:|DATABASE_URL|kind: Service' "$agent_manifest" "$reference_manifest"; then
    fail 'agent workloads must not publish a host port, receive DATABASE_URL, or declare a routable Service'
  fi
  for required in 'runAsNonRoot: true' 'allowPrivilegeEscalation: false' \
    'readOnlyRootFilesystem: true' 'emptyDir: {}'; do
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
