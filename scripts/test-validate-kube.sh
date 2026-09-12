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

failures=0

assert_rejected() {
  case_name=$1
  legacy_line=$2
  {
    printf '%s\n' 'apiVersion: v1' 'kind: ConfigMap' 'metadata:'
    printf '%s\n' '  name: thought-khoral-validator-fixture' '  labels:'
    printf '%s\n' "$legacy_line"
  } >"$fixture_dir/kube/legacy-label.yaml"

  if output=$(PATH="$fixture_dir/bin:$PATH" sh "$fixture_dir/scripts/validate-kube.sh" 2>&1); then
    printf 'test-validate-kube: validator accepted %s\n' "$case_name" >&2
    failures=$((failures + 1))
    return
  fi
  if ! printf '%s\n' "$output" | grep -Fq "$legacy_line"; then
    printf 'test-validate-kube: rejection did not identify %s\n' "$case_name" >&2
    failures=$((failures + 1))
  fi
}

assert_rejected 'an active legacy name label' \
  '    app.kubernetes.io/name: n2n-gateway'
assert_rejected 'an active legacy n2n_role label key' \
  '    n2n_role: active-legacy-label'
assert_rejected 'a legacy app name containing an allowed contract value' \
  '    app.kubernetes.io/name: n2n-gateway-n2n.room.v1'

[ "$failures" -eq 0 ] || exit 1

printf 'test-validate-kube: compatibility allowlist and legacy-label rejection passed\n'
