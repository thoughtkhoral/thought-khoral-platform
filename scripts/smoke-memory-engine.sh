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

compose_config=$(podman-compose -f "$compose_file" config)
memory_service=$(printf '%s\n' "$compose_config" | \
  awk '
    /^  thought-khoral-memory-engine:/ { in_memory=1 }
    in_memory && NR > 1 && /^  [^[:space:]][^:]*:/ && $0 !~ /thought-khoral-memory-engine:/ { exit }
    in_memory { print }
  ')
printf '%s\n' "$memory_service" | grep -q 'DATABASE_URL' && {
  printf '%s\n' 'memory smoke: memory engine must not receive gateway DATABASE_URL' >&2
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
