# Packaged v1.1 acceptance-profile validation

Status: local implementation authorized by the maintainer's 2026-11-20
“proceed” with released broker/UI consumption and acceptance-profile preparation,
continuing the [approved Task 9 plan](task9-packaged-acceptance-2026-11-20.md).
Activation and provider calls remain separate. Historical fake, candidate and
v1.0 live inputs remain unchanged.

## Files and input authority

Add `scripts/lib/codex-packaged-profile.mjs` for closed profile/packet checks,
clean source verification and actual worker identity/package evidence. Keep
`scripts/smoke-codex-conversation.mjs` responsible for the existing live workflow.
Tests in `scripts/tests/codex-packaged-pins.test.mjs` use synthetic process
observations; they never activate a stack or contact a provider.

After consumer review and actual final-lock worker package verification, create
`scripts/fixtures/codex-conversation/packaged-v1.1-pins.json` with the closed
fields already defined by the Task 9 plan. Bind its exact bytes to a source
SHA-256 literal in the module. Operators cannot supply another profile path or
change authoritative inputs through their packet. No placeholder profile or
fabricated image ID is permitted before actual verification completes.

Selection is only `--live --allow-live --acceptance-profile packaged-v1.1`.
Reject unknown/repeated/malformed profile selection, missing live opt-in and
candidate combinations before credential reads, engine access or output writes.
The default v1.0 live path retains its exact old interpretation.

## Closed metadata and clean sources

Require profile id packaged-v1.1, platform linux/arm64 and CLI 0.160.0; six full
source commits for contracts/broker/mediator/worker/UI/memory; published release
tag/source/archive/lock hashes; and immutable worker image ID, Cargo lock, CLI
and native proof hashes. Fix release provenance to the freshly downloaded
v1.1 artifact and retain the approved ARM64 CLI/proof/final-lock identities.
Reject unknown or missing fields, arrays in place of objects, malformed hashes
and zero identities. Require contracts revision equal to contract source.

The operator packet retains its existing fields and additionally carries
`acceptanceProfile`, `platform`, `contract` and `worker`, matching the reviewed
profile exactly. Existing contractTag, cliVersion and workerImageDigest must
agree too. Its revisions equal all six reviewed commits plus the actual platform
runner commit. Check the platform and six source checkouts are exact and clean
with Git before reading tokens or accessing the engine. TASK9_*_REPO variables
may select paths only; default sibling repository paths retain the normal
workspace layout. A detached clean contracts checkout at the release tag is
valid; a later evidence commit is not silently accepted as the frozen source.

## Real worker checks and evidence

Before broker requests, inspect the activated worker's immutable image and the
image's Linux/ARM64 identity. Require reviewed source and Cargo-lock labels
`org.thoughtkhoral.source` and `org.thoughtkhoral.cargo-lock`, installed by the
provider-free build after measuring the clean source and lock. Labels are not
independent trust: the reviewed immutable image ID and retained build provenance
bind them. Execute the existing worker --verify-package path, requiring its
exact tool-policy and package success markers. Measure actual CLI and ARM64
proof file SHA-256 values inside the worker and require exact profile matches.
Do not infer package acceptance from a version string, label or image name.

Return only closed safe image/platform/CLI/proof/lock identity evidence. Fail
with generic diagnostics on engine/worker errors; do not retain raw private
process output. Existing private file, distinct human, loopback origin, terminal
and exclusive new evidence-file guards remain in force. Include reviewed
profile SHA-256 and safe package/source observations in live evidence. The
worker image can be built and synthetically tested during preparation; no live
stack is activated to test this module.

## Required offline verification

Observe failures before implementation for missing profile behavior. Cover
unknown/extra/missing fields, altered release/worker provenance, wrong packet
identities, dirty/wrong source, wrong image/architecture/labels, package refusal,
CLI/proof tampering and generic engine failures. Include a positive synthetic
preflight that consumes exact command observations and returns verified identity
evidence; a refusal-only suite is insufficient. Actual CLI tests with guarded
synthetic PATH commands must prove invalid selections/configurations perform no
engine/network contact and create no output. Preserve the existing live guard
and candidate-pin suites. Actual package build/admission/capture evidence remains
separate from offline tests and from future live-provider acceptance.
