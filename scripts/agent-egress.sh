#!/bin/sh
# Trusted namespace setup, before either unprivileged agent starts.
set -eu

room_ips=$(getent ahostsv4 thought-khoral-room-gateway | awk '{print $1}' | sort -u)
keycloak_ips=$(getent ahostsv4 thought-khoral-keycloak | awk '{print $1}' | sort -u)
dns_ips=$(awk '$1 == "nameserver" && $2 !~ /:/ {print $2}' /etc/resolv.conf)
[ -n "$room_ips" ] && [ -n "$keycloak_ips" ] && [ -n "$dns_ips" ]

# Both IPv4 and IPv6 fail closed. The reference process (UID 10002) has
# loopback only. Only the gateway (UID 10001) can initiate broker/token or
# cluster DNS connections. Neither application has NET_ADMIN or NET_RAW.
for firewall in iptables ip6tables; do
  "$firewall" -w -N THOUGHT_AGENT_EGRESS
  "$firewall" -w -A THOUGHT_AGENT_EGRESS -j REJECT
  "$firewall" -w -I OUTPUT 1 -j THOUGHT_AGENT_EGRESS
  "$firewall" -w -I THOUGHT_AGENT_EGRESS 1 -o lo -j ACCEPT
  "$firewall" -w -I THOUGHT_AGENT_EGRESS 1 -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT
done
for address in $room_ips $keycloak_ips; do
  # Match the original destination, including Kubernetes Service DNAT.
  iptables -w -I THOUGHT_AGENT_EGRESS 1 -m owner --uid-owner 10001 -p tcp -m conntrack --ctorigdst "$address" --ctorigdstport 8080 --ctdir ORIGINAL -j ACCEPT
done
for address in $dns_ips; do
  for protocol in udp tcp; do
    iptables -w -I THOUGHT_AGENT_EGRESS 1 -m owner --uid-owner 10001 -p "$protocol" -m conntrack --ctorigdst "$address" --ctorigdstport 53 --ctdir ORIGINAL -j ACCEPT
  done
done

if [ "${1:-}" = '--hold' ]; then
  touch /tmp/egress-ready
  exec sleep infinity
fi
