# 002 — ThoughtKhoral project identity

## Status

Accepted

## Decision

This repository implements root [Decision 003 — ThoughtKhoral product identity](../../../../.ai/specs/decisions/003-thoughtkhoral-product-identity.md). Its direct-child directory is renamed exactly from `n2n-platform` to `thought-khoral-platform`.

The existing `n2n.room.v1` protocol values remain wire-compatible and unchanged. Database identifiers, database contents, persisted records, persisted fields, and persisted values are excluded from this rename.

## Consequences

- New project-facing identifiers use `thought-khoral-platform`.
- A future protocol or data rename requires its own approved compatibility and migration decision.
