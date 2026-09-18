#!/bin/sh
set -eu

platform_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
compose_file="$platform_dir/compose.yaml"

command -v podman-compose >/dev/null 2>&1 || {
  printf '%s\n' 'memory smoke: podman-compose is required' >&2
  exit 1
}

services=$(podman-compose -f "$compose_file" config --services)
printf '%s\n' "$services" | grep -Fxq thought-khoral-memory-engine || {
  printf '%s\n' 'memory smoke: memory-engine service is missing' >&2
  exit 1
}

status=$(podman-compose -f "$compose_file" exec -T thought-khoral-memory-engine \
  curl --silent --output /dev/null --write-out '%{http_code}' \
  http://127.0.0.1:43121/healthz)
[ "$status" = 200 ] || {
  printf 'memory smoke: expected health status 200, got %s\n' "$status" >&2
  exit 1
}

printf '%s\n' 'memory smoke: memory-engine health and Compose service checks passed'
