# Local MVP platform

## Sole MVP responsibility

`thought-khoral-platform` composes independently built MVP services and
incubating local proofs for rootless development. It validates the matching
Kubernetes manifests where a workload has a Kubernetes profile; the
memory-engine ingestion proof is currently Compose-only.

## Acceptance criteria

- Local composition starts PostgreSQL, Keycloak, the room gateway, workspace
  UI, memory-engine ingestion proof, egress owner, agent gateway, and reference
  agent with health checks and no host-network mode.
- The Compose project is `thought-khoral`; its service and image names use
  the `thought-khoral-` prefix. The egress container has the fixed name
  `thought-khoral-agent-egress` so both agents can join its PID namespace.
- The pinned remote-source helper requires immutable revisions for all four
  source-built components: room gateway, memory engine, agent gateway, and UI.
  It stages each under the same sibling layout as the local build before
  starting Compose; a missing source must fail before any build starts.
- The logical Compose volume `thought-khoral-postgres-data` maps explicitly to the existing external physical volume `n2n_postgres-data`; PostgreSQL and Keycloak database credentials retain their existing development values so persisted state remains usable.
- Development identity configuration supplies the required human and agent roles without tracking production credentials.
- Gateway configuration uses the `THOUGHT_KHORAL_` namespace, and the browser host bootstrap uses `window.thoughtKhoralWorkspace`.
- Kubernetes resources use ThoughtKhoral names and labels, mirror the
  applicable Compose service and environment contracts, and pass local
  manifest validation with non-root security settings. The memory-engine
  proof has no Kubernetes workload manifest yet.
- A cold UI load does not acquire a token, open an authenticated room socket, or join the retained v1 room. A valid UUID in `?room=` may prefill the UI's explicit room-entry control but does not silently join; invalid room IDs are rejected before a socket is opened.
- After the human explicitly enters or confirms a room, a fresh cookie-isolated smoke session completes OAuth 2.0 Authorization Code with PKCE, exchanges the code for a token, authenticates the WebSocket with `session.authenticate`, explicitly joins the retained v1 room, and observes an authenticated participant and room replay result.
- Leave closes the client socket, cancels reconnect, clears the active room binding, and does not clear OIDC tokens; a cold reload or revisit therefore does not replay the retained room until another explicit entry.
- Kubernetes validation rejects every active N2N identity occurrence, including label keys and values, while allowing only enumerated `n2n.room.v1`, `n2n_role`, and database compatibility values.
- The local stack runs one pinned deterministic A2A reference agent and its
  mediator without database credentials or host-published agent ports. A
  human can invoke both permitted skills, observe durable progress, and receive
  a source-cited terminal result; a hidden targeted message is absent from the
  reference agent's authorized context packet.
- Compose installs default-deny IPv4/IPv6 agent egress before either agent
  starts, refreshes allowed room/identity peer addresses while running, and
  stops both agents when the trusted egress namespace owner exits. Recovery
  recreates that owner and both dependents as a group. Kubernetes retains its
  separate init-container plus NetworkPolicy model.

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

The local A2A addition composes the room gateway, agent gateway, and reference
agent through an authenticated internal task interface. The room gateway alone
selects authorized room context and persists normalized task events. The agent
gateway invokes only the pinned local Agent Card and skills; neither agent
receives room storage authority.

The memory-engine proof receives committed room events through a separate
private authenticated ingestion endpoint after gateway persistence. It does
not read the gateway database or become an active decision authority.

## Explicit exclusions

This project does not define the retained v1 protocol, add a `room.leave` RPC
or durable membership, implement gateway event semantics, own the UI experience, deploy
production services, use host networking, include production credentials, or
admit remote third-party agents or MCP runtimes.
