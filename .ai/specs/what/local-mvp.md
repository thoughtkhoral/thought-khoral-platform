# Local MVP platform

## Sole MVP responsibility

`thought-khoral-platform` composes the independently built MVP services for local, rootless execution and validates the matching Kubernetes manifests.

## Acceptance criteria

- Local composition starts PostgreSQL, Keycloak, the room gateway, and the workspace UI with explicit health checks and no host-network mode.
- The Compose project is `thought-khoral`; its services and images are named `thought-khoral-postgres`, `thought-khoral-keycloak`, `thought-khoral-room-gateway`, and `thought-khoral-workspace-ui`, and its persistent volume is `thought-khoral-postgres-data`.
- Development identity configuration supplies the required human and agent roles without tracking production credentials.
- Gateway configuration uses the `THOUGHT_KHORAL_` namespace, and the browser host bootstrap uses `window.thoughtKhoralWorkspace`.
- Kubernetes resources use ThoughtKhoral names and labels, mirror the Compose service and environment contracts, and pass local manifest validation with non-root security settings.
- The authenticated browser room displays the ThoughtKhoral document title and visible product heading.

## Interfaces

The platform composes services that exchange `n2n.room.v1` through the gateway, provides OIDC identity to the UI and gateway, and makes the gateway WebSocket available to the UI. The existing PostgreSQL database and role identifiers, persisted data, `n2n_role` identity claim, and `n2n.room.v1` wire values remain compatibility interfaces and are not renamed.

## Explicit exclusions

This project does not define `n2n.room.v1`, implement gateway event semantics, own the UI experience, deploy production services, use host networking, or include production credentials.
