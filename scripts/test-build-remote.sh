#!/bin/sh
set -eu

platform_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
build_script="$platform_dir/scripts/build-remote.sh"

[ -x "$build_script" ] || {
  printf 'FAIL: missing executable remote build script\n' >&2
  exit 1
}

help_output=$($build_script --help)
printf '%s\n' "$help_output" | grep -Fq '<gateway-ref> <ui-ref>' || {
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
grep -Fq 'THOUGHT_KHORAL_SOURCE_ROOT' "$platform_dir/compose.yaml" || {
  printf 'FAIL: Compose does not expose the remote source-root override\n' >&2
  exit 1
}

printf 'remote-build checks passed\n'
