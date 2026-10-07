# ThoughtKhoral platform specifications

Parent requirements in the ThoughtKhoral root `.ai/specs/` apply here. This project may diverge only through an accepted local decision record that identifies the overridden parent rule and its consequences.

See the [root specification index](https://github.com/thoughtkhoral/thought-khoral/blob/main/.ai/specs/README.md).

For the coordinated Codex conversation status across all projects, see the [shared status guide](https://github.com/thoughtkhoral/thought-khoral/blob/main/docs/codex-conversation-status.md).

## Local areas

- [What: local MVP platform](what/local-mvp.md)
- [What: public documentation](what/public-documentation.md)
- [How: implementation](how/implementation.md)
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

## Current POC publication checkpoint — 2026-10-07

The reviewed opt-in Codex platform candidate is pushed to GitHub `main` at
`251f10f8ad975b6035640adc4a03bbdd34ef4e36`. Candidate pin/safety tests and the
provider-free composed smoke passed. Packaged deployment and separately
authorized live-provider verification remain open; this experimental POC is not
claimed production-ready.
