# Codex Local Service — implementation design

## Status

Approved by the project maintainer in the Codex working session on 2026-10-05,
including this milestone-one specification and the coordinated implementation
plan. Accepted contribution: [issue 1](https://github.com/thoughtkhoral/thought-khoral-platform/issues/1).
Implementation follows the [plan](https://github.com/thoughtkhoral/thought-khoral/blob/main/.ai/specs/how/codex-room-conversations-implementation-plan.md) and its dependency gates.
Release/tag publication, provider use and service activation require their
separate later authorization. No completed runtime or live verification is claimed.

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
occupy separate subdirectories; /workspace contains fixed instructions only.
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

## Task 8 local packaging checkpoint — 2026-10-07

Platform `9626bc46f8b74c2a58b2578c4f744996f3d41317` on `codex-opt-in-platform` and worker `d40e4a8cd5efc77c7161742aec7ade289efb357a` on
`codex-worker-tool-policy` passed independent task review. The coordinated plan
records immutable image identities and synthetic kernel/TLS/startup/native-tool
capture evidence. Feature flags alone were insufficient for initial tool
restriction; the corrected pinned worker binds sanitized catalog, controls and
actual CLI evidence. Linux ARM64 is verified locally; x86_64 remains blocked on
equivalent capture. Cold bootstrap admission expires within five seconds after
health renewal stops; no instantaneous withdrawal from already-open UI is claimed.
Original checkouts retain their runtimes/scaffold. No provider inference,
activation, merge or publication occurred. Task 9 retains end-to-end integration
and separately authorized live verification; directory guidance remains later.

## Task 9 synthetic verification and correction checkpoint — 2026-10-07

The following defaults-discovery amendment status supersedes this pre-amendment F1/default-discovery disposition; Task 9 remains open for release gates.

Provider-free checkpoint only; Task 9 and the milestone remain open.

Task-scoped verification review: Approved. Broad implementation review: Partial
spec compliance; quality Needs follow-up. B1–B3 (pending-ack recovery, omitted
shared settings and receipt-correlated safe failures) are addressed. B4 is
partial: explicit initial/reset selection works for full-capability admission,
but automatic server-default display requires an approved interface amendment.
F1: unresolved Important reasoning-only UI deadlock. Optional capabilities are
independent; an effort-only admission cannot establish the guard-required model
through its hidden selector. This prevents initial/reset invocation and blocks
whole-milestone/merge readiness. No second broad fix wave or waiver is implied.

| Owner | Final reviewed local revision |
|---|---|
| contracts | `85baf86e574276fcd036e53e23641af6aad602f9` |
| broker | `fd05cb48b8508e7939f9cdf9df275742a06fc4f8` |
| mediator | `6c3d96b4763871b9addc9bc7223e71ee7d38abd9` |
| worker | `b0d43ec2b5b0c8da035d4ccff754545132b978d4` |
| ui | `e51d67e9e1a986601df6b5e1acf68aaf7ae0870d` |
| platform | `637a69279f0fe5019560b1e54d28f48c1c715897` |

All six reviewed worktrees were clean when this checkpoint was prepared.
Runtime is committed only on isolated local branches; originals retain their
runtime/scaffold and unrelated edits. Contracts v1.0.0 and dependency lockfiles
remain unchanged.

Controller final verification on platform revision above: `node
scripts/smoke-codex-conversation.mjs --fake` (session85023) exit0, six original
crash/commit boundaries, 11 fake native turns, exact baseline/delta/source IDs,
targeted/cross-room exclusion, duplicate=one logical turn, worker restart and
fresh reset, shared omitted settings, rejected-completion recovery and exact
execution_failed/session_unavailable/runtime_unavailable projections. `node
--test scripts/tests/codex-conversation-smoke.test.mjs` (session23304) exit0,
13 passed, zero failed. This is synthetic native/identity/private-DNS adapter
coverage, not whole packaged Compose, real Keycloak/browser or provider proof.

Inspected owner logs and independent review record broker150 passed + one
pre-existing ignored live test; mediator65 passed, zero failed/ignored (correcting
the earlier reported68); worker46; UI121 + pin/tamper checks and production
build. Owner fixture tests3, actual assertion-failure/SIGTERM/SIGINT cleanup3,
package/startup checks10 passed. Earlier contracts/regression/legal/pin evidence
is retained with original attribution, not presented as rerun here.

New ARM64 worker image:
`sha256:2d8bfade27802f910cf68e832722c93b4a2acc2addb825711e1223617a4cd385`.
Compiled runtime revision `b418a76e0e7ca047b5fe995eb17519aced369a06`; worker
head above adds evidence documentation. Immutable image readiness checks used
network-none/read-only/cap-drop-all, both admission markers; default invocation
refused as expected. Actual native 44-setting/eight-model/resume/six unsolicited
tool refusal evidence remains attributed to its earlier source/image, not this
new image. CLI/catalog/control hashes are unchanged. x86_64 native admission and
Rust1.85 minimum-version checks remain unrun.

Default discovery: pending specification approval. The local proposed How is
`thought-khoral-codex-agent/.ai/specs/how/default-settings-discovery-proposal.md`.
It proposes a read-only authenticated defaults query in a new immutable v1.1.0
artifact and independent mixed-capability controls, covering absent conversation
and explicit New/reset. It authorizes no runtime or published contract changes.
Live provider verification: pending. Account/model availability, actual native
history, live tool/egress/key isolation, packaged deployment/private DNS and real
browser/identity evidence remain separately gated. No merge, push, publication,
service activation or provider inference occurred.

Independent review artifacts are retained outside Git at
`/private/tmp/codex-conversation-task9/final-fix-review.md`,
`final-implementation-review.md`, and `task9-fix-review.md`; owner evidence at
`/private/tmp/Task9-final-fix-evidence/`. Final root/documentation/source-reference
and identity gate results will be recorded in the controller checkpoint after
these source-derived record updates. The aggregate release checklist remains
unchecked; passing synthetic checks do not resolve F1 or default discovery.

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
No release publication, shipped interoperability, merge, push, provider use or
service activation is authorized. Whole milestone/Task9 acceptance remains open.

## Defaults discovery local synthetic checkpoint — 2026-10-07

All four defaults-amendment tasks passed their independent reviews. The final
whole-branch review passed. F1 (initial/New reasoning-only settings deadlock) and
visible defaults discovery are accepted for this local synthetic candidate.

| Source | Exact local revision | Retained worktree |
| --- | --- | --- |
| contracts | `1ea828f28725ddaaefa21d083473f9abbd777975` | `/private/tmp/codex-conversation-defaults/contracts` |
| broker | `2e7d23b467c572819f498c3b9bf14d74a62dc821` | `/private/tmp/codex-conversation-defaults/room-gateway` |
| mediator | `6c3d96b4763871b9addc9bc7223e71ee7d38abd9` | `/private/tmp/codex-conversation-final-fix/agent-gateway` |
| worker | `b0d43ec2b5b0c8da035d4ccff754545132b978d4` | `/private/tmp/codex-conversation-final-fix/worker` |
| ui | `79e5e7310a450efea561548cd87871446c1939aa` | `/private/tmp/codex-conversation-defaults/workspace-ui` |
| platform | `2d856773078a7caa542d719e539b55a5ab2dafaa` | `/private/tmp/codex-conversation-defaults/platform` |

The composed run was executed at `f9afeb20b746200daa9cdef0406c03f88a422b68`.
The subsequent path-provenance correction was tested and scoped-reviewed at
`220f6e0c74a29c000d7de81c0cb77823de0bd15c`;
the final platform revision above adds completion metadata only. The original
repositories retain their runtime; local implementation branches remain unmerged.

The unreleased candidate contract source is
`1ea828f28725ddaaefa21d083473f9abbd777975`, proposed release
`thought-khoral-agent-conversation-v1.1.0`. Its archive SHA-256 is
`fab59a486f6498b843467202debcb0768403bd57ba7dda41be2a01e5f23fdda8`
and externally anchored lock SHA-256 is
`7914d32eae2487879a68405b5095a6b9aa91355f87529c43f4055844821902a9`.
All 156 candidate payload files match in broker/UI; published v1.0 bytes remain
unchanged. The profile and API namespace stay v1. This is not a published release.

Evidence: retained 125 contract fixtures plus 16 additive cases and 6 candidate
integrity tests; broker serial suite 158 passed with 1 existing live-only test
ignored; UI full suite 190 passed with 1 intentional composed skip, followed by
scoped harness/TypeScript checks; 26 source-pin and 13 retained runner guards;
5 fixture unit tests; 3 actual failure/SIGTERM/SIGINT cleanup cases. The composed
candidate passed 12 capability/lifecycle cases, 3 display/send mutation cases,
15 actual HTTP UI children with 90 test passes, and 24 synthetic native turns
(11 retained baseline plus 13 added). Stale/removed pairs allocate no task/event
or native turn, retain the prompt and require explicit Refresh. Default-only
changes preserve the displayed explicit pair; unavailable replay is immutable.
Paused-publisher regressions verified RED before and GREEN after atomic exclusive
handshake publication. Owned processes, containers and staging files were cleaned.

The current composed state is `/var/folders/70/5kxy5kys3bj0252chp3ck8900000gn/T/Task9-codex-conversation-j9py6w`. Full provenance,
task/thread bindings, raw log references, limitations and review reports remain in
`/private/tmp/codex-conversation-defaults/defaults-reviewed-checkpoint.json` and
`/private/tmp/codex-conversation-defaults/task-4-logs/`. Root hierarchy/reference/
identity, scaffold documentation and whitespace results are recorded separately
in `/private/tmp/codex-conversation-defaults/final-gates.json` after synchronization.
The old parallel broker fixture port collision and Vite chunk advisory are
retained limitations; no passing parallel broker-suite claim is made.

Publication: pending

Packaged-stack verification: pending

Live provider verification: pending

The composed gate uses jsdom, a synthetic room socket, actual conversation HTTP
and storage, and a fake native executable. It does not establish packaged Compose,
real browser/Keycloak, provider, architecture-minimum or new-image acceptance.
Task 9 and milestone aggregate gates remain open. Specification/memory-guided
working directories remain the separately scoped future extension.

## Defaults-discovery amendment status — 2026-10-07

The approved defaults amendment and F1 correction were independently reviewed
on the local platform candidate branch `codex-defaults-verification` at
`2d856773078a7caa542d719e539b55a5ab2dafaa`. It records the composed synthetic
candidate run and exact source pins; it is not a packaged Compose or live
provider result. The platform default branch retains its prior runtime.
Publication, packaged-stack and separately authorized live-provider gates
remain pending. See the [coordinated checkpoint](https://github.com/thoughtkhoral/thought-khoral/blob/main/.ai/specs/how/codex-room-conversations-implementation-plan.md).
