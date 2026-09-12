#!/bin/sh
set -eu

platform_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
kube_dir="$platform_dir/kube"
manifests='namespace postgres keycloak gateway ui'
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
  {
    lower = tolower($0)
    if (index(lower, "n2n") == 0) next
    if (lower ~ /n2n\.room\.v1/ || lower ~ /n2n_role/) next
    if ($0 ~ /^[[:space:]]+value:[[:space:]]+n2n[[:space:]]*$/) next
    if ($0 ~ /^[[:space:]]+value:[[:space:]]+n2n-admin-dev-only[[:space:]]*$/) next
    if ($0 ~ /^[[:space:]]+value:[[:space:]]+n2n-dev-only[[:space:]]*$/) next
    if ($0 ~ /^[[:space:]]+value:[[:space:]]+postgres:\/\/n2n:n2n-dev-only@thought-khoral-postgres:5432\/n2n[[:space:]]*$/) next
    if ($0 ~ /^[[:space:]]+command:[[:space:]]+\[pg_isready, -U, n2n, -d, n2n\][[:space:]]*$/) next
    print FILENAME ":" FNR ":" $0
  }
' "$kube_dir"/*.yaml)
[ -z "$legacy_identifiers" ] || \
  fail "active legacy N2N Kubernetes identifiers are present:\n$legacy_identifiers"

# Namespace and Service resources are Kubernetes API objects that Podman does
# not create. Workload manifests keep their Podman-playable Deployment first.
for manifest in postgres keycloak gateway ui; do
  path="$kube_dir/$manifest.yaml"
  podman play kube --replace --start=false "$path"
  validated_manifests="$validated_manifests $path"
done

printf 'validate-kube: all workload manifests were accepted by podman play kube\n'
