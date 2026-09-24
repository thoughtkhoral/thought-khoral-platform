# Platform public documentation

## Status

Approved

## Purpose

Explain how contributors run and verify the local ThoughtKhoral platform and
understand the boundary between development composition and production
deployment.

## Acceptance criteria

- The README identifies the MVP status and platform responsibility.
- The README states the expected source-checkout layout, prerequisites,
  startup/verification/cleanup commands, compatibility volume behavior, and
  development-only credential boundary.
- Documentation names deferred production capabilities and does not present
  local Compose manifests as a production deployment.
- Documentation explains the local reference-agent egress boundary, the
  Compose sidecar fail-stop/recreate procedure, and the distinct Kubernetes
  init-container/NetworkPolicy model without promising production isolation.
- Changes follow the organization issue-first contribution workflow.
