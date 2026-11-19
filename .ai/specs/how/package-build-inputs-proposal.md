# Complete container build inputs for the integrated conversation code

Status: accepted by the maintainer on 2026-11-19.

Approval record: the maintainer replied "approved" to the two concrete
architecture-evidence and platform-build amendments before implementation.
This continues the existing accepted [platform contribution issue 1](https://github.com/thoughtkhoral/thought-khoral-platform/issues/1)
under the coordinated milestone-one plan; it does not assert a newly filed
issue, remote review, or publication.

Governing sources: [local implementation](implementation.md),
[local MVP requirements](../what/local-mvp.md),
[Codex service packaging](codex-local-service.md), and the
[root specification-first rule](https://github.com/thoughtkhoral/thought-khoral/blob/main/.ai/specs/README.md).

## Reproduced failures

At platform revision `f07065c4a17636871b3b8f01d7642b6cd0554db3`, the default
`podman-compose --parallel 1 -f compose.yaml up --build -d` fails while building
the agent gateway. Six `include_str!` expressions in
`src/conversation_validation.rs` cannot read the vendored conversation schemas.
`containers/agent-gateway.Containerfile` copies the source and patched A2A
vendor directory, but omits the `contracts/` directory.

An independent build of `containers/ui.Containerfile` fails during the mandatory
`prebuild` hook with `MODULE_NOT_FOUND` for
`/source/scripts/check-conversation-contract.mjs`. The UI build now needs its
contract-verification scripts and both vendored conversation profiles; neither
directory is copied into the build stage. Existing room-gateway packaging
already includes its `contracts/` directory.

Both failures precede application startup. They do not establish a runtime,
identity-provider, database, or provider-conversation result.

## Approved bounded change

Add the missing existing source inputs before the current build commands:

```dockerfile
# containers/agent-gateway.Containerfile
COPY thought-khoral-agent-gateway/contracts/ contracts/

# containers/reference-agent.Containerfile
COPY thought-khoral-agent-gateway/contracts/ contracts/

# containers/ui.Containerfile
COPY thought-khoral-workspace-ui/contracts/ contracts/
COPY thought-khoral-workspace-ui/scripts/ scripts/
```

Preserve all contract bytes, lockfiles, existing verification hooks, runtime
images, entrypoints, service identities, network controls, and volume mappings.
Do not bypass the UI prebuild checks or replace contract imports with mocks.

## Acceptance

1. Use the actual failed image builds as the initial regression evidence.
2. Build all three images from the sibling source layout and require their existing
   compiler and contract-verification steps to pass.
3. Run the default Compose startup and smoke checks, with the opt-in Codex
   service disabled. Preserve the existing external PostgreSQL volume.
4. Run source-staging tests and root specification/reference/identity gates;
   record exact source/image revisions and any remaining failures separately.

The original 2026-11-19 approval to complete the integrated code's existing
build inputs also covers the identical gateway contracts input in the separate
reference-agent binary's Containerfile. The default build exposed this second
consumer after the first two image builds passed. Both binaries compile the
same conversation-validation library; this discovered packaging omission has
the same rationale and changes no behavior, interface, or policy.

This correction grants no service permission beyond the local MVP checks
requested on 2026-11-19. Live Codex activation, provider calls, release
publication, remote push/merge, contract publication, and full milestone
acceptance retain their existing gates.

## Implementation evidence — 2026-11-19

At the implementation checkpoint, the accepted `COPY` additions were applied on platform base
`f07065c4a17636871b3b8f01d7642b6cd0554db3`, consuming agent-gateway source
`6bfdd4420c72fa7eb25d30d48f06a8a7a0bd0ec1` and UI source
`531cd912b5e5eb9ffd5ae175cbf8498d9c757192`. The actual sequential builds
of the corrected images passed, including the UI's existing prebuild checks:

- Agent gateway: `sha256:f3b2f6c4c72cd9192408f94d849a7c146653f5eb318c7e1550fc6dc6d51f933a`.
- Workspace UI: `sha256:3955414b3994bb4db60c4169d2f1932d44a35ab72bdfaf689f7fb63b4214a897`.
- Reference agent: `sha256:4f92407bf83267e7063b6e7e5ea6d95c7eb065d6e38579229f46ebe75b4f8907`.

`scripts/test-build-remote.sh` and the root specification hierarchy,
repository-reference, and identity gates passed. Build and verification logs
are retained locally under `/private/tmp/thought-khoral-resume-2026-11-19/`.

The subsequent default `podman-compose --parallel 1 -f compose.yaml up --build -d`
failed before startup while building the separate reference-agent image. Its
Containerfile also omits the same existing gateway `contracts/` input, and its
shared library compilation reports the same six missing schema files. This
additional Containerfile now receives the identical contracts input under the
original approved correction. The subsequent default eight-service Compose
build and startup passed.

The first smoke passed bootstrap invariants, service readiness, kernel egress
checks, UI freshness/title, and fresh PKCE authenticated room entry, then failed
at workload-token acquisition with HTTP 500. Keycloak logged a JDBC connection
failure while PostgreSQL logged a backend SIGPIPE and automatic recovery. One
retry failed at browser authorization with HTTP 500 during further database
recovery. Keycloak subsequently exited 137 with `OOMKilled=true`, confirmed by
Podman inspection. These runs do not establish a passing full-stack smoke.
The failure occurred while independent worker-image builds shared the Podman
VM; this observation alone does not establish their causal role.

Logs include `platform-default-up.log`, `platform-default-smoke-first.log`,
`platform-default-smoke.log`, `platform-keycloak.log`, and
`platform-keycloak-retry.log` in the local evidence directory above. No Codex
service or provider was activated, and the existing external PostgreSQL
volume was preserved. No runtime or database correction was
made for these failures.

After the independent worker builds and tests exited, the default stack was
restarted with `podman-compose --parallel 1 -f compose.yaml up --no-build -d`.
With exclusive VM resources, `scripts/smoke.sh` passed in full (exit 0):
bootstrap invariants, all readiness checks, kernel egress allow/deny checks,
fresh PKCE authentication and explicit room join over HTTP/WebSocket,
authorized-context packet filtering, and both human-invoked A2A skills with
ordered progress and one cited result. `browser-smoke.mjs` is a Node.js HTTP
and WebSocket probe; this result does not claim rendered-browser validation.

The final passing run is recorded in `platform-default-smoke-final.log`;
`platform-default-up-final.log`, `platform-final-services.log`, and
`platform-final-images.log` record startup, running services, and exact image
IDs. At that checkpoint all eight default services were running at
`http://localhost:8082`; the seven declared healthchecks were healthy and the
agent gateway was running. The corrected images retain the IDs listed above.
The containers were subsequently stopped and removed at the maintainer's
explicit cleanup request, without deleting the external PostgreSQL volume.
Codex was never activated. All prior failure logs remain available. The root
identity gate passed again after the final documentation correction.

## Published maintenance checkpoint — 2026-11-19

The approved build-input correction was subsequently committed as `74ac640`
and pushed to GitHub `main` under the maintainer's separate authorization.
The three image IDs and passing default eight-service smoke above remain
attributed to their tested source inputs; this source publication is not a
new image build or a rerun against the worker's later final lock. Local
containers remain stopped/removed and the external database volume is preserved.
Opt-in Codex Task 9, rendered-browser, live-provider, contract publication and
production/release acceptance remain open.
