# ThoughtKhoral platform specifications

Parent requirements in the ThoughtKhoral root `.ai/specs/` apply here. This project may diverge only through an accepted local decision record that identifies the overridden parent rule and its consequences.

See the [root specification index](https://github.com/thoughtkhoral/thought-khoral/blob/main/.ai/specs/README.md).

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

## Approved defaults-discovery amendment — 2026-10-07

The [approved design](https://github.com/thoughtkhoral/thought-khoral-codex-agent/blob/main/.ai/specs/how/default-settings-discovery-proposal.md) authorizes local defaults discovery and
independent optional controls, with verified unreleased candidate contract pins.
Implementation and synthetic verification follow the amendment plan; publication,
provider use, activation, merge and push retain their separate gates.

[Defaults discovery implementation plan](how/default-settings-discovery-implementation-plan.md) executes the approved local amendment.
