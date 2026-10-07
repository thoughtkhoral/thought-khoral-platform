# Opt-in local Codex packaging

Governing sources: [local requirements](../.ai/specs/what/codex-local-service.md)
and [approved design](../.ai/specs/how/codex-local-service.md). This is local
packaging; activation and live provider use need separate authorization.

`compose.yaml` remains the deterministic eight-service deployment and requires
no Codex settings. `compose.codex.yaml` is an explicit additional file. It adds a
separate Codex network/PID owner, restricted proxy, and independent worker image.
The base Kubernetes manifests do not include Codex.

The reviewed local packaging architecture is Linux aarch64. The proxy dependency
lock and worker native tool-capture evidence must be reviewed separately before
another architecture can be admitted.

The original Task 6 image does **not** pass this packaging gate. Version-only
verification is insufficient: `--verify-package` must validate the corrected
pinned catalog/control and actual tool-capture evidence and print
`tool policy verified; exposed tools: []`. Readiness never uses an operator-set
boolean as evidence that native tools are absent.

## Configuration

Use an immutable reviewed worker image ID or digest in
`THOUGHT_KHORAL_CODEX_IMAGE`. Keep these three distinct, private files outside
version control and use their absolute paths:

- `THOUGHT_KHORAL_CODEX_PROVIDER_KEY_FILE`: only the worker receives this file.
- `THOUGHT_KHORAL_CODEX_INVOCATION_KEY_FILE`: only worker and mediator receive it.
- `THOUGHT_KHORAL_CODEX_CATALOG_BRIDGE_KEY_FILE`: only broker and mediator receive it.

Invocation keys must have 16–4096 graphic ASCII bytes. Bridge keys must have
32–4096 characters from letters, digits, `-._~+/=`. The mediator and broker adapt
their own mounted files to the existing environment interface without logging
values. Existing broker workload credentials remain separate.

`THOUGHT_KHORAL_CODEX_STATE_ROOT` identifies separately prepared engine-visible
state directories. `native` and `receipts` must be UID/GID 10003, mode 0700;
`admission` must be UID/GID 10003, mode 0755; `mediator` must be UID/GID 10001,
mode 0700. Account for rootless user-namespace mapping on the container engine.
The worker refuses incorrect ownership/mode, unwritable directories, imported
`config.toml`/`auth.json`, or a mutable instruction directory. Native sessions
and receipt SQLite data persist in different mounts. No host working tree or
personal Codex session is mounted.

Set `THOUGHT_KHORAL_CODEX_ADMISSION_EXPIRES_AT` to a future RFC3339 expiry and
`THOUGHT_KHORAL_CODEX_MODELS` to exact comma-separated native catalog IDs. Set
`THOUGHT_KHORAL_CODEX_MODEL_POLICY_JSON` to the mediator's exact ID-to-effort
object and `THOUGHT_KHORAL_CODEX_POLICY_JSON` to the broker policy (enabled,
policyRevision, guidanceRevision, catalogRevision, default model/effort, models).
Match all three allowlists, use `catalog-1` and `fixed-1`, and use native effort
names in policies. Public opaque effort IDs are produced by the worker.

Use the host preflight wrapper for opt-in operations. It checks all three secret
values differ before handing them to their separate recipients, and refuses a
mutable image reference. The remote opt-in helper runs the same secret preflight
before cloning/building. The raw overlay is the underlying composition artifact;
it does not replace this host-only cross-secret check.

Review interpolation without starting services:

```sh
sh scripts/codex-compose.sh config
```

The existing remote helper retains its four-source default interface. Explicit
opt-in adds `--codex` and requires the fifth immutable commit/tag before any
external build:

```sh
scripts/build-remote.sh --codex GATEWAY_REF MEMORY_REF MEDIATOR_REF UI_REF WORKER_REF
```

That helper builds the worker-owned Containerfile independently and passes the
resulting image ID to Compose. It starts services, so run it only after activation
authorization. `THOUGHT_KHORAL_PLATFORM_ROOT` keeps the three read-only startup
script mounts anchored to the retained platform checkout after temporary source
staging is deleted. Recreate the owner and all dependents as a group after an
isolation failure; restarting only a worker cannot restore the boundary.

## Isolation and admission

The worker is UID/GID 10003, root filesystem read-only, capabilities dropped,
no host port. It shares neither the Reference Agent namespace nor the base
platform network. Its only initiated external-work path is loopback port 3128;
DNS and direct IPv4/IPv6 connections are denied. Worker replies on inbound 9091
connections are allowed; the invocation secret still authenticates every request.
The existing mediator UID 10001 can contact the admitted worker on 9091 and return
catalog bridge responses on broker-initiated 9092 connections. UID 10002 retains
its loopback-only deterministic boundary.

The proxy has UID/GID 10004, no added capabilities, no provider credential, and a
closed `CONNECT api.openai.com:443` policy. It refuses alternate hosts, ports,
IP literals and alternate authority syntax before dialing. It resolves once,
rejects non-public addresses and validates the selected address with a TLS
preflight. The worker then independently validates end-to-end TLS on the actual
tunnel. The preflight does not authenticate a different future TCP connection.
No headers or request bodies are logged or forwarded as proxy HTTP headers.
The tunnel cannot inspect encrypted redirects; another-origin connection is
unusable because its CONNECT authority is denied.

The trusted namespace owner alone has NET_ADMIN. It installs both kernel
policies before exposing readiness, compares complete filter rules each second,
and monitors proxy health. Policy change, owner exit, or proxy loss terminates
the owner PID namespace and its worker/proxy processes. This is Linux PID
namespace fail-stop, not a UID 0 signal-privilege assumption. There is no direct
fallback route.

After package/state checks and authenticated worker-card health, the worker
publishes the closed non-secret UI declaration. The declaration is inside a readiness envelope renewed once per second after
authenticated health, expiring within five seconds. The bootstrap loads it with no
cache, a timeout, strict expiry and field validation. Abrupt owner termination
can leave the file on disk; it ceases to enable admission within five seconds.
Startup removes it before checks, so failed restarts do not refresh old readiness. Missing/unknown fields fail closed;
roster membership/catalog presence cannot enable admission. This declaration
only enables the human controls: the broker remains the authority on current
admission and rejects requests when the worker is unavailable. An already-open
UI keeps its host declaration and relies on those broker checks; expiry prevents
a new cold bootstrap from accepting stale on-disk readiness.

## Verification scope

The [room-conversation verification guide](codex-verification.md) records the
composed provider-free smoke, its limits, and the gated live operator procedure
for the conversation milestone.

- `bash scripts/test-codex-compose.sh`: Compose parser/default/required config.
- `node scripts/test-codex-bootstrap.mjs`: host declaration admission behavior.
- `python3 scripts/test-codex-egress.py`: real loopback proxy parsing and invalid
  upstream TLS tests; these alone do not establish kernel enforcement.
- `python3 scripts/test-codex-egress.py --kernel`: isolated Task8 synthetic Podman
  fixtures, reachable IPv4/IPv6 and DNS controls, restricted real proxy and local
  TLS endpoint, and actual namespace owner/proxy/policy fail-stop.
- `python3 scripts/tests/codex-package.py`: real image startup refusal. Select the
  corrected immutable image with `--image sha256:... --expect-verified` to
  require a passing corrected package gate. Environment overrides remain test-only.
- `python3 scripts/test-codex-egress.py --deterministic`: the existing deterministic
  smoke against isolated Task8 synthetic listeners, including the opt-in worker
  and catalog-response paths.
- Existing remote-build, bootstrap, deterministic egress and Kubernetes checks
  remain required. No fake protocol test establishes live model access, billing,
  provider availability or live conversation behavior.

## Local defaults-discovery candidate

Under the [approved local defaults amendment](../.ai/specs/how/codex-local-service.md#approved-defaults-discovery-amendment--2026-10-07),
settings-capable clients read the authenticated, read-only
`GET /api/agent-conversations/v1/rooms/{roomId}/agents/{agentId}/defaults` route
before their first turn or explicit New session. It returns the broker's
validated next-turn pair with `Cache-Control: no-store`, without allocating a
conversation, task, event, lease or native thread. A restored shared conversation
retains its accepted pair. A later deployment-default-only change preserves a
still-valid explicitly displayed pair.

Model and reasoning-effort controls are independent: an unsupported control is
read-only while the supported control remains usable. Neither capability keeps
the settings-free invocation path and makes no catalog/default requests. Stale
or removed pairs retain the prompt, block invocation and require explicit Refresh
settings; unavailable defaults never fall back to catalog order.

The [candidate verification procedure](codex-verification.md#unreleased-defaults-discovery-candidate)
uses exact local unreleased contract/broker/UI pins and unchanged reviewed
mediator/worker sources. It adds `--fake --defaults-candidate`; existing fake and
live pins remain unchanged. This mode provides synthetic HTTP/jsdom/fake-native
evidence only. Contract publication, packaged-stack validation, live-provider
use and activation remain separately gated.
