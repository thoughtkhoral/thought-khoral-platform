# Local platform implementation

Follow the [root MVP foundation implementation plan](../../../../.ai/specs/how/n2n-mvp-foundation-implementation-plan.md), the [ThoughtKhoral identity migration design](../../../../.ai/specs/how/thoughtkhoral-identity-migration.md), and the root governance decisions before changing this project.

Implementation begins only after the relevant task is approved. Local composition must remain rootless, use explicit health checks, and keep credentials out of tracked files.

The accepted local [ThoughtKhoral identity decision](../decisions/002-thoughtkhoral-identity.md) renames this project to `thought-khoral-platform`. The `n2n.room.v1` wire value, database identifiers, and persisted values remain unchanged.

## Runtime identity

Compose uses project name `thought-khoral`, services and images named
`thought-khoral-postgres`, `thought-khoral-keycloak`,
`thought-khoral-room-gateway`, and `thought-khoral-workspace-ui`. The logical
volume `thought-khoral-postgres-data` is declared external and resolves to the
existing physical volume `n2n_postgres-data`; this exact legacy name is a
persisted-data compatibility exception, not an active product identity. Build contexts consume the sibling
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

The PostgreSQL database and role remain `n2n`, the physical volume remains
`n2n_postgres-data`, and existing PostgreSQL and Keycloak database passwords
remain `n2n-dev-only`. Existing database contents stay in that volume, the
OIDC role claim remains `n2n_role`, and `n2n.room.v1` remains the wire contract.
These data and protocol values are documented exceptions, not active platform
identity aliases. The Keycloak bootstrap administrator password remains
`n2n-admin-dev-only` because an existing imported realm stores that credential
semantics; changing its environment value would not migrate persisted state.
The ThoughtKhoral realm allocates distinct fixture user UUIDs because Keycloak
stores user primary keys globally across realms. Reusing the legacy realm's
fixture UUIDs would collide during import; allocating new rows leaves every
legacy realm row and previously persisted room event unchanged.

## Verification

`scripts/smoke.sh` rejects legacy Compose service/container identities and
waits for all four services. It then launches `scripts/browser-smoke.mjs`,
which starts with an empty in-memory cookie jar, performs OAuth 2.0
Authorization Code with PKCE as the documented `alice` development fixture,
exchanges the returned code, opens the UI WebSocket with the browser Origin,
sends `session.authenticate`, and joins the fixture room through
`n2n.room.v1`. The probe succeeds only when the token actor and room replay
responses prove an authenticated connected room; access and refresh tokens are
never printed.

`scripts/validate-kube.sh` scans every N2N occurrence and allows only exact
wire, claim, database identifier, persisted credential, and database URL
forms. `scripts/test-validate-kube.sh` copies the real manifests into an
isolated fixture and proves that an active legacy label value such as
`app.kubernetes.io/name: n2n-gateway` is rejected before Podman parsing. The
release check then asks rootless Podman to parse every workload manifest and
performs narrowly scoped Compose cleanup without deleting the external
PostgreSQL volume.
