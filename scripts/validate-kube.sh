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

legacy_labels=$(grep -En \
  '^[[:space:]]+(n2n([.-][^:]*):|app\.kubernetes\.io/part-of:[[:space:]]+n2n([[:space:]]|$))' \
  "$kube_dir"/*.yaml || true)
[ -z "$legacy_labels" ] || \
  fail "active legacy N2N Kubernetes labels are present:\n$legacy_labels"

# Namespace and Service resources are Kubernetes API objects that Podman does
# not create. Workload manifests keep their Podman-playable Deployment first.
for manifest in postgres keycloak gateway ui; do
  path="$kube_dir/$manifest.yaml"
  podman play kube --replace --start=false "$path"
  validated_manifests="$validated_manifests $path"
done

printf 'validate-kube: all workload manifests were accepted by podman play kube\n'
