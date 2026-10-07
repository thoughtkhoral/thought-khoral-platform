#!/bin/sh
set -eu

platform_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
build_script="$platform_dir/scripts/build-remote.sh"

[ -x "$build_script" ] || {
  printf 'FAIL: missing executable remote build script\n' >&2
  exit 1
}

help_output=$($build_script --help)
printf '%s\n' "$help_output" | grep -Fq '<gateway-ref> <memory-engine-ref> <agent-gateway-ref> <ui-ref>' || {
  printf 'FAIL: remote build help does not document pinned component refs\n' >&2
  exit 1
}

grep -Fq 'https://github.com/thoughtkhoral/thought-khoral-room-gateway.git' "$build_script" || {
  printf 'FAIL: remote build does not identify the gateway repository\n' >&2
  exit 1
}
grep -Fq 'https://github.com/thoughtkhoral/thought-khoral-workspace-ui.git' "$build_script" || {
  printf 'FAIL: remote build does not identify the UI repository\n' >&2
  exit 1
}
grep -Fq 'https://github.com/thoughtkhoral/thought-khoral-agent-gateway.git' "$build_script" || {
  printf 'FAIL: remote build does not identify the agent-gateway repository\n' >&2
  exit 1
}
grep -Fq 'THOUGHT_KHORAL_SOURCE_ROOT' "$platform_dir/compose.yaml" || {
  printf 'FAIL: Compose does not expose the remote source-root override\n' >&2
  exit 1
}

fixture_root=$(mktemp -d "${TMPDIR:-/tmp}/thought-khoral-remote-test.XXXXXX")
trap 'rm -rf "$fixture_root"' EXIT
mkdir -p "$fixture_root/bin"
printf synthetic-task8-provider-key-123456 > "$fixture_root/provider"
printf synthetic-task8-invocation-key-4567 > "$fixture_root/invocation"
printf synthetic-task8-bridge-secret-7890123 > "$fixture_root/bridge"
export THOUGHT_KHORAL_CODEX_PROVIDER_KEY_FILE="$fixture_root/provider"
export THOUGHT_KHORAL_CODEX_INVOCATION_KEY_FILE="$fixture_root/invocation"
export THOUGHT_KHORAL_CODEX_CATALOG_BRIDGE_KEY_FILE="$fixture_root/bridge"

cat >"$fixture_root/bin/git" <<'SH'
#!/bin/sh
set -eu
if [ "$1" = clone ]; then
  mkdir -p "$5"
  printf '%s\n' "$4" >"$5/.source-url"
  case "$4" in
    *thought-khoral-workspace-ui.git) touch "$5/package.json" ;;
    *thought-khoral-memory-engine.git)
      [ "${THOUGHT_KHORAL_TEST_SKIP_MEMORY:-0}" = 1 ] || touch "$5/Cargo.toml"
      ;;
    *thought-khoral-codex-agent.git)
      touch "$5/Cargo.toml"
      [ "${THOUGHT_KHORAL_TEST_SKIP_CODEX:-0}" = 1 ] || touch "$5/Containerfile"
      ;;
    *) touch "$5/Cargo.toml" ;;
  esac
  exit 0
fi
[ "$1" = -C ] || exit 1
destination=$2
shift 2
case "$1" in
  fetch)
    for argument do ref=$argument; done
    printf '%s\n' "$ref" >"$destination/.ref"
    ;;
  checkout) ;;
  rev-parse) cat "$destination/.ref" ;;
  *) exit 1 ;;
esac
SH

cat >"$fixture_root/bin/podman-compose" <<'SH'
#!/bin/sh
set -eu
root=$THOUGHT_KHORAL_SOURCE_ROOT
test "$1" = -f
test "$2" = "$root/thought-khoral-platform/compose.yaml"
shift 2
if [ "${THOUGHT_KHORAL_TEST_CODEX:-0}" = 1 ]; then
  test "$1" = -f
  test "$2" = "$root/thought-khoral-platform/compose.codex.yaml"
  shift 2
  test -f "$root/thought-khoral-codex-agent/.source-url"
  test "$(cat "$root/thought-khoral-codex-agent/.ref")" = eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee
  test "$THOUGHT_KHORAL_CODEX_IMAGE" = sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
fi
test "$1" = up
test "$2" = --build
test "$3" = -d
for component in room-gateway memory-engine agent-gateway workspace-ui; do
  test -f "$root/thought-khoral-$component/.source-url"
  test -f "$root/thought-khoral-$component/.ref"
done
test "$(cat "$root/thought-khoral-room-gateway/.ref")" = aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
test "$(cat "$root/thought-khoral-memory-engine/.ref")" = bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
test "$(cat "$root/thought-khoral-agent-gateway/.ref")" = cccccccccccccccccccccccccccccccccccccccc
test "$(cat "$root/thought-khoral-workspace-ui/.ref")" = dddddddddddddddddddddddddddddddddddddddd
touch "$THOUGHT_KHORAL_TEST_FIXTURE_ROOT/compose-called"
SH

cat >"$fixture_root/bin/podman" <<'SH'
#!/bin/sh
set -eu
if [ "${1:-}" = build ]; then
  test "$2" = --iidfile
  printf 'sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff\n' > "$3"
  touch "$THOUGHT_KHORAL_TEST_FIXTURE_ROOT/build-called"
fi
SH
chmod +x "$fixture_root/bin/git" "$fixture_root/bin/podman-compose" "$fixture_root/bin/podman"

THOUGHT_KHORAL_TEST_FIXTURE_ROOT="$fixture_root" PATH="$fixture_root/bin:$PATH" \
  "$build_script" \
  aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
  bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb \
  cccccccccccccccccccccccccccccccccccccccc \
  dddddddddddddddddddddddddddddddddddddddd || {
    printf 'FAIL: pinned remote build did not stage all four component sources\n' >&2
    exit 1
  }
test -f "$fixture_root/compose-called" || {
  printf 'FAIL: staged remote sources did not reach Compose\n' >&2
  exit 1
}

rm "$fixture_root/compose-called"
if THOUGHT_KHORAL_TEST_FIXTURE_ROOT="$fixture_root" \
  THOUGHT_KHORAL_TEST_SKIP_MEMORY=1 PATH="$fixture_root/bin:$PATH" \
  "$build_script" \
  aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
  bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb \
  cccccccccccccccccccccccccccccccccccccccc \
  dddddddddddddddddddddddddddddddddddddddd; then
  printf 'FAIL: missing memory-engine build source was accepted\n' >&2
  exit 1
fi
test ! -f "$fixture_root/compose-called" || {
  printf 'FAIL: Compose started with a missing memory-engine build source\n' >&2
  exit 1
}

# The fifth source is required before any clone/build/start in explicit opt-in mode.
if THOUGHT_KHORAL_TEST_FIXTURE_ROOT="$fixture_root" PATH="$fixture_root/bin:$PATH" \
  "$build_script" --codex aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb cccccccccccccccccccccccccccccccccccccccc dddddddddddddddddddddddddddddddddddddddd; then
  printf 'FAIL: opt-in accepted without fifth revision\n' >&2; exit 1
fi
test ! -f "$fixture_root/build-called"
THOUGHT_KHORAL_TEST_FIXTURE_ROOT="$fixture_root" THOUGHT_KHORAL_TEST_CODEX=1 PATH="$fixture_root/bin:$PATH" \
  "$build_script" --codex aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb cccccccccccccccccccccccccccccccccccccccc dddddddddddddddddddddddddddddddddddddddd eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee
test -f "$fixture_root/build-called"
test -f "$fixture_root/compose-called"
rm "$fixture_root/build-called" "$fixture_root/compose-called"
if THOUGHT_KHORAL_TEST_FIXTURE_ROOT="$fixture_root" THOUGHT_KHORAL_TEST_CODEX=1 THOUGHT_KHORAL_TEST_SKIP_CODEX=1 PATH="$fixture_root/bin:$PATH" \
  "$build_script" --codex aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb cccccccccccccccccccccccccccccccccccccccc dddddddddddddddddddddddddddddddddddddddd eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee; then
  printf 'FAIL: missing Codex source accepted\n' >&2; exit 1
fi
test ! -f "$fixture_root/build-called"
test ! -f "$fixture_root/compose-called"
printf 'remote-build checks passed\n'
