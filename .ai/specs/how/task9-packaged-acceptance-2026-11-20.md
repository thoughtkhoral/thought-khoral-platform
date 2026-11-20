# Task 9 packaged acceptance preparation plan

> **For agentic workers:** Execute this plan inline using
> superpowers:executing-plans after specification approval. Steps use checkbox
> syntax for tracking; no parallel delegation is required.

**Status:** approved for local implementation on 2026-11-20 by the maintainer's
“approve”, after review of this plan and the root procedure. Publication,
activation and provider calls retain their separate gates.

**Goal:** Verify the current defaults-capable Codex integration in the real
packaged room stack and browser with exact, reproducible acceptance inputs.

**Architecture:** Preserve the historical provider-free and v1.0 live modes.
Add an explicitly selected packaged-v1.1 acceptance profile after contract
release and consumer transition. Keep operator credentials and private evidence
outside Git; use the existing actual worker, proxy and namespace boundary.

**Tech Stack:** Existing Node test runner, Rust component suites, rootless
Podman/Compose, Keycloak PKCE, React UI and an available real-browser controller.

**Spec:** [root acceptance procedure](https://github.com/thoughtkhoral/thought-khoral/blob/main/.ai/specs/how/task9-packaged-acceptance-2026-11-20.md),
[local service What](../what/codex-local-service.md),
[local service How](codex-local-service.md), and
[approved defaults plan](default-settings-discovery-implementation-plan.md).

## Global constraints

- “Publication still needs its separate authorization.”
- “This local merge does not authorize release publication, shipped interoperability, push, provider use or service activation.”
- “Existing v1.0.0 clients remain compatible with the additive route; existing schemas and fixture bytes remain unchanged.”
- Profile remains `thought-khoral.agent-conversation.v1`; API remains `/api/agent-conversations/v1`.
- CLI remains `0.160.0`; worker minimum Rust remains 1.85.0; production builder is Rust 1.93.1.
- Keep `--fake`, `--fake --defaults-candidate`, their exact source/artifact pins and candidate-plus-live refusal unchanged.
- Keep private-path/symlink/linked-worktree guards, separate human identities, explicit live opt-in and exclusive evidence creation.
- Preserve namespace/egress/tool isolation, distinct recipient-scoped credentials and the existing external database volume.
- Preserve others' edits. Record failures and missing coverage without treating them as acceptance.

## Preparation findings

The root procedure freezes all eight observed preparation commits and retained
native worker package/proof identities. Platform preparation base is
`fbd2d525e62a7d5e58948f91570095c6821667d4`.

The existing runner SHA-256 is
`7980f38b164d6622b162f455d32f6c7d524ea72c11b3dffe68ee47054e58c90a`;
`scripts/fixtures/codex-conversation/defaults-candidate-pins.json` SHA-256 is
`b63e6371f0c349052a627b51c819e735a00f184978345ed9217c9d9074b437a9`.
These are preparation fingerprints, not hashes of a future modified runner.
Live mode currently requires old worker image
`sha256:2d8bfade27802f910cf68e832722c93b4a2acc2addb825711e1223617a4cd385`
and old broker/mediator/worker/UI commits. Replacing those constants with current
HEADs would discard the historical run definition and still leave contract
publication and package provenance unresolved.

## Task A — Release and consumer prerequisite

**Owners:** contracts owns artifact preparation/publication; broker/UI own
release consumption; root coordinates review. Platform consumes their results.

- [x] Review and approve this amendment before changing executable verification.
- [ ] Contracts prepares its owning release How and exact v1.1 release archive,
  lock, payload comparison and retained-v1.0 compatibility results. Review these
  concrete inputs before requesting tag/release publication. Its current
  candidate builder intentionally emits unreleased metadata; no renamed
  candidate lock is accepted as a release.
- [ ] After explicit publication approval, verify the downloaded published
  archive and lock against the reviewed bytes. Record full tag target and
  release asset digests; absence leaves Task A open.
- [ ] Broker/UI prepare and review their owning specification changes, adopt the
  released artifact provenance and run their existing pin/fixture suites.
  Preserve the historical local candidate vendors and fixture mode. Record
  exact final consumer commits and published-v1.0 compatibility.

**Exit:** actual released artifact and verified consumer commits exist. Current
candidate fingerprints cannot satisfy this exit. This plan supplies no new
contract format or automatic publication authority.

## Task B — Add a separately reviewed acceptance profile

**Proposed files owned by platform:**

| Path | Responsibility |
| --- | --- |
| `scripts/fixtures/codex-conversation/packaged-v1.1-pins.json` | Exact final source/contract/package/architecture inputs; created only after Task A and actual image preparation |
| `scripts/smoke-codex-conversation.mjs` | Explicit profile selection and preflight; historical modes unchanged |
| `scripts/tests/codex-packaged-pins.test.mjs` | Offline profile/identity refusal and success tests; no provider or engine |
| `scripts/fixtures/codex-conversation/live-operator-packet.packaged.example.json` | Non-secret example matching the new profile, with approvals false |
| `docs/codex-verification.md`, `docs/codex-local.md` | Derived commands and evidence interpretation |

**Interface:** proposed CLI selection is
`--live --allow-live --acceptance-profile packaged-v1.1`. Selection without
`--live`, unknown/repeated profile selection, or any combination with
`--defaults-candidate` is rejected before credential reads or engine calls.
Default `--live --allow-live` retains its exact historical v1.0 interpretation.

The checked-in profile has closed fields: `id`, `profileVersion`, `cliVersion`,
`platform`, `revisions`, `contract`, and `worker`. `id` is `packaged-v1.1`;
`platform` initially is `linux/arm64`; `revisions` has full commits for contracts,
broker, mediator, worker, UI and memory engine. `contract` contains `tag`,
`commit`, `archiveSha256`, `lockSha256`; `worker` contains `imageId`,
`cargoLockSha256`, `cliSha256`, `proofSha256`. Release hashes and the image ID
are measured outputs of Task A/image preparation, not operator-supplied values.
The platform runner's exact clean commit is resolved from its own checkout and
required in the packet, avoiding a self-referential platform commit inside its
own committed profile. Root's coordination commit is recorded in the run summary.

- [ ] Before implementation, add offline rejection cases for wrong revision,
  unknown/missing fields, wrong platform/image/CLI/proof, unreleased contract,
  candidate/live combination and operator/profile mismatch. Model external
  commands through synthetic executables recording contact; each invalid
  configuration must produce no engine/network contact and no evidence file.
- [ ] Run the new test file and record the expected failures for missing profile
  behavior. Then implement closed profile parsing, exact selection and identity
  checks. Do not accept arbitrary JSON paths or a packet's self-declared inputs
  as review authority.
- [ ] Include a positive synthetic case proving the exact selected profile
  reaches the mocked engine boundary; refusal-only tests cannot prove the new
  profile is usable. Preserve existing private-file and interactive-terminal
  checks and record actual clean source/image identity at run time.
- [ ] Record engine-inspected immutable image, OS/architecture, package-admission
  result and actual CLI/proof/lock identity. Use a newly measured local image ID
  if CI-local images are unavailable. A version string alone cannot pass.
- [ ] Run the new test, existing live guard test and candidate-pin test with
  Node's test runner. Tests must show the historical profile still accepts its
  exact prior packet and the candidate mode still refuses live execution.
- [ ] Review the exact diff and commit specifications before implementation in
  each owner. Record final profile/runner commits; source changes after review
  require renewed identity review and affected verification.

**Exit:** a reviewed exact profile and usable guarded runner; no service
activation or provider execution is implied by offline success.

## Task C — Package and browser preflight

- [ ] Select clean exact source checkouts from Task B; verify input/source locks
  and build actual production images serially. Inspect `linux/arm64` engine and
  image architecture and retain image IDs and build inputs. Run actual package
  admission with `python3 scripts/tests/codex-package.py --image` followed by
  the measured immutable ID and `--expect-verified`; record its exit status.
  Run the worker's existing actual-native policy capture for a newly built
  package; bind its CLI/proof/lock hashes to the profile before final review.
- [ ] Verify the three distinct private credential files and prepared state
  ownership through the existing wrapper. With configured private paths run
  `sh scripts/codex-compose.sh config`; retain only redacted output. This
  command requires configuration even though it does not start services.
- [ ] Confirm free ports, adequate VM resources, the preserved external database
  volume, and access to a real browser controller. Never alter browser security
  settings to make automation work. If unavailable, record browser coverage as
  not run and keep its gate open.
- [ ] Prepare a dedicated test room, two distinct human identities and private
  new operator/evidence paths. Keep approvals false until actual approval.
  Browser testing requiring a model turn is part of the provider run.

**Exit:** concrete reviewed images/configuration/operator packet and accessible
browser. Request activation/provider approval against these exact inputs.

## Task D — Execute and retain acceptance evidence

Requires the separate activation approval and, for model turns, provider-call
approval. Tool-probe prompts also require their approved operator scope.

- [ ] Activate through the existing opt-in wrapper, verify exact image IDs,
  readiness, authenticated catalog/defaults and private DNS. Run deterministic
  regressions without simultaneous image builds. Compare base and opt-in
  behavior; neither may bypass existing authority or isolation checks.
- [ ] Execute root scenarios P1 and B1–B4 using actual browser controls and two
  independent sessions. Retain redacted screenshots and event/task correlation.
  Record the actual browser version; Node HTTP/WebSocket and jsdom are separate
  evidence classes. Exercise the deployed admission's capabilities. Keep the
  four-capability synthetic matrix separately attributed; do not add production
  capability switches just to expand browser coverage.
- [ ] Execute L1–L3 and I1 through the new selected live profile using the private
  `TASK9_LIVE_OPERATOR_PACKET`, then inspect native receipts and runtime/network
  evidence. One missing inaccessible model is missing coverage, not account
  denial or a successful live gate. Never use answer text alone as proof of
  history delivery, thread identity or tool isolation.
- [ ] Execute I2 only against owned test resources; verify owner/proxy/policy
  fail-stop and admission expiry. Recover by recreating owner plus dependents,
  then verify readiness again. A failed isolation check disables Codex.
- [ ] Run R1: applicable contracts, broker, mediator, worker, UI and platform
  checks; root hierarchy/reference/identity and organization link checks;
  artifact/legal/session-exclusion checks. Record each command and actual exit
  code at selected sources. Reuse unchanged native CI evidence only within its
  tested-source scope and label it separately from this composed run.
- [ ] Complete C1 with owned-resource cleanup and database-volume preservation.
  Redact evidence and inspect the diff before committing the public summary.
  Record hashes of raw private evidence without exposing tokens, cookies,
  provider keys, native sessions or private room content.

**Exit:** per-scenario evidence and explicit gaps. Root reviews the original
Task 9 checklist and P1–C1 before acceptance. Product release, image publication
and production deployment remain later decisions.

## Preparation validation

This plan changes Markdown only. Validate local links across all seven child
repositories, specification hierarchy, repository references, identity and
whitespace. Run the existing provider-free live-guard tests to confirm the
observed procedure still fails closed. Do not run container fixtures, builds,
browser activation or provider checks merely to validate this plan.

## Local execution checkpoint — 2026-11-20

Task A approval is recorded above. Contracts preparation now exists on isolated
branch `codex/task9-contract-release-20261120` at source `65bdc00`, following
owning specification commits. Full worktree and extracted-archive suites pass;
two independently prepared TAR/lock outputs match; the 156 candidate payload
and 135 retained schema/fixture hashes match independently reviewed source.
The [root execution report](https://github.com/thoughtkhoral/thought-khoral/blob/main/docs/task9-release-preparation-2026-11-20.md)
retains exact local identities and evidence limits.

The required online audit blocks publication on the existing fast-uri 3.1.8
development dependency. A bounded 3.1.9 patch is tested outside the release
worktree with a clean audit; approval of that dependency exception is pending.
Task A's release/consumer exit is not satisfied. Task B's concrete profile and
Tasks C/D remain unexecuted; no candidate was enabled for live use.

### Approved dependency correction checkpoint

The maintainer approved the exact 3.1.9 exception, recorded before implementation
source `f1c8621810c8c121c1aafc8e5a342d6626c4b843` in the isolated contracts
branch. Full worktree/extracted tests and both online audits now pass; repeated
corrected TAR/lock outputs match, with all 156 candidate payload and 135 retained
schema/fixture hashes preserved. The current root report names all asset hashes.
This closes the local audit blocker, superseding the preceding pending-exception
status. Task A's publication and verified consumer transition remain open; no
new live profile or service/provider activation is authorized by this correction.
