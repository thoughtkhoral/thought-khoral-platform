# ThoughtKhoral platform specifications

## Current workspace state — 2026-11-20

The exact packaged-v1.1 acceptance profile is implemented and reviewed at local source `996e1887234b8958a315279aa3d5aa194a7fad52`. Profile SHA-256 is `7106f7e41a6bc13feab8a0b51fccde9b47fefca2d63980d551ed929eac1c1a71`. Forty-six profile/historical-guard tests and actual credential-free image/source preflight passed. The verified local ARM64 image passed ten package/startup probes and byte-identical native capture; this is not full packaged-stack/private-DNS/browser/live acceptance.

This checkout contains the reviewed released-v1.1 implementation. The
maintainer authorized coordinated integration on 2026-11-23. Frozen component
acceptance commits remain unchanged; packaged/browser/live gates remain open.

- [How: packaged profile validation](how/packaged-profile-validation.md)

See the [root workspace authority](https://github.com/thoughtkhoral/thought-khoral/blob/main/.ai/specs/how/workspace-state.md) for exact source/release
identities, preserved source snapshots and remaining gates. Documentation-only
HEADs are not substitutions for frozen acceptance commits. Earlier dated
checkpoints retain their historical evidence; no runtime suite was rerun for
this prose refresh.

Parent requirements in the ThoughtKhoral root `.ai/specs/` apply here. This project may diverge only through an accepted local decision record that identifies the overridden parent rule and its consequences.

See the [root specification index](https://github.com/thoughtkhoral/thought-khoral/blob/main/.ai/specs/README.md).

For the coordinated Codex conversation status across all projects, see the [shared status guide](https://github.com/thoughtkhoral/thought-khoral/blob/main/docs/codex-conversation-status.md).

<a id="current-verification-scope--2026-11-19"></a>

## Historical verification scope — 2026-11-19

See the [owning current checkpoint](how/codex-local-service.md#current-maintenance-and-verification-scope--2026-11-19) for source publication and
verification attribution. The integrated Codex candidate is pushed on `main`;
F1/defaults discovery are accepted within their composed synthetic scope.
Final-lock native Linux worker acceptance is recorded in the coordinated plan,
but Task 9 opt-in packaged room-stack, rendered-browser/Keycloak, live-provider,
v1.1 contract publication and production/release gates remain open.

The dated entries below preserve the sequence of local review, integration and
publication. Local-only/unmerged wording applies to those historical checkpoints.

## Local areas

- [Approved How: packaged v1.1 profile validation](how/packaged-profile-validation.md) — exact source/package preflight; no activation/provider authority.
- [Approved Task 9 packaged acceptance preparation plan — 2026-11-20](how/task9-packaged-acceptance-2026-11-20.md) — local implementation authorized; activation/provider gates retained.
- [What: local MVP platform](what/local-mvp.md)
- [What: public documentation](what/public-documentation.md)
- [How: implementation](how/implementation.md)
- [Approved How: complete container build inputs](how/package-build-inputs-proposal.md)
- [Decisions](decisions/README.md)
- [Decision 002: ThoughtKhoral project identity](decisions/002-thoughtkhoral-identity.md)

## Room lifecycle ownership

The platform owns the non-joining browser bootstrap and non-secret room URL
binding. The UI owns explicit Enter/Leave presentation and socket lifecycle;
the gateway remains unchanged and exposes no `room.leave` RPC. See the
[workspace UI lifecycle specification](https://github.com/thoughtkhoral/thought-khoral-workspace-ui/blob/main/.ai/specs/what/mvp-ui.md)
and [gateway lifecycle boundary](https://github.com/thoughtkhoral/thought-khoral-room-gateway/blob/main/.ai/specs/what/mvp-room.md).

## Approved Codex room-participation extension

- [What: codex local service](what/codex-local-service.md)
- [How: codex local service](how/codex-local-service.md)
- [Coordinated implementation plan](https://github.com/thoughtkhoral/thought-khoral/blob/main/.ai/specs/how/codex-room-conversations-implementation-plan.md)

Approved by the maintainer on 2026-10-05 under [issue 1](https://github.com/thoughtkhoral/thought-khoral-platform/issues/1).
Implementation follows the coordinated plan and its artifact/dependency gates.
Existing runtime behavior is unchanged until the relevant tasks pass verification.

Task 8 opt-in Codex packaging is independently reviewed and committed locally at
`9626bc46f8b74c2a58b2578c4f744996f3d41317` on `codex-opt-in-platform`. The original checkout retains its prior
runtime. The [local checkpoint](how/codex-local-service.md) records scope; full-stack
and separately authorized live verification remain Task 9.

## Task 9 synthetic verification and correction checkpoint — 2026-10-07

Local corrections and synthetic evidence are recorded in the [owning checkpoint](how/codex-local-service.md). The approved defaults amendment and F1 are accepted on reviewed local synthetic candidate branches. Contract publication, packaged-stack and separately authorized provider/live evidence remain pending. Original runtime is retained; local corrected branches are unmerged.

## Approved defaults-discovery amendment — 2026-10-07

The [approved design](https://github.com/thoughtkhoral/thought-khoral-codex-agent/blob/main/.ai/specs/how/default-settings-discovery-proposal.md) authorizes local defaults discovery and
independent optional controls, with verified unreleased candidate contract pins.
Implementation and synthetic verification follow the amendment plan; publication,
provider use, activation, merge and push retain their separate gates.

Approved defaults extension execution plan: [four coordinated local tasks](https://github.com/thoughtkhoral/thought-khoral/blob/main/.ai/specs/how/codex-default-settings-implementation-plan.md). The prior Task9 checkpoint remains open until amendment acceptance and its separate packaged/live gates.

## Local main integration checkpoint — 2026-10-07

After explicit user authorization, the reviewed platform source at
`2d856773078a7caa542d719e539b55a5ab2dafaa` was merged into local `main`.
The v1.1 contract remains unreleased. The owning How records the candidate pin
checks and provider-free composed smoke. Publication, deployment activation,
provider use, and push remain separate gates.

## POC publication checkpoint — 2026-10-07

The reviewed opt-in Codex platform candidate is pushed to GitHub `main` at
`251f10f8ad975b6035640adc4a03bbdd34ef4e36`. Candidate pin/safety tests and the
provider-free composed smoke passed. Packaged deployment and separately
authorized live-provider verification remain open; this experimental POC is not
claimed production-ready.

## Approved build-input maintenance — 2026-11-19

The maintainer approved the [bounded build-input correction](how/package-build-inputs-proposal.md)
before implementation. It supplies existing contracts to the gateway and
reference-agent build stages and existing contracts/scripts to the UI stage.
It implements the existing [local MVP composition/start criteria](what/local-mvp.md)
without changing product behavior, protocols, identity, isolation, or storage.
The main [implementation design](how/implementation.md) incorporates this
amendment. The correction was subsequently pushed at `74ac640`; its owning
How records the publication checkpoint separately from the earlier local evidence.

All three image builds and the default deterministic-stack smoke passed.
The containers were subsequently stopped and removed at the user's request;
the database volume was preserved. This does not close opt-in Codex Task 9,
live-provider, rendered-browser, or production acceptance gates.
