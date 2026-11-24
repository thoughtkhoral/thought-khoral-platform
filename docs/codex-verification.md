# Codex room-conversation verification

## Local loopback health verification — 2026-11-24

Authority: [approved local design](../.ai/specs/how/task9-runtime-compatibility.md#approved-loopback-health-diagnosis--2026-11-24).
The original policy left three health-check clients in LAST_ACK; counter-only
classification confirmed six socketless ESTABLISHED reply ACKs with the approved
zero-payload header form. The narrow correction leaves zero clients in LAST_ACK.
It adds no application-owned socket permission and changes no watchdog timeout,
whole-policy comparison, IPv6 rule or initial connection authority.

The local ARM64 proxy/namespace-owner image is
`sha256:54dc5c7d649e486bff7f5f8821050b800ae5cb454cd8496a9f93ce0047d07cf1`,
built from clean source `a9c3ff5c79e7d3525bed0b5968ab1d8be6b2cc23`.
Installed policy SHA-256 is
`023203915acb11fe9b66183bc65426e52783fa934a89f252c15be5a147ae01dc`.
Actual packaged-image tests pass without a mounted policy override. Nine raw
cases match the header predicate only for the exact empty ACK; data/options/
length/flag variants do not match. These counter-only probes target refused
port 3130; actual health teardown separately proves the allow rule.
The retained `python3 scripts/test-codex-egress.py --kernel` passes controlled
local TLS, IPv4/IPv6/direct-provider/DNS denial, proxy loss/readiness expiry and
policy-tamper fail-stop. All 54 profile/historical/schema/compatibility guards,
five Compose checks and input/startup checks pass.

Reproduce the owned network-none regression, negative header probes and bounded
duration/proxy-loss check without provider credentials or a live stack:

```sh
python3 scripts/tests/codex-loopback-health.py \
  --image sha256:54dc5c7d649e486bff7f5f8821050b800ae5cb454cd8496a9f93ce0047d07cf1 \
  --header-probes --watchdog-seconds 660
```

The exact packaged image passed 660.005 seconds; stopping the proxy then stopped
the namespace and dependent. All owned containers were removed. An earlier run
using the same policy bytes on the unchanged base image passed 660.003 seconds,
then stopped its namespace and dependent when the proxy was deliberately lost.
This evidence is bounded; it accepts no full packaged-stack/browser/live gate.
Source/image publication, existing-data upgrades and changed-input activation
remain separate. Both original profile files and their digests remain unchanged.

## Current workspace context — 2026-11-20

The exact packaged-v1.1 acceptance profile is implemented and reviewed at local source `996e1887234b8958a315279aa3d5aa194a7fad52`. Profile SHA-256 is `7106f7e41a6bc13feab8a0b51fccde9b47fefca2d63980d551ed929eac1c1a71`. Forty-six profile/historical-guard tests and actual credential-free image/source preflight passed. The verified local ARM64 image passed ten package/startup probes and byte-identical native capture; this is not full packaged-stack/private-DNS/browser/live acceptance.

This checkout contains the reviewed released-v1.1 implementation. The
maintainer authorized coordinated integration on 2026-11-23. Frozen component
acceptance commits remain unchanged; packaged/browser/live gates remain open.

Derived from the [local specification index](../.ai/specs/README.md) and
[root workspace authority](https://github.com/thoughtkhoral/thought-khoral/blob/main/.ai/specs/how/workspace-state.md).
The cited tests retain their original source identities; this documentation
refresh adds no activation, publication or runtime acceptance.

Governing sources: [local requirements](../.ai/specs/what/codex-local-service.md),
[approved local design](../.ai/specs/how/codex-local-service.md), and the
[coordinated milestone plan](https://github.com/thoughtkhoral/thought-khoral/blob/main/.ai/specs/how/codex-room-conversations-implementation-plan.md).
The exact profile is `thought-khoral.agent-conversation.v1` from contracts tag
`thought-khoral-agent-conversation-v1.0.0` (`85baf86e574276fcd036e53e23641af6aad602f9`).
This guide is derived verification procedure; it does not change those specifications.

<a id="current-evidence-boundary--2026-11-19"></a>

## Historical evidence boundary — 2026-11-19

The [owning maintenance checkpoint](../.ai/specs/how/codex-local-service.md#current-maintenance-and-verification-scope--2026-11-19)
records the published platform build-input correction, passing deterministic
eight-service smoke and separate native Linux worker acceptance. The commands
and exact source tables below retain their reviewed synthetic fixture pins;
they are not updated to later `main` heads without a new acceptance run.
Task 9 opt-in packaged room-stack, rendered-browser/Keycloak and live-provider
verification remain open, along with v1.1 contract, image and release publication.
Local containers remain stopped and the external database volume preserved.

## Approved runtime compatibility preparation — 2026-11-23

The [approved local design](../.ai/specs/how/task9-runtime-compatibility.md)
records reproducible proxy loopback extra-host syntax and mediator DNS ownership.
`bash scripts/test-codex-compose.sh` exercises installed Compose's actual create
argv with synthetic inputs, a temporary project and a replacement engine. Its
dry run creates no containers and contacts no provider. Real broker-to-mediator
private DNS and authenticated catalog verification remain separate integration
evidence.

The standalone preflight below inspects an explicitly selected owned PostgreSQL
container's actual schema over container-local authenticated TCP. It uses only
system catalogs in a read-only transaction; it never applies migrations. All
connection/statement/lock/process time and output are bounded. It requires the
0006/0007 conversation columns, defaults, validated constraints and valid indexes,
plus the referenced room table structure. Migration-ledger rows alone do not
satisfy it. Output contains only readiness and migration identifiers; database
errors and credentials are suppressed.

```sh
node --test scripts/tests/codex-schema-readiness.test.mjs
node scripts/check-codex-schema.mjs <owned-postgres-container>
```

The exported `verifyConversationSchema(container, execute)` hook is reserved for
the future separately reviewed compatibility profile. Existing historical modes
and profile bytes remain unchanged. That new profile must call the hook before
provider requests and bind the exact clean runner and corrected worker image.
The root orchestrator owns the private backup/isolated migration check and the
separate existing-volume upgrade gate. This preparation grants no upgrade,
activation, publication, provider-call or Task 9 acceptance authority.

## Prepared separate compatibility profile — 2026-11-23

The separate [compatibility pins](../scripts/fixtures/codex-conversation/packaged-v1.1-runtime-compat-pins.json)
have SHA-256 `14c5a784a1f6d8c944f25570f1b0dda741831178348ad7052ecb7301529d2f0c`.
They change only the profile identity, corrected worker source `423d4e8` and
immutable ARM64 image `sha256:1ac229b25167845b4d7cec217304f3064b02d65d60c53f35b07c5ae3e3683b3b`.
All other frozen component sources, CLI, lock and proof identities remain equal
to the original profile, whose bytes and digest remain unchanged.

Actual network-none production-image adapter regression returns HTTP 503 on the
original worker and HTTP 200 on the corrected worker through authenticated
`GET /control/v1/models`. The corrected normalized catalog contains gpt-6.1-sol
with low, medium, high, xhigh, max and ultra reasoning options. The native catalog
default is low; platform defaults remain gpt-6.1-sol/medium. Fresh receipt storage
contains zero conversations and tasks. Owned fixtures are removed after each
run. This checks real native initialization/catalog through the adapter, with
no model turn, provider access or stack activation.

```sh
python3 scripts/tests/codex-native-catalog.py --image sha256:1ac229b25167845b4d7cec217304f3064b02d65d60c53f35b07c5ae3e3683b3b
node --test scripts/tests/codex-runtime-compat.test.mjs scripts/tests/codex-schema-readiness.test.mjs
```

The [separate operator example](../scripts/fixtures/codex-conversation/live-operator-packet.runtime-compat.example.json)
retains false activation/provider approvals and an invalid all-zero platform
revision until an operator supplies separately approved exact inputs. It adds
`postgresContainer`, a closed local container name. Only this profile invokes
the bounded authenticated read-only schema check before provider requests,
recording only safe readiness evidence. The unchanged original database currently
fails that check; the separately restored and migrated clone passes. The
existing-data upgrade requires separate approval. The namespace-owner watchdog's
provider-proxy health failure remains under investigation. Profile preparation
does not establish activation readiness or accept Task 9.

After separate readiness/activation/provider gates are satisfied, the explicit
selector is `--live --allow-live --acceptance-profile packaged-v1.1-runtime-compat`.
Its actual clean runner HEAD must match the packet's platform revision; it is
not embedded in its own profile. Cross-profile packets, malformed database names
and missing schema fail closed. Historical/default/original-profile behavior
and fixtures retain their prior interpretation.

## Prepared released-v1.1 packaged profile — 2026-11-20

The [approved validation design](../.ai/specs/how/packaged-profile-validation.md)
adds a separate exact `packaged-v1.1` profile after published artifact download
verification and local broker/UI released-provenance transition. Its
[source-controlled pins](../scripts/fixtures/codex-conversation/packaged-v1.1-pins.json)
bind broker `673294e`, UI `29a9400`, published contracts `f1c8621`, and the actual
provider-free verified final-lock ARM64 worker package. The profile bytes are
anchored at SHA-256
`7106f7e41a6bc13feab8a0b51fccde9b47fefca2d63980d551ed929eac1c1a71`.
Worker image ID is
`sha256:2dd78cdf5eddb9ca271611998ea44fe6d6cfde6cfca8ba7d1d6c8858240b540d`.
It is a local immutable image, not a published registry reference. Component
branches remain local until separately published/merged; their full identities
are in the profile. Historical modes and their exact candidate pins below
remain unchanged.

Run offline preflight regression coverage without a container or provider:

```sh
node --test scripts/tests/codex-packaged-pins.test.mjs scripts/tests/codex-conversation-smoke.test.mjs scripts/tests/codex-defaults-pins.test.mjs
```

After separately authorized activation/provider configuration, copy the
[packaged operator example](../scripts/fixtures/codex-conversation/live-operator-packet.packaged.example.json)
outside all Git worktrees with owner-only permissions. Set a dedicated empty
room, two actual private human token paths, new evidence path and the exact
platform commit from the clean executing checkout. The all-zero example
platform revision is deliberately invalid; never treat the example as approved
activation. Approval booleans remain false until actually approved.

Set `TASK9_CONTRACTS_REPO`, `TASK9_BROKER_REPO`, `TASK9_MEDIATOR_REPO`,
`TASK9_WORKER_REPO`, `TASK9_UI_REPO` and `TASK9_MEMORY_REPO` when source checkouts
are not in the default sibling layout. These override paths only. Each must
have the exact clean commit in the profile; contracts may use a detached clean
checkout at the release tag. An evidence-only later commit is not silently
substituted. The platform's actual commit is checked separately to avoid a
self-referential committed profile.

```sh
node scripts/smoke-codex-conversation.mjs --live --allow-live --acceptance-profile packaged-v1.1
```

That invocation still requires `TASK9_LIVE_OPERATOR_PACKET` and all existing
private path, loopback, distinct-human, output-exclusivity and interactive
terminal checks. Before broker requests, it verifies activated container image,
Linux/ARM64, source/lock labels, actual worker package success and measured CLI
and proof-file bytes. Labels alone cannot establish acceptance. New engine
commands have bounded output/time and failures expose generic diagnostics.
Evidence adds the profile digest, release provenance and safe measured package
identities; no credentials or raw native output are recorded.

Local preparation passed full broker tests with a removed temporary database,
UI tests/build, actual worker package/startup probes and byte-identical synthetic
native capture. The CLI's selected and historical profiles both reach the
terminal gate in synthetic offline integration. These results do not activate
the stack or establish real-browser, private-DNS, provider or live-isolation
acceptance. The original Task 9 live/manual evidence requirements still apply.

## Historical provider-free composed smoke

Latest contract checkpoint — 2026-11-20: the
[v1.1.0 artifact](https://github.com/thoughtkhoral/thought-khoral-contracts/releases/tag/thought-khoral-agent-conversation-v1.1.0)
is now published and fresh downloads verify. Broker/UI still retain historical
candidate pins pending released-provenance adoption. The commands below keep
their exact historical source/profile definitions; no new live profile or
activation follows from the contract release. The earlier open-publication
statement is a dated checkpoint, not current artifact status.

For the next current-source packaged/browser run, read the
[approved 2026-11-20 acceptance plan](../.ai/specs/how/task9-packaged-acceptance-2026-11-20.md)
and its root procedure first. They identify release/consumer prerequisites and
a separate acceptance profile, now implemented above. The executable commands and historical pins below
remain unchanged; current `main` revisions cannot simply replace their inputs.

From this platform checkout, with Podman, Node, Cargo, Python 3, the local
`postgres:16` image, and free loopback ports 8080, 9091, and 9092:

```sh
node --test scripts/tests/codex-conversation-smoke.test.mjs
node --test scripts/tests/codex-conversation-cleanup.test.mjs
node scripts/smoke-codex-conversation.mjs --fake
```

The runner requires clean source worktrees at these exact revisions. Set the
three path variables if they are elsewhere:

| Variable | Reviewed repository revision |
| --- | --- |
| `TASK9_BROKER_REPO` | room gateway `fd05cb48b8508e7939f9cdf9df275742a06fc4f8` |
| `TASK9_MEDIATOR_REPO` | agent gateway `6c3d96b4763871b9addc9bc7223e71ee7d38abd9` |
| `TASK9_WORKER_REPO` | Codex worker `b0d43ec2b5b0c8da035d4ccff754545132b978d4` |

The default paths are the Task 9 review worktrees under `/private/tmp`.
The runner checks each revision and cleanliness, generates local Cargo path
dependencies and the reviewed `a2a-client-lf` patch, then builds offline with
the committed fixture lockfile. Its external package/version/source set is a
subset of the union of the three reviewed runtime locks. The fixture itself is
Apache 2.0; its fake app-server is derived from the worker's Apache 2.0 test
fixture. It does not change a runtime package or its release image.

The runner starts only its own `Task9-postgres-<pid>` container with an
ephemeral database and no volume, and binds its Axum services to loopback.
The broker and synthetic RSA issuer share port 8080 because the real mediator
pins both service authorities to port 8080. The mediator's reviewed loopback
DNS injection resolves those authorities only inside this fixture. The worker
serves authenticated A2A and controls on 9091; the real mediator catalog
service listens on 9092. A thin `CatalogQuery` transport adapter forwards
broker reads to that real catalog HTTP endpoint. The runner rejects occupied
fixed ports before making fixture requests. It never edits hosts or DNS.

The worker is a child process with retained SQLite receipts and synthetic
native session files. The mediator runs as restartable child processes. Only
the native app-server executable is fake. The fixture uses the real broker
Axum routes, JWT validation, PostgreSQL migrations, conversation store and
context builder, mediator dispatcher/catalog/WorkerClient, and worker A2A,
receipt, and native-protocol adapter. Ordinary room history is seeded through
the broker's production `append_event` function. The runner registers owned native process groups, terminates its own detached
fixture group and native descendants, waits for exit, checks container removal,
and only then removes its temporary state. Assertion failure, SIGINT, and
SIGTERM receive the same cleanup; errors retain state and fail the run. The
cleanup regression command injects a real held-boundary assertion failure and
both outer signals, with a held native descendant, and checks no owned processes
or container remain. Set
`TASK9_KEEP_FIXTURE=1` only for debugging synthetic state; it still stops the
container. `TASK9_CARGO_TARGET_DIR` may select a separate fixture build cache.

The fake executable advertises `0.160.0` in its protocol fields so the real
worker adapter accepts its synthetic responses. That is a stub value, not a
measurement of the pinned Codex CLI binary or package. The actual CLI/image
identity and tool-policy proof remain separate package evidence: the Task9 image
`sha256:2d8bfade27802f910cf68e832722c93b4a2acc2addb825711e1223617a4cd385`
compiles runtime source `b418a76e0e7ca047b5fe995eb17519aced369a06`; worker
HEAD `b0d43ec2b5b0c8da035d4ccff754545132b978d4` adds its evidence record.
Its unchanged native capture retains Task8 attribution, not a new capture claim.

The assertions use stored context source and trigger event IDs, native request
capture, PostgreSQL task/reply rows, mediator durable records, worker SQLite
receipts, and authenticated HTTP task views. They cover public baseline and
ordered delta, hidden targeted sequence gap, one occurrence of the invoking
text, another room's rejected conversation binding, concurrent duplicate
requests yielding one task/turn/reply, worker restart with one native
`thread/start` and one `thread/resume`, a distinct reset thread with defaults,
model and effort change, latest-request usage, and synthetic provider denial
without fallback. Both native JSON inputs are decoded and compared to the exact
frozen packet, including trigger and every source entry. Continuation asserts
exactly two entries and one native reply binding with the accepted event,
source task, generation, sequence, and text digest. Old baseline entries and
assistant reply text cannot appear as new input. Synthetic provider denial is
now a handled failure: the broker must expose the authenticated receipt-correlated
`execution_failed` enum, without native error strings or a fallback. All mediator
stderr remains bounded; any unexpected diagnostic fails the gate.

Another human continues with omitted settings after worker restart: the accepted,
effective, and native pair must remain model B/high. Explicit New without overrides
still selects deployment defaults. A separately held completed result is submitted
after the real broker deadline and rejected; mediator recovery quarantines it.
A fresh higher generation succeeds after worker restart on a distinct native
thread, preserving the old completed receipt and never committing its reply.
Stale continuation and premature New are rejected. Deleting only that fresh
synthetic history exercises `session_unavailable`; a fake runtime exiting before
thread creation exercises `runtime_unavailable`. Both require real matching
worker receipts, exact public failure codes, zero additional turns, and no reply.

| Injected boundary | Durable state before process loss | Required observation |
| --- | --- | --- |
| After submission intent, before worker request | mediator `submission-intent`; no worker receipt | zero native turns; broker fails after its real deadline |
| Before native thread creation | worker `reserved`, no thread or turn binding | zero native turns; restart marks receipt interrupted; no reply |
| After native thread binding, before turn response | worker `running`, thread bound, turn unbound | one native request; interrupted receipt; no reply |
| After native turn binding | worker `running`, thread and turn bound | one native request; interrupted receipt; no replay |
| After normalized worker completion, before broker commit | worker `completed`, mediator `completed`, broker `running` | restart commits one reply and acknowledges same worker receipt |
| After broker commit, before acknowledgement | broker `completed`, worker acknowledgement absent | restart stores the acknowledgement without another native turn or reply |

The last two use a one-shot delay around the real broker update route; no
broker validation or commit logic is replaced. The first four use a bounded
five-second fixture turn deadline and real broker expiry, so they establish
visible failure rather than an exactly-once provider guarantee. The turn-bound
window polls the actual SQLite receipt until running phase and thread/turn
binding are durable; the fake protocol marker alone is not the kill barrier.

This evidence is **real synthetic composed-stack** evidence. It does not run
the packaged Compose deployment, a real Keycloak server, browser UI, retained
WebSocket ingress, production private DNS for `CatalogBridge`, kernel/proxy
policy, or provider inference. Those boundaries retain their own component,
packaging, UI, and kernel gates. The fake answer text alone is never used as
proof of history, isolation, or exactly-once execution.

## Gated live operator run

Live provider checks require separate approval to activate the reviewed local
service and to make provider calls. No live call is part of `--fake`. After
that authorization, use a dedicated empty room, two distinct human accounts,
externally configured API-key access, and an already healthy opt-in deployment.
Copy [the packet template](../scripts/fixtures/codex-conversation/live-operator-packet.example.json)
to a private path outside Git and fill in the actual room ID, unique `Task9-`
codes, two private token-file paths, activated worker container/image digest,
and exact revisions. Set `activationApproved` and `providerCallsApproved` only
after they are actually approved. The script reads but never prints the human
tokens; the operator packet and token files must be owner-only regular files
outside **all** Git worktrees. The path guard follows existing parent symlinks
and recognizes linked worktrees before reading packet/token contents.
The broker must be port-forwarded to the exact loopback HTTP origin in the
packet. Replace the template's zero platform revision with `git rev-parse HEAD`
from this verified checkout; the runner requires the reviewed revisions and
image digest. Select `alternateSettings` from the active authenticated model catalog
if an accessible second model or effort is available. Set
`toolProbesApproved` only if the five live tool and key-exposure prompts are in
the approved scope. The evidence file must be a new path in an existing
directory outside all Git worktrees. Its canonical destination is preflighted
before container inspection, then reserved with mode 0600 before broker requests;
an existing file is refused and never overwritten.

```sh
TASK9_LIVE_OPERATOR_PACKET=/private/tmp/Task9-live-operator.json \
  node scripts/smoke-codex-conversation.mjs --live --allow-live
```

The runner fails before contacting the broker unless it has the explicit flag,
packet approvals, exact loopback origin, separate human credentials, named
revisions/contract/CLI/image, and a matching activated worker image. In the
interactive run it asks Maya to post a public fact without invoking Codex;
Leo then posts a public correction and a targeted canary to someone other than
Codex. It checks that those messages did not start a conversation, explicitly
submits a Codex turn as Leo, and checks the committed reply for both public
codes and absence of the targeted canary. Maya adds intervening public
discussion. The operator restarts the worker through the approved local
Compose procedure; the runner compares container start times and continues
the same conversation, then creates a new session and checks that the initial
public fact is available as a fresh authorized baseline. It records the
selected/effective settings and usage from each task. A configured alternate
setting is exercised on the same conversation. Any failed task records its
known safe broker failure code, fails the live gate, and retains partial
evidence without reply text. The pinned profile cannot distinguish account
model denial from its generic errors, so account access remains unclassified;
authentication, timeout, protocol, authorization, and recovery failures cannot
be reported as an unavailable account. Approved shell, file, external-tool, destination,
and provider-key prompts are submitted as separate explicit turns.

The JSON result deliberately stores IDs and projections, not tokens or reply
text. A passing script result is **live API/answer evidence**, not complete
release evidence. Before accepting live coverage, inspect the retained worker
receipt database for equal thread IDs on continuation and a distinct ID after
reset; verify task/source IDs against broker packet projections; inspect the
real native request and tool-policy evidence and proxy/kernel audit for zero
shell, file, MCP, arbitrary-destination, or key-disclosure execution. A reply
that merely says it refused a tool is insufficient. The worker receipt file is
under the activated `THOUGHT_KHORAL_CODEX_STATE_ROOT/receipts/worker.sqlite`;
perform any read-only inspection with the authorized state owner and keep
private room data outside Git. If the pinned CLI or network evidence cannot
establish the restrictions, keep the service disabled and record the failed
gate. Do not infer isolation from model forgetting: a reset may legitimately
receive older public room facts through its new baseline.

Milestone acceptance also requires the retained contracts, gateway,
mediator, worker, UI, platform, root specification/reference/identity,
licensing, secret/session exclusion, and independent build/release checks.
Record each result against the source revision or image used. Publishing an
image or tag, activating or deploying a service, pushing or merging a branch,
and making live provider calls remain separate actions.

## Unreleased defaults-discovery candidate

The [approved defaults amendment and local plan](../.ai/specs/how/default-settings-discovery-implementation-plan.md)
authorize this separate local synthetic mode:

```sh
node --test scripts/tests/codex-defaults-pins.test.mjs
TASK9_KEEP_FIXTURE=1 node scripts/smoke-codex-conversation.mjs --fake --defaults-candidate
```

The default `--fake` and live operator packet retain the reviewed published v1.0
pins above. `--defaults-candidate` is rejected with `--live` before any packet,
credential, listener or container access. Candidate metadata is committed in
[`defaults-candidate-pins.json`](../scripts/fixtures/codex-conversation/defaults-candidate-pins.json)
and anchored by the runner. It names contracts commit
`1ea828f28725ddaaefa21d083473f9abbd777975`, archive SHA-256
`fab59a486f6498b843467202debcb0768403bd57ba7dda41be2a01e5f23fdda8`, and external
lock SHA-256 `7914d32eae2487879a68405b5095a6b9aa91355f87529c43f4055844821902a9`.
The historical candidate remains explicitly `unreleased-local-candidate`.
The separate v1.1.0 release was published on 2026-11-20; this fixture's exact
candidate pins and synthetic-only authorization remain unchanged pending the
verified consumer transition.

| Source | Exact candidate revision | Path override |
| --- | --- | --- |
| Contracts | `1ea828f28725ddaaefa21d083473f9abbd777975` | `TASK9_CONTRACTS_REPO` |
| Broker | `2e7d23b467c572819f498c3b9bf14d74a62dc821` | `TASK9_BROKER_REPO` |
| UI including test-only orchestration | `79e5e7310a450efea561548cd87871446c1939aa` | `TASK9_UI_REPO` |
| Mediator | `6c3d96b4763871b9addc9bc7223e71ee7d38abd9` | `TASK9_MEDIATOR_REPO` |
| Worker | `b0d43ec2b5b0c8da035d4ccff754545132b978d4` | `TASK9_WORKER_REPO` |

Overrides change source paths only. Every revision and clean tree is checked
before resources start. The anchored external archive and all 156 candidate
payload files must agree with both broker and UI vendors, including exact lock
bytes. The 135 published payload files and published locks in broker, UI,
mediator and worker remain byte-for-byte unchanged. Extra, missing, changed or
symlinked vendor entries fail closed. Mediator and worker continue using the
unchanged v1 task/result protocol and therefore retain their reviewed sources.

The candidate runner preserves the baseline's 11 native turns and six recovery
boundaries, then adds 12 actual rendered `RoomPage` HTTP cases: both controls,
effort only, model only, and neither, each with initial, restored and explicit
New phases. B/high deployment defaults differ from the first catalog model A.
After initial completion, deployment defaults change to A/medium; restored
shared B/high persists without another defaults read, while explicit New uses
A/medium and a distinct native thread. Neither-capability UI requests omit
settings and make zero defaults/catalog reads; assertions cover the broker's
selected pair without claiming that hidden settings were displayed.

The fixture reads the actual DOM pair and actual accepted HTTP response,
compares the task-bound PostgreSQL frozen packet, dispatches through the real
mediator and worker, and matches native input by the exact trigger plus worker
receipt thread binding. Additional display/send barriers prove a default-only
change preserves a displayed valid B/high pair, and stale/removed pairs are
rejected without any task, event or native turn. The UI retains its prompt,
blocks invocation on a repeated Send action and exposes explicit Refresh settings. Replaying an accepted
request after defaults/catalog unavailability returns its original acceptance
without another native turn. Candidate cases add 13 native turns independently
of the baseline count.

`GET /api/agent-conversations/v1/rooms/{roomId}/agents/{agentId}/defaults` requires
the existing authenticated-human authority and returns only the closed selected
settings view with `Cache-Control: no-store`. Direct HTTP checks confirm no
conversation, task, event, lease or native-thread allocation. The current
five-second settings-validation bound still applies. Only the test policy's
turn deadline is longer to allow the UI child to finish its local test process
before mediation; it does not alter deployment behavior.

Display and release handshake messages are written and closed in unique
same-directory staging files, then atomically published through a hard link
that refuses replacement of an existing final name. Paused-writer tests verify
that readers cannot observe incomplete messages; completed invalid release
messages still fail closed. Staging files are removed on success or conflict.

Each candidate run prints its owned state directory and exact source metadata.
The printed record and `defaults-source-pins.json` also include `validatedPaths`,
the resolved checkout paths actually checked by source preflight, including any
`TASK9_*_REPO` overrides. Reviewed metadata paths, source heads and artifact hashes
remain unchanged beside this run-specific provenance.
`defaults-matrix.json`, per-task comparisons, exclusive UI evidence files and
child logs with bounded process completion remain available with `TASK9_KEEP_FIXTURE=1`; candidate
failures retain evidence. Processes and the owned database container are always
cleaned. The child uses jsdom and a synthetic room socket with real conversation
HTTP. This is local synthetic acceptance; packaged Compose, a browser, real
Keycloak and live-provider acceptance remain separate pending gates. The four task
reviews and final whole-branch review passed for the exact runtime and test-only UI
revisions. F1 and defaults discovery are accepted for this local synthetic candidate.

## Documentation followup and frozen sources

Documentation-only HEADs can advance after the tested profile source. Set the
component TASK9_*_REPO paths to preserved clean snapshots when current checkouts
no longer equal the profile commits. The executing platform revision is measured
separately. See the [current workspace guide](https://github.com/thoughtkhoral/thought-khoral/blob/main/docs/workspace-status.md)
for the local source map and publication/integration boundary. No profile or
artifact bytes changed during this documentation refresh.
