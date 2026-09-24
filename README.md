# ThoughtKhoral local MVP platform

MVP / active development. This repository composes the local development stack;
it is not a production deployment distribution.

This independent project builds the
[room gateway](https://github.com/thoughtkhoral/thought-khoral-room-gateway),
[memory engine](https://github.com/thoughtkhoral/thought-khoral-memory-engine),
[agent gateway](https://github.com/thoughtkhoral/thought-khoral-agent-gateway),
and [workspace UI](https://github.com/thoughtkhoral/thought-khoral-workspace-ui),
then composes them with PostgreSQL/pgvector and Keycloak. The default source
build uses checked-out sibling repositories. The stack runs through rootless
Podman without host networking.

The local source-build path expects the platform, room gateway, memory engine,
agent gateway, and UI repositories to be checked out as sibling directories.
See the [local
specification index](.ai/specs/README.md), the [repository
map](https://github.com/thoughtkhoral/thought-khoral/blob/main/docs/repository-map.md),
and the [organization contribution guide](https://github.com/thoughtkhoral/.github/blob/main/CONTRIBUTING.md).

The Compose project is `thought-khoral`. Its services and local image tags are:

| Service | Image |
| --- | --- |
| `thought-khoral-postgres` | `localhost/thought-khoral-postgres:dev` |
| `thought-khoral-keycloak` | `localhost/thought-khoral-keycloak:dev` |
| `thought-khoral-room-gateway` | `localhost/thought-khoral-room-gateway:dev` |
| `thought-khoral-memory-engine` | `localhost/thought-khoral-memory-engine:dev` |
| `thought-khoral-reference-agent` | `localhost/thought-khoral-reference-agent:dev` |
| `thought-khoral-agent-gateway` | `localhost/thought-khoral-agent-gateway:dev` |
| `thought-khoral-agent-egress` | `localhost/thought-khoral-agent-egress:dev` |
| `thought-khoral-workspace-ui` | `localhost/thought-khoral-workspace-ui:dev` |

The stack uses the `thought-khoral-network` network. Its logical
`thought-khoral-postgres-data` volume is an explicit compatibility mapping to
the pre-migration external volume `n2n_postgres-data`, so the identity rename
does not abandon persisted PostgreSQL and Keycloak state.

## Prerequisites

- A running rootless Podman machine
- `podman-compose`
- `curl`
- For full local builds, the room gateway, memory engine, agent gateway, and
  UI repositories checked out beside this platform repository
- For pinned remote builds, immutable commit or release-tag refs for all four
  component repositories and network access to their GitHub remotes

## Start and verify

```sh
podman-compose up --build -d
bash scripts/smoke.sh
bash scripts/validate-kube.sh
```

After changing any locally checked-out component, recreate the service
containers so they use the newly built images:

```sh
podman-compose up --build -d --force-recreate
```

To build from pinned GitHub revisions rather than local sibling directories,
provide one ref for each source-built component:

```sh
sh scripts/build-remote.sh <gateway-ref> <memory-engine-ref> <agent-gateway-ref> <ui-ref>
bash scripts/smoke.sh
```

The helper validates all four staged source manifests before invoking Compose.
Its offline staging test is `sh scripts/test-build-remote.sh`; a live remote
build still requires those revisions to be published and reachable.

Open <http://localhost:8082>. Sign in through the ThoughtKhoral development
realm using one of these local-only accounts:

| Identity | Password | Retained `n2n_role` claim |
| --- | --- | --- |
| `alice` | `alice-dev-only` | `human` |
| `bob` | `bob-dev-only` | `human` |
| `facilitator-agent` | `agent-dev-only` | `agent` |

After login, the document title is `ThoughtKhoral workspace` and the browser
shows the explicit room-entry screen. Enter a valid room UUID, such as
`10000000-0000-4000-8000-000000000001`, and choose **Enter room** to reach the
authenticated collaborative room. The UI uses OAuth 2.0
Authorization Code with PKCE. Its browser socket sends a
`session.authenticate` JSON-RPC request containing the access token as its
first application message; the gateway never receives a token in a URL.

Stop only this Compose stack without deleting its named PostgreSQL volume:

```sh
podman-compose down
```

## Compatibility boundary

ThoughtKhoral is the active product and runtime identity. The existing
`n2n.room.v1` protocol, PostgreSQL database and role `n2n`, physical
`n2n_postgres-data` volume, development-only persisted credential values
`n2n-dev-only` and `n2n-admin-dev-only`, and `n2n_role` OIDC claim remain
unchanged for wire and persisted-data compatibility. Existing database
contents, record fields, and values in that volume remain unchanged as well.
These values are not aliases for new platform resources. The external volume
must already exist; it is created by the pre-migration stack and is
intentionally not deleted by `podman-compose down`.

## Local security boundary

Every password in this repository is intentionally marked `dev-only` and is
limited to the disposable local realm. Do not reuse these values or these
manifests in a production deployment. The tracked files contain no production
credentials. Host ports bind to loopback in Compose, containers cannot gain
privileges, and writable paths are explicit volumes or temporary filesystems.

The Kubernetes files mirror the ThoughtKhoral Compose service and environment
names. The validation script rejects active pre-migration labels, then passes each
workload manifest to `podman play kube --replace --start=false`;
Kubernetes-only Namespace and Service resources remain in the same files for
cluster parity.

## Local A2A reference-agent boundary

The two application services run non-root with read-only filesystems, all
capabilities dropped, no host ports, and no database credentials. The reference
agent uses UID 10002 and binds only `127.0.0.1:9090`; the gateway uses UID 10001.
They share the trusted `agent-egress` container's network and PID namespaces.
That container installs IPv4 and IPv6 default-deny OUTPUT filters before
either application starts and remains running to refresh allowed peer IPs.
The reference agent can use loopback only. The gateway additionally reaches
only the resolved room and Keycloak services on TCP 8080 and the configured
DNS resolver on UDP/TCP 53. Original-destination matching supports Kubernetes
Service DNAT. Compose refreshes the peer allowlist when service IPs change;
failed resolution revokes stale peer access.

Compose uses an internal network shared only with room gateway and Keycloak.
Kubernetes uses a NET_ADMIN init container plus `agent-egress-policy.yaml`;
apply all files in `kube/` with a NetworkPolicy-capable CNI. Adapt the DNS
selector to the cluster's DNS labels when needed. Podman does not enforce
NetworkPolicy, so its kernel filter remains mandatory. Setup failure prevents
application startup. Only the credential-free egress container has NET_ADMIN;
neither agent can change the filter. If the egress PID 1 exits, both agent
processes stop with their shared PID namespace rather than continuing under a
stale allowlist. Recover by recreating the egress owner and both dependent
containers together, then rerun `bash scripts/smoke-agent-egress.sh` and
`node scripts/smoke-agent-gateway.mjs`. The local all-service recovery command
is `podman-compose -f compose.yaml up -d --no-build --force-recreate`.
Kubernetes uses a one-shot egress init container plus CNI NetworkPolicy, not
this Compose PID fail-stop mechanism. No routable A2A Service exists.

The broker owns the five-minute lease. The dispatcher publishes at most three
distinct progress updates and one terminal update per invocation. Polling is
configurable; unsupported lease-duration and progress-rate settings are rejected.

`bash scripts/smoke.sh` authenticates the local Alice fixture, creates a fresh
room, invokes `summarize-context` and `extract-action-items`, and requires the
durable sequence of three progress events followed by exactly one cited terminal
result for each task. Its packet-capture harness creates a hidden Bob-only
targeted message and verifies that the authorized Alice packet excludes it.
After each terminal result, it continues observing the task for two seconds
(eight local polling intervals) and rejects any post-terminal lifecycle event,
including a delayed second success.

`agent-gateway-client-dev-only` is the disposable local Keycloak client
credential; `reference-agent-inbound-dev-only` is the distinct local A2A
inbound bearer secret. The reference-agent receives only the latter, so it
cannot mint gateway workload tokens. Neither is a production secret.
Production deployment must inject managed service credentials and a distinct
inbound secret, then add workload identity and mTLS without changing the room
or A2A authority boundaries. Do not commit production credentials,
certificates, or private keys.
