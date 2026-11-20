# Platform documentation requirements

## Status and authority

Approved documentation scope, refreshed by the maintainer's 2026-11-20 request
to update documentation specifications before derived pages at workspace and
individual-project level. This supersedes stale scaffold/MVP-only documentation
criteria, without changing approved runtime requirements or acceptance gates.

Read the [local specification index](../README.md), [component requirements](codex-local-service.md)
and [implementation design](../how/codex-local-service.md) first. Root
[documentation requirements](https://github.com/thoughtkhoral/thought-khoral/blob/main/.ai/specs/what/open-source-documentation.md)
and [current workspace authority](https://github.com/thoughtkhoral/thought-khoral/blob/main/.ai/specs/how/workspace-state.md)
apply. Documentation cannot approve implementation, publication or activation.

## Maintained documentation set

| Page | Required role |
| --- | --- |
| [Repository README](../../../README.md) | Purpose, maturity, prerequisites, valid commands, compatibility and source links. |
| [Project documentation index](../../../docs/README.md) | Local documentation entrypoint mapping maintained pages to their governing specs and checkout applicability. |
| [Local operation and isolation](../../../docs/codex-local.md) | Opt-in packaging, private state/credentials, admission and network boundary. |
| [Conversation verification](../../../docs/codex-verification.md) | Historical fake/candidate modes, prepared profile applicability and gated live procedure. |

## Acceptance criteria

- Update applicable documentation specs and accepted evidence first, then
  maintain these authored Markdown pages. CI validates; it does not generate
  prose or rewrite specifications.
- Map each maintained page to its governing local What/How, and link from the
  repository README to this documentation specification and the project index.
- Identify canonical runtime versus prepared-branch applicability. Documentation
  HEADs, published artifacts and frozen runtime acceptance inputs are distinct;
  do not repin a tested source because a prose-only followup advanced HEAD.
- Document sibling source layout, prerequisites, startup/verification/cleanup and preservation of the external database volume.
- Separate the deterministic default stack from opt-in Codex packaging and the locally prepared released-v1.1 profile; do not advertise unavailable canonical-runner flags.
- Describe recipient-scoped secrets/state ownership, admission, proxy/kernel isolation and Compose fail-stop/recreate behavior separately from Kubernetes init/NetworkPolicy.
- Label component, package, fake-native, real-browser and live-provider evidence separately; activation/provider approvals and configured private operator inputs remain required.
- Preserve historical evidence and existing anchors. Attribute results to exact
  source/image/contract inputs and distinguish implemented, verified, published,
  enabled and production-ready states.
- Keep private configuration, provider keys, human tokens, native sessions and
  room contents outside docs and fixtures. Preserve licenses and legal notices.
- Direct contributions through the issue/specification workflow and private
  security-reporting policy. Validate links, source references and whitespace;
  run any additional checks required by this project's agent instructions.

## Verification boundary

Documentation checks establish discoverability and consistency, not runtime,
network, browser or provider acceptance. Fresh verification must name what was
actually run; retained results keep their original source identity. Current
state comes from approved component/root evidence, not the age of this file.
