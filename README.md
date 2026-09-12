# ThoughtKhoral local MVP platform

This independent project builds the checked-out
`../thought-khoral-room-gateway` and `../thought-khoral-workspace-ui`
repositories and composes them with PostgreSQL/pgvector and Keycloak. The
stack runs through rootless Podman without host networking.

The Compose project is `thought-khoral`. Its services and local image tags are:

| Service | Image |
| --- | --- |
| `thought-khoral-postgres` | `localhost/thought-khoral-postgres:dev` |
| `thought-khoral-keycloak` | `localhost/thought-khoral-keycloak:dev` |
| `thought-khoral-room-gateway` | `localhost/thought-khoral-room-gateway:dev` |
| `thought-khoral-workspace-ui` | `localhost/thought-khoral-workspace-ui:dev` |

The stack uses the `thought-khoral-network` network. Its logical
`thought-khoral-postgres-data` volume is an explicit compatibility mapping to
the pre-migration external volume `n2n_postgres-data`, so the identity rename
does not abandon persisted PostgreSQL and Keycloak state.

## Prerequisites

- A running rootless Podman machine
- `podman-compose`
- `curl`
- The sibling gateway and UI repositories at the paths shown above

## Start and verify

```sh
podman-compose up --build -d
bash scripts/smoke.sh
bash scripts/validate-kube.sh
```

Open <http://localhost:8082>. Sign in through the ThoughtKhoral development
realm using one of these local-only accounts:

| Identity | Password | Retained `n2n_role` claim |
| --- | --- | --- |
| `alice` | `alice-dev-only` | `human` |
| `bob` | `bob-dev-only` | `human` |
| `facilitator-agent` | `agent-dev-only` | `agent` |

After login, the document title is `ThoughtKhoral workspace` and the browser
reaches the authenticated collaborative room. The UI uses OAuth 2.0
Authorization Code with PKCE. Its browser socket sends a
`session.authenticate` JSON-RPC request containing the access token as its
first application message; the gateway never receives a token in a URL.

Stop only this Compose stack without deleting its named PostgreSQL volume:

```sh
podman-compose down
```

## Compatibility boundary

ThoughtKhoral is the active product and runtime identity. The existing
`n2n.room.v1` protocol, PostgreSQL database and role names, database contents,
persisted fields and values, the physical `n2n_postgres-data` volume, existing
development database credential values, and the `n2n_role` OIDC claim remain
unchanged for wire and data compatibility. They are not aliases for new
platform resources. The external volume must already exist; it is created by
the pre-migration stack and is intentionally not deleted by
`podman-compose down`.

## Local security boundary

Every password in this repository is intentionally marked `dev-only` and is
limited to the disposable local realm. Do not reuse these values or these
manifests in a production deployment. The tracked files contain no production
credentials. Host ports bind to loopback in Compose, containers cannot gain
privileges, and writable paths are explicit volumes or temporary filesystems.

The Kubernetes files mirror the ThoughtKhoral Compose service and environment
names. The validation script rejects active legacy N2N labels, then passes each
workload manifest to `podman play kube --replace --start=false`;
Kubernetes-only Namespace and Service resources remain in the same files for
cluster parity.
