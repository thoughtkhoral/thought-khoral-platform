# Local platform implementation

Follow the [root MVP foundation implementation plan](../../../../.ai/specs/how/n2n-mvp-foundation-implementation-plan.md), the [ThoughtKhoral identity migration design](../../../../.ai/specs/how/thoughtkhoral-identity-migration.md), and the root governance decisions before changing this project.

Implementation begins only after the relevant task is approved. Local composition must remain rootless, use explicit health checks, and keep credentials out of tracked files.

The accepted local [ThoughtKhoral identity decision](../decisions/002-thoughtkhoral-identity.md) renames this project to `thought-khoral-platform`. The `n2n.room.v1` wire value, database identifiers, and persisted values remain unchanged.
