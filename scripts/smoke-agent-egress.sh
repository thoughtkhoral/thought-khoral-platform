#!/bin/sh
set -eu
platform_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
probe() {
  probe_uid=$1
  shift
  podman-compose -f "$platform_dir/compose.yaml" exec -T --user "$probe_uid:$probe_uid" \
    thought-khoral-agent-egress "$@"
}
room_ip=$(probe 10001 getent ahostsv4 thought-khoral-room-gateway | awk 'NR == 1 {print $1}')
keycloak_ip=$(probe 10001 getent ahostsv4 thought-khoral-keycloak | awk 'NR == 1 {print $1}')
[ -n "$room_ip" ] && [ -n "$keycloak_ip" ]
for address in "$room_ip" "$keycloak_ip"; do
  # Any HTTP status proves connection; authorization remains application-owned.
  probe 10001 curl --silent --noproxy '*' --connect-timeout 2 --max-time 3 --output /dev/null "http://$address:8080/"
  if probe 10002 curl --silent --noproxy '*' --connect-timeout 2 --max-time 3 --output /dev/null "http://$address:8080/"; then
    printf 'agent-egress: reference UID reached %s\n' "$address" >&2
    exit 1
  fi
done
for probe_uid in 10001 10002; do
  for target in 'http://1.1.1.1:80/' 'http://[2606:4700:4700::1111]:80/'; do
    if probe "$probe_uid" curl --silent --noproxy '*' --connect-timeout 2 --max-time 3 --output /dev/null "$target"; then
      printf 'agent-egress: UID %s escaped to %s\n' "$probe_uid" "$target" >&2
      exit 1
    fi
  done
done
printf 'agent-egress: kernel allowed broker/token and denied reference/external destinations\n'
