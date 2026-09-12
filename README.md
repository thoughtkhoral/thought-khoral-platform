# N:N local MVP platform

This independent project builds the checked-out `../n2n-room-gateway` and
`../n2n-workspace-ui` repositories and composes them with PostgreSQL/pgvector
and Keycloak. The stack runs through rootless Podman without host networking.

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

Open <http://localhost:8082>. The development realm provides these local-only
accounts:

| Identity | Password | `n2n_role` |
| --- | --- | --- |
| `alice` | `alice-dev-only` | `human` |
| `bob` | `bob-dev-only` | `human` |
| `facilitator-agent` | `agent-dev-only` | `agent` |

The UI uses OAuth 2.0 Authorization Code with PKCE. Its browser socket sends a
`session.authenticate` JSON-RPC request containing the access token as its
first application message; the gateway never receives a token in a URL.

Stop the stack without deleting its named PostgreSQL volume:

```sh
podman-compose down
```

## Local security boundary

Every password in this repository is intentionally marked `dev-only` and is
limited to the disposable local realm. Do not reuse these values or these
manifests in a production deployment. The tracked files contain no production
credentials. Host ports bind to loopback in Compose, containers cannot gain
privileges, and writable paths are explicit volumes or temporary filesystems.

The Kubernetes files mirror the Compose service and environment names. The
validation script passes each workload manifest to
`podman play kube --replace --start=false`; Kubernetes-only Namespace and
Service resources remain in the same files for cluster parity.
