# Local MVP platform

## Sole MVP responsibility

`n2n-platform` composes the independently built MVP services for local, rootless execution and validates the matching Kubernetes manifests.

## Acceptance criteria

- Local composition starts PostgreSQL, Keycloak, the room gateway, and the workspace UI with explicit health checks and no host-network mode.
- Development identity configuration supplies the required human and agent roles without tracking production credentials.
- Kubernetes manifests use the same service names and environment contracts and pass local manifest validation with non-root security settings.

## Interfaces

The platform composes services that exchange `n2n.room.v1` through the gateway, provides OIDC identity to the UI and gateway, and makes the gateway WebSocket available to the UI.

## Explicit exclusions

This project does not define `n2n.room.v1`, implement gateway event semantics, own the UI experience, deploy production services, use host networking, or include production credentials.
