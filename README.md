# ThoughtKhoral local MVP platform

MVP / active development. This repository composes the local development stack;
it is not a production deployment distribution.

This independent project builds the
[room gateway](https://github.com/thoughtkhoral/thought-khoral-room-gateway),
[agent gateway](https://github.com/thoughtkhoral/thought-khoral-agent-gateway),
and [workspace UI](https://github.com/thoughtkhoral/thought-khoral-workspace-ui),
then composes them with PostgreSQL/pgvector and Keycloak. The default source
build uses checked-out sibling repositories; `scripts/build-remote.sh` also
builds from pinned GitHub revisions. The stack runs through rootless Podman
without host networking.

The local source-build path expects the platform, room gateway, agent gateway,
and UI repositories to be checked out as sibling directories. See the [local
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
| `thought-khoral-workspace-ui` | `localhost/thought-khoral-workspace-ui:dev` |

The stack uses the `thought-khoral-network` network. Its logical
`thought-khoral-postgres-data` volume is an explicit compatibility mapping to
the pre-migration external volume `n2n_postgres-data`, so the identity rename
does not abandon persisted PostgreSQL and Keycloak state.

## Prerequisites

- A running rootless Podman machine
- `podman-compose`
- `curl`
- For local builds, the room gateway, agent gateway, and UI repositories checked
  out beside this platform repository
- For remote builds, immutable room-gateway, agent-gateway, and UI commit or
  release-tag refs

## Start and verify

```sh
podman-compose up --build -d
bash scripts/smoke.sh
bash scripts/validate-kube.sh
```

After changing a locally checked-out room gateway, agent gateway, or UI, recreate the service
containers so they use the newly built images:

```sh
podman-compose up --build -d --force-recreate
```

To build from GitHub sources instead of local sibling directories, provide one
ref for each component:

```sh
sh scripts/build-remote.sh <gateway-ref> <agent-gateway-ref> <ui-ref>
bash scripts/smoke.sh
```

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

The local A2A vertical slice runs two additional non-root, read-only services.
Neither publishes a host port or receives `DATABASE_URL`. The reference agent
binds only `127.0.0.1:9090`; the agent gateway shares its network namespace in
Compose and its Kubernetes pod as a sidecar, so the pinned Agent Card stays
loopback-only rather than becoming a routable service. This intentionally
replaces a Kubernetes `Service`: the sidecar is the only way to preserve the
reviewed loopback authority without publishing a ClusterIP endpoint.

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
