# Platform documentation

Latest operational checkpoint — 2026-11-25: all eleven corrected images and
authenticated model/default reads passed on internal-only Codex networking.
The bounded eleven-minute observation remains unverified. Codex and mediator
are off; seven base services are healthy, with no conversation allocations or
model/provider calls. The [owning execution checkpoint](../.ai/specs/how/task9-runtime-compatibility.md)
distinguishes these subchecks from full acceptance. A new observation window
requires separate approval; the original admission expiry was not renewed.

This index is maintained from the [documentation requirements](../.ai/specs/what/public-documentation.md).
Use the [repository README](../README.md) for prerequisites, valid commands and
checkout context; use the [specification index](../.ai/specs/README.md) for
requirements and evidence authority. The page/source map below follows the
current documentation contract, not a new runtime or release decision.

## Checkout applicability

Separately approved existing-database maintenance completed on 2026-11-25;
the actual database now passes schema readiness and base availability is restored.
See the [maintenance verification](codex-verification.md#existing-database-readiness--2026-11-25).
This does not authorize starting the corrected Codex stack or publishing sources.

This isolated local branch includes the approved loopback teardown correction
and separately prepared compatibility profile. See the
[loopback verification checkpoint](codex-verification.md#local-loopback-health-verification--2026-11-24)
and [owning design](../.ai/specs/how/task9-runtime-compatibility.md).
The exact packaged image passed 660.005 seconds and proxy-loss fail-stop. Existing-data
upgrades, publication and changed-input activation remain gated; older
integration text below describes its own earlier checkpoint.

This checkout contains the reviewed released-v1.1 implementation. The
maintainer authorized coordinated integration on 2026-11-23. Frozen component
acceptance commits remain unchanged; packaged/browser/live gates remain open.

The [workspace guide](https://github.com/thoughtkhoral/thought-khoral/blob/main/docs/workspace-status.md)
identifies canonical mains, prepared branches, published artifacts and exact
preserved inputs. Documentation-only HEADs do not replace frozen acceptance
commits or enable a prepared command in a checkout that lacks its implementation.

## Maintained pages and sources

| Page | Read it for | Governing specification |
| --- | --- | --- |
| [Repository README](../README.md) | Purpose, maturity, prerequisites, valid commands, compatibility and source links. | [Source](../.ai/specs/what/public-documentation.md) |
| [Local operation and isolation](codex-local.md) | Opt-in packaging, private state/credentials, admission and network boundary. | [Source](../.ai/specs/how/codex-local-service.md) |
| [Conversation verification](codex-verification.md) | Historical fake/candidate modes, prepared profile applicability, schema maintenance and gated live procedure. | [Source](../.ai/specs/how/codex-local-service.md), [compatibility/maintenance](../.ai/specs/how/task9-runtime-compatibility.md) |

## Authoring and verification

1. Read the [component requirements](../.ai/specs/what/codex-local-service.md) and
   [implementation design](../.ai/specs/how/codex-local-service.md). Update applicable specs and
   accepted evidence before changing their documentation.
2. Update the README, this index and affected guides with checkout-specific
   applicability. Keep historical anchors and exact source/image/contract IDs.
3. Validate local/source links and whitespace, and follow any project agent
   instructions for additional checks. Documentation validation does not prove
   runtime, real-browser, network, provider or production acceptance.

These are authored Markdown pages. CI validates their structure/links and does
not generate prose or rewrite specifications. Keep credentials, private state
and user content out of documentation; preserve licensing and legal notices.
