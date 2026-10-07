# Codex room-conversation verification

Governing sources: [local requirements](../.ai/specs/what/codex-local-service.md),
[approved local design](../.ai/specs/how/codex-local-service.md), and the
[coordinated milestone plan](https://github.com/thoughtkhoral/thought-khoral/blob/main/.ai/specs/how/codex-room-conversations-implementation-plan.md).
The exact profile is `thought-khoral.agent-conversation.v1` from contracts tag
`thought-khoral-agent-conversation-v1.0.0` (`85baf86e574276fcd036e53e23641af6aad602f9`).
This guide is derived verification procedure; it does not change those specifications.

## Provider-free composed smoke

From this platform checkout, with Podman, Node, Cargo, Python 3, the local
`postgres:16` image, and free loopback ports 8080, 9091, and 9092:

```sh
node --test scripts/tests/codex-conversation-smoke.test.mjs
node scripts/smoke-codex-conversation.mjs --fake
```

The runner requires clean source worktrees at these exact revisions. Set the
three path variables if they are elsewhere:

| Variable | Reviewed repository revision |
| --- | --- |
| `TASK9_BROKER_REPO` | room gateway `50d293491ae65250e0603d26645d4bcc4e692b90` |
| `TASK9_MEDIATOR_REPO` | agent gateway `1900f8d127d744ffb996021fdc2f3fb34858fbda` |
| `TASK9_WORKER_REPO` | Codex worker `d40e4a8cd5efc77c7161742aec7ade289efb357a` |

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
the broker's production `append_event` function. The runner stops its own
container and children and removes its own temporary state. Set
`TASK9_KEEP_FIXTURE=1` only for debugging synthetic state; it still stops the
container. `TASK9_CARGO_TARGET_DIR` may select a separate fixture build cache.

The fake executable advertises `0.160.0` in its protocol fields so the real
worker adapter accepts its synthetic responses. That is a stub value, not a
measurement of the pinned Codex CLI binary or package. The actual CLI/image
identity and tool-policy proof remain separate Task 8 package evidence.

The assertions use stored context source and trigger event IDs, native request
capture, PostgreSQL task/reply rows, mediator durable records, worker SQLite
receipts, and authenticated HTTP task views. They cover public baseline and
ordered delta, hidden targeted sequence gap, one occurrence of the invoking
text, another room's rejected conversation binding, concurrent duplicate
requests yielding one task/turn/reply, worker restart with one native
`thread/start` and one `thread/resume`, a distinct reset thread with defaults,
model and effort change, latest-request usage, and synthetic provider denial
without fallback.

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
visible failure rather than an exactly-once provider guarantee.

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
tokens; their files must be outside this repository with owner-only access.
The broker must be port-forwarded to the exact loopback HTTP origin in the
packet. Replace the template's zero platform revision with `git rev-parse HEAD`
from this verified checkout; the runner requires the reviewed revisions and
image digest. Select `alternateSettings` from the active authenticated model catalog
if an accessible second model or effort is available. Set
`toolProbesApproved` only if the five live tool and key-exposure prompts are in
the approved scope. The evidence file must be a new path outside Git.

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
setting is exercised on the same conversation; inaccessible account coverage
is reported as unavailable. Approved shell, file, external-tool, destination,
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
