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


## Task 9 verification refinements (2026-10-07)

The platform-owned composed fixture decodes both initial and continuation native
inputs and compares their complete context to the broker's frozen packet. The
continuation has exactly the intervening public event and current trigger,
with one accepted-reply binding naming its event, source task, sequence,
generation, and text digest; prior history and assistant reply text cannot be
reinjected. A fake native protocol marker does not establish persistence:
after-turn-binding process loss waits for the actual SQLite running/thread/turn
receipt with a bounded deadline.

Each run owns a detached fixture process group and registered native groups.
Boundary guards terminate held native groups even when assertions fail; the
outer runner terminates and waits for its own descendants on success, failure,
SIGINT, or SIGTERM before removing state. It checks container removal and
surfaces cleanup errors while retaining diagnostic state. No cleanup searches
or kills resources from another fixture run. Negative cleanup verification
uses held native subprocesses and descendants, assertion failure, and both
outer signals. The synthetic denied-model failure must preserve the receipt-correlated safe
`execution_failed` code; bounded unexpected child stderr fails the check.

Live packet, credential, and new evidence paths are canonicalized through the
nearest existing ancestor and must lie outside every Git worktree, including
linked worktrees and symlinked parents. Existing packet/credential files must
be owner-only regular files; the evidence destination is preflighted before
container inspection and reserved privately before broker requests. A failed
alternate setting records only the broker's known safe failure code and fails
the live gate, retaining partial evidence. The pinned error vocabulary has no
account-specific model denial code, so account availability stays unclassified
for those failures. No live execution or provider-access evidence follows from
these guard and synthetic fixture tests.


## Final composed recovery regressions

Verification also covers completed-but-unacknowledged worker state after an
authenticated broker completion is rejected at its real deadline. Quarantine
preserves the immutable receipt; explicit fresh New must use a strictly newer
generation and a distinct native thread, including after restart. Stale
continuation and active-task replacement remain refused; no old reply is inserted.
An omitted-settings continuation by another human after restart must preserve
the accepted shared pair through native execution; a fresh New still uses the
deployment defaults. Exact broker failure projections distinguish synthetic
provider denial, missing native history, and unavailable native runtime using
matching authenticated failed receipts. These fixtures retain the published
profile and use only synthetic private state.

## Approved defaults-discovery amendment — 2026-10-07

The maintainer approved the [visible server defaults design](https://github.com/thoughtkhoral/thought-khoral-codex-agent/blob/main/.ai/specs/how/default-settings-discovery-proposal.md) in
this conversation on 2026-10-07 after an explicit specification approval request.
It authorizes coordinated local implementation and synthetic verification of
the additive authenticated defaults query and independently optional model/effort
controls, including the F1 initial/reset effort-only deadlock. The accepted
design is the governing amendment to earlier default-visibility wording.

The contracts owner defines `ResolvedSettingsView` at
`GET /api/agent-conversations/v1/rooms/{roomId}/agents/{agentId}/defaults` in
new immutable artifact `thought-khoral-agent-conversation-v1.1.0`, retaining the
v1 profile/namespace and all existing published v1.0 schema/fixture bytes.
The broker validates authenticated room/agent authority, current admission,
catalog revision, policy-default pair and five-second bound before responding.
The read has no task/event/conversation/lease/native-state mutation, exposes no
effective-settings confirmation, credentials or private/native identifiers,
uses the existing safe ProfileError/HTTP mapping and `Cache-Control: no-store`.
There is no inferred catalog-order model or inference fallback.

The UI resolves and displays the concrete explicit next-turn pair when absent
or explicitly New/reset; restored continuation uses accepted shared settings.
Both capabilities allow both controls; effort-only keeps the resolved model
read-only; model-only keeps the displayed model-specific catalog default effort
read-only; neither capability retains the settings-free path. Unsupported
controls stay uneditable and no hidden control blocks a valid required choice.
Catalog/pair mismatch requires bounded refresh or an explicit unavailable state.
A still-valid explicit pair is not replaced after a deployment-default-only change.

As a scoped exception to the earlier published-artifact-first execution order,
isolated consumers may pin a reproducible local candidate from an exact committed
contracts revision, verified archive and per-file SHA-256 values, clearly marked
unreleased. This exception is only for this amendment's local pre-publication
development and synthetic testing. Published v1.0 provenance/bytes remain intact.
This local merge does not authorize release publication, shipped interoperability,
push, provider use or service activation. Whole milestone/Task9 acceptance remains open.


## Defaults candidate verification evidence — 2026-10-07

Task 4 implements the [approved amendment plan](default-settings-discovery-implementation-plan.md)
in an isolated platform verification branch. Exact sources and artifact anchors
are recorded in `scripts/fixtures/codex-conversation/defaults-candidate-pins.json`.
The candidate retains the published v1 profile and immutable v1.0 vendor bytes;
its additive defaults route is local and unreleased. Mediator and worker remain
at their earlier reviewed v1-compatible revisions.

Actual synthetic acceptance covers 12 rendered RoomPage HTTP lifecycle cases
(four independent capability combinations × initial/restored/New), read-only
validated defaults queries, three display/send mutations and immutable replay
while defaults/catalog access is unavailable. Accepted UI settings, the frozen
broker packet and task-correlated native requests agree. The original 11 native
turns and six crash/commit boundaries remain separate from 13 candidate native
turns. A fresh New baseline contains prior public room replies in its new thread;
continuation keeps the existing strict native-reply substitution assertions.

The display/send handshake is test-only. Safe stale/removed rejections retain
the prompt and explicit Refresh settings; an attempted repeated Send performs
no additional conversation HTTP submission or ordinary chat send. The shared
composer button need not be visually disabled. No automatic alternate pair is
shown or sent. Synthetic evidence does not establish browser, packaged-stack,
real Keycloak or live-provider acceptance.

Local synthetic verification is recorded for independent specification and code
review of the exact platform commit and the scoped UI harness range
`a6aea070199e1eb2e0bd059f7cf0899131a9f092..79e5e7310a450efea561548cd87871446c1939aa`.
The task reviews and final whole-branch review passed. F1 and defaults discovery
are accepted for this local synthetic candidate; publication, packaged deployment,
live-provider use and activation retain their separate gates.

## Local main integration checkpoint — 2026-10-07

The reviewed platform candidate at source commit
`2d856773078a7caa542d719e539b55a5ab2dafaa` is integrated into this repository's
local `main` under the user's explicit merge authorization. The v1.1 contract
remains an unreleased local candidate. Fresh candidate pin/safety verification
passed all 26 tests with `node --test scripts/tests/codex-defaults-pins.test.mjs`.
The provider-free composed run passed with
`node scripts/smoke-codex-conversation.mjs --fake --defaults-candidate`, including
the rendered defaults matrix and recovery checks; its temporary PostgreSQL
container and owned fixture processes were cleaned. This does not establish
packaged deployment, real browser/identity, or live-provider acceptance, and
does not authorize publication, activation, provider use, or push.
