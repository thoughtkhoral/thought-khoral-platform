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

rm -f "$fixture_dir/kube/legacy-label.yaml"
awk '
  /value: reference-agent-inbound-dev-only/ && !injected {
    print
    print "            - name: THOUGHT_KHORAL_AGENT_GATEWAY_CLIENT_SECRET"
    print "              value: forbidden-client-secret"
    injected = 1
    next
  }
  { print }
  END { if (!injected) exit 2 }
' "$fixture_dir/kube/agent-gateway.yaml" >"$fixture_dir/kube/agent-gateway.yaml.next"
mv "$fixture_dir/kube/agent-gateway.yaml.next" "$fixture_dir/kube/agent-gateway.yaml"

if output=$(PATH="$fixture_dir/bin:$PATH" sh "$fixture_dir/scripts/validate-kube.sh" 2>&1); then
  printf 'test-validate-kube: validator accepted a Keycloak client secret in the reference-agent container\n' >&2
  failures=$((failures + 1))
elif ! printf '%s\n' "$output" | grep -Fq 'reference agent must not receive the Keycloak client credential'; then
  printf 'test-validate-kube: credential-separation rejection was not identified\n' >&2
  failures=$((failures + 1))
fi

cp "$platform_dir/kube/agent-gateway.yaml" "$fixture_dir/kube/agent-gateway.yaml"
awk '
  /^[[:space:]]+volumeMounts:/ { in_volume_mounts = 1; print; next }
  in_volume_mounts && /^        - name:/ { in_volume_mounts = 0 }
  in_volume_mounts && /^[[:space:]]+- name:/ {
    sub(/name: .*/, "name: shared-agent-tmp")
  }
  { print }
' "$fixture_dir/kube/agent-gateway.yaml" >"$fixture_dir/kube/agent-gateway.yaml.next"
mv "$fixture_dir/kube/agent-gateway.yaml.next" "$fixture_dir/kube/agent-gateway.yaml"

if output=$(PATH="$fixture_dir/bin:$PATH" sh "$fixture_dir/scripts/validate-kube.sh" 2>&1); then
  printf 'test-validate-kube: validator accepted a shared writable agent /tmp volume\n' >&2
  failures=$((failures + 1))
elif ! printf '%s\n' "$output" | grep -Fq 'agent containers must mount distinct /tmp volumes'; then
  printf 'test-validate-kube: shared /tmp rejection was not identified\n' >&2
  failures=$((failures + 1))
fi

cp "$platform_dir/kube/agent-gateway.yaml" "$fixture_dir/kube/agent-gateway.yaml"
awk '$0 !~ /^[[:space:]]+medium: Memory$/ { print }' \
  "$fixture_dir/kube/agent-gateway.yaml" >"$fixture_dir/kube/agent-gateway.yaml.next"
mv "$fixture_dir/kube/agent-gateway.yaml.next" "$fixture_dir/kube/agent-gateway.yaml"

if output=$(PATH="$fixture_dir/bin:$PATH" sh "$fixture_dir/scripts/validate-kube.sh" 2>&1); then
  printf 'test-validate-kube: validator accepted a disk-backed agent /tmp volume\n' >&2
  failures=$((failures + 1))
elif ! printf '%s\n' "$output" | grep -Fq 'agent /tmp volumes must use memory-backed emptyDir'; then
  printf 'test-validate-kube: disk-backed /tmp rejection was not identified\n' >&2
  failures=$((failures + 1))
fi

[ "$failures" -eq 0 ] || exit 1

printf 'test-validate-kube: compatibility allowlist and legacy-label rejection passed\n'
