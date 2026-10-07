# Codex Local Service — implementation design

## Status

Approved by the project maintainer in the Codex working session on 2026-10-05,
including this milestone-one specification and the coordinated implementation
plan. Accepted contribution: [issue 1](https://github.com/thoughtkhoral/thought-khoral-platform/issues/1).
Implementation follows the [plan](https://github.com/thoughtkhoral/thought-khoral/blob/main/.ai/specs/how/codex-room-conversations-implementation-plan.md) and its dependency gates.
Release/tag publication, provider use and service activation require their
separate later authorization. Local implementation and synthetic verification are recorded below; no live provider verification is claimed.

## Governing sources

- [Local requirements](../what/codex-local-service.md)
- [Root solution design](https://github.com/thoughtkhoral/thought-khoral/blob/main/.ai/specs/how/codex-chat-agent.md)
- [Exact profile](https://github.com/thoughtkhoral/thought-khoral-contracts/blob/thought-khoral-agent-conversation-v1.0.0/.ai/specs/how/agent-conversation-profile.md)
- [Repository tasks and gates](https://github.com/thoughtkhoral/thought-khoral/blob/main/.ai/specs/how/codex-room-conversations-implementation-plan.md)

## Design

Deploy thought-khoral-codex-agent as a separately built reviewed image, non-root
UID/GID 10003, read-only root filesystem, dropped capabilities, no published agent
port, and dedicated persistent /var/lib/thought-khoral-codex state with restrictive
ownership. Mount provider-key and invocation-key files separately under /run/secrets;
only the worker receives the provider key. Native CODEX_HOME and SQLite receipts
occupy separate subdirectories; the worker-owned fixed instruction directory is
/opt/thought-khoral-codex/workspace.
Add a dedicated restricted provider proxy; the worker can reach only that proxy,
and the proxy permits CONNECT api.openai.com:443 with valid upstream TLS, no
arbitrary hosts/ports/IP literals, and no redirects to other origins. It receives
no provider key and logs no authorization headers. Direct external IPv4/IPv6
traffic and DNS bypass from the worker are denied. The agent gateway can reach
the pinned Codex port and control paths under its existing default-deny egress
owner; preserve Reference Agent loopback-only rules. Readiness verifies image CLI
version, writable state ownership, disabled tools, and profile admission before
advertising Codex. A proxy/policy failure stops Codex; it cannot fall back to direct
egress. Extend immutable source staging to include the seventh independent worker.
Live smoke requires externally configured API-key access and records model-access
coverage explicitly; fake protocol smoke never counts as provider verification.

## Verification

Use the exact contract fixture cases, root What acceptance criteria, and assigned
repository tasks in the plan. Scope tests to real protocol/storage/UI behavior;
fake only the provider/app-server boundary where a live dependency is unnecessary.
Publication/runtime execution requires accepted issue links and written review.
Do not claim live model, history, sandbox, or egress coverage from schema tests.

## Task 8 implementation refinements (2026-10-07)

The reviewed local packaging target is Linux aarch64. The worker-owned image
binds Codex CLI 0.160.0 to the exact sanitized upstream bundled catalog,
explicit tool-selection controls, and captured empty-tool request evidence.
`--verify-package` must validate those artifacts and the CLI binary hash before
printing the tool-policy marker. Native model/list remains authoritative, with
execution restricted to reviewed catalog and model/effort capture cases.

A dedicated owner holds NET_ADMIN and the Codex network/PID namespace. Worker
UID10003 and proxy UID10004 have no capabilities. Owner PID1 exit terminates its
dependents through Linux PID namespace semantics. IPv4/IPv6 kernel rules reject
worker direct traffic and DNS. The proxy accepts only the exact
api.openai.com:443 authority, checks the selected public unicast IP with TLS,
and preserves worker end-to-end certificate verification on the actual tunnel.
Encrypted redirects cannot be inspected; attempts to use another origin are
blocked by the CONNECT/network boundary. The existing mediator namespace allows
only UID10001's admitted 9091 calls and replies to broker-initiated 9092 calls;
UID10002 retains the deterministic boundary.

The opt-in host preflight validates the immutable image reference and three
bounded, distinct credential files before recipient mounts are assembled. The
provider, worker invocation, and catalog bridge values never share recipient
scope. Startup validates native/receipt ownership and rejects imported native
configuration and authentication files. Remote source builds preserve the
four-source default and require an explicit fifth immutable worker source when
opted in; the independently built worker is passed to Compose by image ID.

Trusted worker health renews a closed host admission declaration in an envelope
that expires within five seconds. Cold UI bootstrap checks freshness and the
exact declaration. Abrupt namespace exit may leave the file; it cannot enable a
new bootstrap after expiry. Startup removes stale admission before checks.
An already-open UI retains the declaration and relies on the broker's current
admission/availability checks. No instantaneous UI withdrawal is claimed.

See the derived [local operator and verification guide](../../../docs/codex-local.md)
for precise evidence scope. Synthetic kernel, TLS, package, and CLI captures do
not establish real provider access, billing, or live room conversation behavior.
