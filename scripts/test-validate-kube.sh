#!/bin/sh
set -eu

platform_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
fixture_dir=$(mktemp -d "${TMPDIR:-/tmp}/thought-khoral-kube-test.XXXXXX")

cleanup() {
  case "$fixture_dir" in
    "${TMPDIR:-/tmp}"/thought-khoral-kube-test.*) rm -rf "$fixture_dir" ;;
  esac
}
trap cleanup EXIT INT TERM

mkdir -p "$fixture_dir/bin" "$fixture_dir/kube" "$fixture_dir/scripts"
cp "$platform_dir/scripts/validate-kube.sh" "$fixture_dir/scripts/validate-kube.sh"
cp "$platform_dir"/kube/*.yaml "$fixture_dir/kube/"
printf '%s\n' '#!/bin/sh' 'exit 0' >"$fixture_dir/bin/podman"
chmod +x "$fixture_dir/bin/podman"

PATH="$fixture_dir/bin:$PATH" sh "$fixture_dir/scripts/validate-kube.sh" >/dev/null

printf '%s\n' '    app.kubernetes.io/name: n2n-gateway' >>"$fixture_dir/kube/gateway.yaml"
if output=$(PATH="$fixture_dir/bin:$PATH" sh "$fixture_dir/scripts/validate-kube.sh" 2>&1); then
  printf 'test-validate-kube: validator accepted an active legacy name label\n' >&2
  exit 1
fi
printf '%s\n' "$output" | grep -q 'app.kubernetes.io/name: n2n-gateway' || {
  printf 'test-validate-kube: rejection did not identify the legacy label\n' >&2
  exit 1
}

printf 'test-validate-kube: compatibility allowlist and legacy-label rejection passed\n'
