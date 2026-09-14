# 002 — ThoughtKhoral project identity

## Status

Accepted

## Decision

This repository implements root [Decision 003 — ThoughtKhoral product identity](https://github.com/thoughtkhoral/thought-khoral/blob/main/.ai/specs/decisions/003-thoughtkhoral-product-identity.md). Its direct-child directory is renamed exactly from `n2n-platform` to `thought-khoral-platform`.

The existing `n2n.room.v1` protocol values remain wire-compatible and unchanged. Database identifiers, database contents, persisted records, persisted fields, and persisted values are excluded from this rename.

The logical Compose volume `thought-khoral-postgres-data` therefore continues
to resolve to the existing external physical volume `n2n_postgres-data`.
Persisted PostgreSQL and Keycloak development credential values are retained;
this identity migration does not rotate or reinterpret them.

## Consequences

- New project-facing identifiers use `thought-khoral-platform`.
- A future protocol or data rename requires its own approved compatibility and migration decision.
