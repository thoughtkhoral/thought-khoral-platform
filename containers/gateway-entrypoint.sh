#!/bin/sh
set -eu

: "${N2N_OIDC_JWKS_URL:?N2N_OIDC_JWKS_URL is required}"

attempt=0
until jwks=$(curl --fail --silent --show-error "$N2N_OIDC_JWKS_URL"); do
  attempt=$((attempt + 1))
  if [ "$attempt" -ge 30 ]; then
    printf 'gateway: OIDC JWKS unavailable after %s attempts\n' "$attempt" >&2
    exit 1
  fi
  sleep 2
done

export N2N_OIDC_JWKS=$jwks
unset jwks
exec /usr/local/bin/n2n-room-gateway
