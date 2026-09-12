#!/bin/sh
set -eu

: "${THOUGHT_KHORAL_OIDC_JWKS_URL:?THOUGHT_KHORAL_OIDC_JWKS_URL is required}"

attempt=0
until jwks=$(curl --fail --silent --show-error "$THOUGHT_KHORAL_OIDC_JWKS_URL"); do
  attempt=$((attempt + 1))
  if [ "$attempt" -ge 30 ]; then
    printf 'thought-khoral-room-gateway: OIDC JWKS unavailable after %s attempts\n' "$attempt" >&2
    exit 1
  fi
  sleep 2
done

export THOUGHT_KHORAL_OIDC_JWKS=$jwks
unset jwks
exec /usr/local/bin/thought-khoral-room-gateway
