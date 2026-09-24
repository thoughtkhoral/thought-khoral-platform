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
test "$3" = up
test "$4" = --build
test "$5" = -d
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
exit 0
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

printf 'remote-build checks passed\n'
