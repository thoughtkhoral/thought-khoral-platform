#!/bin/sh
set -eu

gateway_repo='https://github.com/thoughtkhoral/thought-khoral-room-gateway.git'
memory_engine_repo='https://github.com/thoughtkhoral/thought-khoral-memory-engine.git'
agent_gateway_repo='https://github.com/thoughtkhoral/thought-khoral-agent-gateway.git'
ui_repo='https://github.com/thoughtkhoral/thought-khoral-workspace-ui.git'
platform_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

usage() {
  printf 'Usage: %s <gateway-ref> <memory-engine-ref> <agent-gateway-ref> <ui-ref>\n' "$0"
  printf '\nBuilds and starts the platform from pinned GitHub component revisions.\n'
  printf 'Refs may be release tags or immutable commit IDs.\n'
}

fail() {
  printf 'build-remote: %s\n' "$*" >&2
  exit 1
}

if [ "${1:-}" = '--help' ] || [ "${1:-}" = '-h' ]; then
  usage
  exit 0
fi

[ "$#" -eq 4 ] || {
  usage >&2
  exit 2
}

gateway_ref=$1
memory_engine_ref=$2
agent_gateway_ref=$3
ui_ref=$4

for ref do
  case "$ref" in
    ''|-*) fail 'component refs must be non-empty and must not begin with -' ;;
  esac
done

command -v git >/dev/null 2>&1 || fail 'git is required'
command -v podman-compose >/dev/null 2>&1 || fail 'podman-compose is required'
command -v podman >/dev/null 2>&1 || fail 'podman is required'

context_root=$(mktemp -d "${TMPDIR:-/tmp}/thought-khoral-remote-build.XXXXXX")
cleanup() {
  rm -rf "$context_root"
}
trap cleanup EXIT INT TERM

mkdir -p \
  "$context_root/thought-khoral-platform" \
  "$context_root/thought-khoral-room-gateway" \
  "$context_root/thought-khoral-memory-engine" \
  "$context_root/thought-khoral-agent-gateway" \
  "$context_root/thought-khoral-workspace-ui"

clone_at_ref() {
  repo_url=$1
  destination=$2
  ref=$3

  git clone --no-checkout --quiet "$repo_url" "$destination"
  if printf '%s\n' "$ref" | grep -Eq '^[0-9a-fA-F]{40}$'; then
    git -C "$destination" fetch --depth 1 --quiet origin "$ref"
  else
    git -C "$destination" check-ref-format "refs/tags/$ref" || \
      fail "component ref must be a 40-character commit ID or a valid release tag: $ref"
    git -C "$destination" fetch --depth 1 --quiet origin "refs/tags/$ref"
  fi
  git -C "$destination" checkout --detach --quiet FETCH_HEAD

  actual_ref=$(git -C "$destination" rev-parse HEAD)
  printf '%s\n' "$actual_ref" | grep -Eq '^[0-9a-f]{40}$' || \
    fail "resolved revision for $repo_url is not a commit: $actual_ref"
  if printf '%s\n' "$ref" | grep -Eq '^[0-9a-fA-F]{40}$'; then
    expected_ref=$(printf '%s\n' "$ref" | tr '[:upper:]' '[:lower:]')
    [ "$actual_ref" = "$expected_ref" ] || \
      fail "fetched revision for $repo_url did not match requested commit $ref"
  fi
  printf 'build-remote: %s at %s\n' "$repo_url" "$actual_ref"
}

tar -C "$platform_dir" \
  --exclude .git \
  --exclude .DS_Store \
  -cf - . | tar -C "$context_root/thought-khoral-platform" -xf -

clone_at_ref "$gateway_repo" \
  "$context_root/thought-khoral-room-gateway" "$gateway_ref"
clone_at_ref "$memory_engine_repo" \
  "$context_root/thought-khoral-memory-engine" "$memory_engine_ref"
clone_at_ref "$agent_gateway_repo" \
  "$context_root/thought-khoral-agent-gateway" "$agent_gateway_ref"
clone_at_ref "$ui_repo" \
  "$context_root/thought-khoral-workspace-ui" "$ui_ref"

for rust_project in thought-khoral-room-gateway thought-khoral-memory-engine thought-khoral-agent-gateway; do
  [ -f "$context_root/$rust_project/Cargo.toml" ] || \
    fail "missing $rust_project/Cargo.toml in pinned source"
done
[ -f "$context_root/thought-khoral-workspace-ui/package.json" ] || \
  fail 'missing thought-khoral-workspace-ui/package.json in pinned source'

export THOUGHT_KHORAL_SOURCE_ROOT=$context_root
podman-compose \
  -f "$context_root/thought-khoral-platform/compose.yaml" \
  up --build -d

printf 'build-remote: platform started from pinned GitHub sources\n'
