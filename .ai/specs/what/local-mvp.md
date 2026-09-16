# Local MVP platform

## Sole MVP responsibility

`thought-khoral-platform` composes the independently built MVP services for local, rootless execution and validates the matching Kubernetes manifests.

## Acceptance criteria

- Local composition starts PostgreSQL, Keycloak, the room gateway, and the workspace UI with explicit health checks and no host-network mode.
- The Compose project is `thought-khoral`; its services and images are named `thought-khoral-postgres`, `thought-khoral-keycloak`, `thought-khoral-room-gateway`, and `thought-khoral-workspace-ui`.
- The logical Compose volume `thought-khoral-postgres-data` maps explicitly to the existing external physical volume `n2n_postgres-data`; PostgreSQL and Keycloak database credentials retain their existing development values so persisted state remains usable.
- Development identity configuration supplies the required human and agent roles without tracking production credentials.
- Gateway configuration uses the `THOUGHT_KHORAL_` namespace, and the browser host bootstrap uses `window.thoughtKhoralWorkspace`.
- Kubernetes resources use ThoughtKhoral names and labels, mirror the Compose service and environment contracts, and pass local manifest validation with non-root security settings.
- A cold UI load does not acquire a token, open an authenticated room socket, or join the retained v1 room. A valid UUID in `?room=` may prefill the UI's explicit room-entry control but does not silently join; invalid room IDs are rejected before a socket is opened.
- After the human explicitly enters or confirms a room, a fresh cookie-isolated smoke session completes OAuth 2.0 Authorization Code with PKCE, exchanges the code for a token, authenticates the WebSocket with `session.authenticate`, explicitly joins the retained v1 room, and observes an authenticated participant and room replay result.
- Leave closes the client socket, cancels reconnect, clears the active room binding, and does not clear OIDC tokens; a cold reload or revisit therefore does not replay the retained room until another explicit entry.
- Kubernetes validation rejects every active N2N identity occurrence, including label keys and values, while allowing only enumerated `n2n.room.v1`, `n2n_role`, and database compatibility values.

## Interfaces

The platform composes services that exchange `n2n.room.v1` through the gateway,
provides OIDC identity to the UI and gateway, and makes the gateway WebSocket
available to the UI. The browser bootstrap exposes an optional `roomId`
suggestion, `onEnterRoom(roomId)` and `onLeaveRoom()` lifecycle callbacks, and
the existing access-token/socket adapters. Bootstrap does not acquire a token,
construct a socket, or send `room.join`; the UI invokes those adapters only
after explicit entry. The existing PostgreSQL database and role identifiers,
persisted data, `n2n_role` identity claim, and `n2n.room.v1` wire values remain
compatibility interfaces and are not renamed.

## Explicit exclusions

This project does not define the retained v1 protocol, add a `room.leave` RPC
or durable membership, implement gateway event semantics, own the UI experience, deploy
production services, use host networking, or include production credentials.
