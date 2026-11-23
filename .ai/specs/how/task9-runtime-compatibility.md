# Task 9 runtime compatibility preparation

Status: approved for bounded local implementation on 2026-11-23, under the
[root proposal](https://github.com/thoughtkhoral/thought-khoral/blob/main/.ai/specs/how/task9-runtime-compatibility-proposal-2026-11-23.md)
and [execution record](https://github.com/thoughtkhoral/thought-khoral/blob/main/.ai/specs/how/task9-runtime-compatibility-execution-2026-11-23.md).
Record this design before changing implementation. No product What expansion.

Express the existing Codex proxy hostname-to-127.0.0.1 extra-host mapping as
list syntax supported by installed podman-compose 1.6.0. Verify actual generated
Podman argv with an isolated, credential-free dry run: service enumeration alone
does not detect malformed bare --add-host output. Declare the mediator DNS alias
thought-khoral-agent-gateway on the actual namespace owner, agent-egress, on its
existing agent-internal interface. Preserve its Codex internal membership. Add
no network, published port, service authority or admission permission.

Provide a bounded read-only database preflight for actual 0006/0007 conversation
tables, columns and constraints; an empty migration ledger or image init files
cannot prove readiness. Container-local authenticated psql may inspect catalogs
only in a read-only transaction, with connection, statement, lock and process
timeouts and bounded private output. Fail closed with generic diagnostics. The
root orchestrator owns backup, isolated restore, migration verification and any
separate request to upgrade the existing volume. This work never migrates data.

Keep all existing profile bytes, digests and historical fixtures unchanged.
Prepare no new compatibility profile until final exact worker source/image and
shutdown, catalog/admission and database investigations are available. The future
separate packaged-v1.1-runtime-compat runner must invoke schema readiness before
provider requests, bind the actual clean platform commit and the corrected
immutable worker package, and retain all existing activation/provider gates.

Regression tests must demonstrate RED/GREEN generated argv and readiness
failures; run the existing 46 profile/historical guards and relevant host/input/
startup checks. No stack activation, provider/model call, image build, push,
merge, current-stack restart or existing-volume mutation is authorized here.
