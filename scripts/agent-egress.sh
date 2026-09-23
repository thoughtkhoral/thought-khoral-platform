#!/bin/sh
# Trusted namespace setup, before either unprivileged agent starts.
set -eu

resolve_ipv4() {
  # Preserve getent's exit status. A resolver may emit a partial answer and
  # then fail; piping it directly through awk/sort would hide that failure.
  answer=$(getent ahostsv4 "$1") || return 1
  [ -n "$answer" ] || return 1
  printf '%s\n' "$answer" | awk '{print $1}' | sort -u
}

room_ips=$(resolve_ipv4 thought-khoral-room-gateway) || exit 1
keycloak_ips=$(resolve_ipv4 thought-khoral-keycloak) || exit 1
dns_ips=$(awk '$1 == "nameserver" && $2 !~ /:/ {print $2}' /etc/resolv.conf)
[ -n "$room_ips" ] && [ -n "$keycloak_ips" ] && [ -n "$dns_ips" ]
peer_ips=$(printf '%s\n%s\n' "$room_ips" "$keycloak_ips" | sort -u)

# Both IPv4 and IPv6 fail closed. The reference process (UID 10002) has
# loopback only. Only the gateway (UID 10001) can initiate broker/token or
# cluster DNS connections. The trusted setup process (UID 0) may resolve DNS
# while refreshing Compose's changing peer addresses; neither agent has
# NET_ADMIN or NET_RAW.
iptables -w -N THOUGHT_AGENT_EGRESS
iptables -w -N THOUGHT_AGENT_PEERS_A
iptables -w -N THOUGHT_AGENT_PEERS_B
for address in $peer_ips; do
  # Match the original destination, including Kubernetes Service DNAT.
  iptables -w -A THOUGHT_AGENT_PEERS_A -m owner --uid-owner 10001 -p tcp -m conntrack --ctorigdst "$address" --ctorigdstport 8080 --ctdir ORIGINAL -j ACCEPT
done
iptables -w -A THOUGHT_AGENT_EGRESS -o lo -j ACCEPT
# Rule 2 is the only mutable policy pointer. Replacing it switches the full
# peer allowlist in one kernel operation; the inactive chain is built first.
iptables -w -A THOUGHT_AGENT_EGRESS -j THOUGHT_AGENT_PEERS_A
for address in $dns_ips; do
  for uid in 0 10001; do
    for protocol in udp tcp; do
      iptables -w -A THOUGHT_AGENT_EGRESS -m owner --uid-owner "$uid" -p "$protocol" -m conntrack --ctorigdst "$address" --ctorigdstport 53 --ctdir ORIGINAL -j ACCEPT
    done
  done
done
iptables -w -A THOUGHT_AGENT_EGRESS -j REJECT
iptables -w -I OUTPUT 1 -j THOUGHT_AGENT_EGRESS
ip6tables -w -N THOUGHT_AGENT_EGRESS
ip6tables -w -A THOUGHT_AGENT_EGRESS -o lo -j ACCEPT
ip6tables -w -A THOUGHT_AGENT_EGRESS -j REJECT
ip6tables -w -I OUTPUT 1 -j THOUGHT_AGENT_EGRESS

if [ "${1:-}" = '--hold' ]; then
  touch /tmp/egress-ready
  active_chain=THOUGHT_AGENT_PEERS_A
  inactive_chain=THOUGHT_AGENT_PEERS_B
  fail_closed_refresh() {
    # If staging or switching a new policy fails, revoke the old peer
    # allowances before retrying. A failed revocation is fatal: we cannot
    # claim that the namespace is isolated when kernel policy writes fail.
    iptables -w -F "$active_chain" || exit 1
    peer_ips=
  }
  while :; do
    sleep 2
    next_room_ips=$(resolve_ipv4 thought-khoral-room-gateway) || next_room_ips=
    next_keycloak_ips=$(resolve_ipv4 thought-khoral-keycloak) || next_keycloak_ips=
    if [ -n "$next_room_ips" ] && [ -n "$next_keycloak_ips" ]; then
      next_peer_ips=$(printf '%s\n%s\n' "$next_room_ips" "$next_keycloak_ips" | sort -u)
    else
      # Never retain an address after its service name stops resolving.
      next_peer_ips=
    fi
    [ "$next_peer_ips" = "$peer_ips" ] && continue
    if ! iptables -w -F "$inactive_chain"; then
      fail_closed_refresh
      continue
    fi
    for address in $next_peer_ips; do
      if ! iptables -w -A "$inactive_chain" -m owner --uid-owner 10001 -p tcp -m conntrack --ctorigdst "$address" --ctorigdstport 8080 --ctdir ORIGINAL -j ACCEPT; then
        fail_closed_refresh
        continue 2
      fi
    done
    if ! iptables -w -R THOUGHT_AGENT_EGRESS 2 -j "$inactive_chain"; then
      fail_closed_refresh
      continue
    fi
    previous_chain=$active_chain
    active_chain=$inactive_chain
    inactive_chain=$previous_chain
    peer_ips=$next_peer_ips
    # Failure to clear the now-inactive chain does not affect the live policy;
    # the next refresh will clear it before staging any new rules.
    iptables -w -F "$inactive_chain" || :
  done
fi
