# Local platform implementation

Follow the [root MVP foundation implementation plan](../../../../.ai/specs/how/n2n-mvp-foundation-implementation-plan.md), the [ThoughtKhoral identity migration design](../../../../.ai/specs/how/thoughtkhoral-identity-migration.md), and the root governance decisions before changing this project.

Implementation begins only after the relevant task is approved. Local composition must remain rootless, use explicit health checks, and keep credentials out of tracked files.

The accepted local [ThoughtKhoral identity decision](../decisions/002-thoughtkhoral-identity.md) renames this project to `thought-khoral-platform`. The `n2n.room.v1` wire value, database identifiers, and persisted values remain unchanged.

## Runtime identity

Compose uses project name `thought-khoral`, services and images named
`thought-khoral-postgres`, `thought-khoral-keycloak`,
`thought-khoral-room-gateway`, and `thought-khoral-workspace-ui`, and the named
volume `thought-khoral-postgres-data`. Build contexts consume the sibling
`thought-khoral-room-gateway` and `thought-khoral-workspace-ui` projects and
their renamed binary and package artifacts.

The gateway receives `THOUGHT_KHORAL_ALLOWED_ORIGINS`,
`THOUGHT_KHORAL_LISTEN_ADDRESS`, `THOUGHT_KHORAL_OIDC_AUDIENCE`,
`THOUGHT_KHORAL_OIDC_ISSUER`, `THOUGHT_KHORAL_OIDC_JWKS_URL`, and
`THOUGHT_KHORAL_SESSION_AUTH_TIMEOUT_MS`. Its entrypoint resolves the JWKS URL
into `THOUGHT_KHORAL_OIDC_JWKS`, which is the variable consumed by the gateway.
The UI bootstrap is served as `thought-khoral-bootstrap.js`, discovers the
application through the `thought-khoral-app-module` metadata name, and provides
authentication and socket adapters through `window.thoughtKhoralWorkspace`.

Kubernetes uses namespace `thought-khoral-dev`, ThoughtKhoral-prefixed resource
names, `app.kubernetes.io/part-of: thought-khoral`, and the
`thought-khoral.redhat.com/environment` label. The identity realm and client
use ThoughtKhoral names. Standard PostgreSQL and Keycloak environment-variable
names remain unchanged because they are external image interfaces.

## Compatibility boundary

The PostgreSQL database and role remain `n2n`, existing database contents stay
in the persistent volume, the OIDC role claim remains `n2n_role`, and
`n2n.room.v1` remains the wire contract. These data and protocol values are
documented exceptions, not active platform identity aliases.

## Verification

`scripts/smoke.sh` rejects legacy Compose service/container identities, waits
for all four services, and requires the ThoughtKhoral browser title.
`scripts/validate-kube.sh` rejects legacy N2N labels before asking rootless
Podman to parse every workload manifest. The release check builds the Compose
stack, signs in to the local realm through the browser entry point, reaches an
authenticated room, and then performs narrowly scoped Compose cleanup without
deleting the persistent PostgreSQL volume.
