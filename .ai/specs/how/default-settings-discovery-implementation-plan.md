# Visible Codex Server Defaults Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Display and explicitly submit the broker's validated next-turn model/effort before initial and New-session invocation, including reasoning-only and model-only admission.

**Architecture:** Add an authenticated, read-only defaults endpoint and a closed schema in an immutable, explicitly unreleased local v1.1.0 candidate. Broker and UI consume that candidate beside their unchanged published v1.0 vendors; the worker and mediator retain their reviewed wire contracts and runtime. The UI binds defaults to its validated catalog and independent capabilities; composed synthetic tests compare the actual rendered selection, submitted request, broker acceptance and native request.

**Tech Stack:** Existing JSON Schema 2020-12/Ajv, Node test runner, Rust/Axum/Tokio/sqlx, React/TypeScript/Vitest/Testing Library, Python standard library for artifact generation, and existing provider-free Podman PostgreSQL fixture.

**Spec:** `/Users/rnaszcyn/Development/rh-n2n/thought-khoral-codex-agent/.ai/specs/how/default-settings-discovery-proposal.md`, approved 2026-10-07; controller-amended root How/plan and local contracts/broker/UI How files. Read `/private/tmp/codex-conversation-defaults/controller-context.md` and `source-bases.json` before execution. Canonical implementation copy belongs to platform `.ai/specs/how/default-settings-discovery-implementation-plan.md`.

## Global Constraints

- "Existing v1.0.0 clients remain compatible with the additive route; existing schemas and fixture bytes remain unchanged."
- "Publication still needs its separate authorization."
- "Use `Cache-Control: no-store` and the current five-second settings-validation bound; no unbounded catalog traversal or retry loop is added."
- "This query creates no conversation, task, native thread, human event or lease."
- "A deployment-default-only change does not make an explicitly submitted still-valid pair stale: the displayed explicit pair continues to govern."
- "Agents with neither settings capability preserve the existing settings-free path."
- "If the authoritative pair cannot be resolved, show an explicit unavailable reason and block invocation."
- Profile remains `thought-khoral.agent-conversation.v1`; API remains `/api/agent-conversations/v1`; candidate proposed release is `thought-khoral-agent-conversation-v1.1.0`.
- No provider calls, activation, new worker images/native capture, release/tag publication, merge, push, or unrelated runtime changes. Local commits only on new isolated branches, never the reviewed source branches.
- Credentials, native session files and private room content stay outside version control and fixtures. Synthetic identities and catalogs only. Preserve Apache 2.0 notices.
- Workers are not alone in the workspace: preserve others' edits and coordinate only through the interfaces below.

## Execution preflight and retained evidence

The controller has created these worktrees from exact reviewed heads and copied approved governance amendments into owning worktrees. Those specific initial documentation changes are expected; do not reset them. Record their hashes before implementation. Dependency order is Task 1, Task 2, Task 3, Task 4; do not start concurrent implementers.

| Owner | New worktree under `/private/tmp/codex-conversation-defaults/` | Reviewed base |
| --- | --- | --- |
| Contracts | `contracts` | `85baf86e574276fcd036e53e23641af6aad602f9` |
| Broker | `room-gateway` | `fd05cb48b8508e7939f9cdf9df275742a06fc4f8` |
| UI | `workspace-ui` | `e51d67e9e1a986601df6b5e1acf68aaf7ae0870d` |
| Platform | `platform` | `637a69279f0fe5019560b1e54d28f48c1c715897` |

Unchanged mediator: `/private/tmp/codex-conversation-final-fix/agent-gateway` at `6c3d96b4763871b9addc9bc7223e71ee7d38abd9`. Unchanged worker: `/private/tmp/codex-conversation-final-fix/worker` at `b0d43ec2b5b0c8da035d4ccff754545132b978d4`. Retain `/private/tmp/codex-conversation-task9/reviewed-checkpoint.json` and all previous worktrees/evidence.

- [ ] Verify exact HEADs and expected-only dirty governance files with `git rev-parse HEAD`, `git status --short` and `git diff -- .ai/specs` in each new worktree. Read its `.ai/specs/README.md` and owning How; stop runtime implementation if approved amendment is missing.
- [ ] Preserve existing baseline logs at the exact reviewed HEADs/lockfiles instead of repeating unchanged full suites before edits. Controller already ran contracts `npm test`: 125 conversation fixtures plus retained suite passed; reviewed UI/broker/platform baseline evidence remains applicable. Use focused RED tests before edits and final changed-owner full suites after implementation. Record actual output/counts; a missing synthetic database is a preflight dependency, not a passing test.
- [ ] Contract publication exception is local candidate testing only. The released lock, release archive and legacy fixture manifests remain unchanged.

## File and interface map

| Task | Production/artifact ownership | Independently reviewable result |
| --- | --- | --- |
| 1 | Contracts new schema, additive fixtures/oracle, reproducible candidate builder | Closed additive contract; immutable local provenance; byte preservation |
| 2 | Broker defaults handler, resolver visibility, candidate schema registration/pin tests | Authenticated bounded read with zero persistence/inference |
| 3 | UI API/hook/controls, capability propagation, candidate verifier | Visible authoritative pair; all 12 supported capability/lifecycle paths |
| 4 | Platform exact candidate/source preflight, composed fixture, verification docs | Real UI HTTP → broker → mediator → worker → fake native pair evidence |

### Task 1: Additive contract and reproducible immutable local candidate

**Files:**
- Create `schemas/agent-conversation-v1/resolved-settings.schema.json`.
- Create `fixtures/agent-conversation-v1.1/manifest.json`, `valid/resolved-settings.json`, and named negative fixtures listed below.
- Modify `test/conversation-validation.mjs` only to add `resolved-settings` semantics.
- Create `test/validate-default-settings-fixtures.mjs`, `test/conversation-candidate.test.mjs`, `scripts/build-conversation-candidate.py`.
- Modify `package.json`, `protocol.md`, `README.md`, approved local How/index as necessary to record implemented candidate status. Do not edit any preexisting schema, fixture, or old fixture manifest.

**Interfaces:**
- New closed response type `ResolvedSettingsView = { profileVersion: "thought-khoral.agent-conversation.v1", roomId: UUID, agentId: UUID, selectedSettings: SelectedSettings }`.
- `GET /api/agent-conversations/v1/rooms/{roomId}/agents/{agentId}/defaults`, no query/body, success 200; error is existing closed `ProfileError`. Map malformed IDs to 400, missing/invalid auth to 401, authority denial to 403, unavailable scope to 404, admission/catalog/configured-pair failure to 503 `runtime_unavailable`. No terminal task failure on a read.
- New validator branch uses existing `semanticErrors(schema, value, trusted = {})`. Trusted context keys: `roomId`, `agentId`, `catalogRevision`, `models: [{ id, efforts: string[] }]`; errors `room-binding`, `agent-binding`, `catalog-stale`, `unsupported-settings`.
- Builder CLI: `python3 scripts/build-conversation-candidate.py --commit HEAD --output /private/tmp/codex-conversation-defaults/artifacts`. Returns/prints candidate directory, exact commit, archive SHA-256, lock SHA-256. It fails on a dirty source or an existing candidate directory. No remote API, release URL or tag creation.

- [ ] **1.1 Write contract RED tests and the additive fixtures.** Keep the original 125-fixture manifest unchanged. New manifest uses its existing `{path, schema, valid, reason?, context?}` case format. Valid fixture:

```json
{"profileVersion":"thought-khoral.agent-conversation.v1","roomId":"22222222-2222-4222-8222-222222222222","agentId":"74686f75-6768-746b-686f-72616c000004","selectedSettings":{"model":"model-b","reasoningEffort":"effort-high","catalogRevision":"catalog-1"}}
```

Negative fixture files: `unknown-field.json` (threadId), `effective-settings.json`, `missing-settings.json`, `null-settings.json`, `partial-settings.json`, `wrong-room.json`, `wrong-agent.json`, `stale-catalog.json`, `unsupported-model.json`, `unsupported-effort.json`, `uppercase-uuid.json`, `overlong-model.json`, `overlong-effort.json`, `overlong-revision.json`, `duplicate-key.json`. Each must fail its named schema/raw-JSON/semantic reason; include a valid case where policy model B is after model A in trusted model data. Runner registers all old schemas plus the new schema using Ajv; `parseStrictJson` remains the raw JSON oracle. It loads only the additive manifest, checks fixture enumeration, and invokes `semanticErrors`.

```js
assert.deepEqual(semanticErrors('resolved-settings', value, {
  roomId: value.roomId, agentId: value.agentId,
  catalogRevision: 'catalog-1',
  models: [{ id: 'model-a', efforts: ['effort-medium'] }, { id: 'model-b', efforts: ['effort-high'] }],
}), []);
assert.ok(semanticErrors('resolved-settings', value, {
  catalogRevision: 'catalog-2', models: [],
}).includes('catalog-stale'));
```

Run `node test/validate-default-settings-fixtures.mjs`. RED must be absent schema/validator or missing semantic rejection, not missing dependency installation.

- [ ] **1.2 Implement only the new schema and semantic branch.** Copy UUID property definitions verbatim from `view.schema.json`; reuse existing settings `$ref` instead of recreating constraints:

```json
{
  "$schema":"https://json-schema.org/draft/2020-12/schema",
  "$id":"https://github.com/thoughtkhoral/thought-khoral-contracts/schemas/agent-conversation-v1/resolved-settings.schema.json",
  "title":"ResolvedSettingsView",
  "type":"object",
  "properties":{
    "profileVersion":{"const":"thought-khoral.agent-conversation.v1"},
    "roomId":{"type":"string","minLength":1,"format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"},
    "agentId":{"type":"string","minLength":1,"format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"},
    "selectedSettings":{"$ref":"turn.schema.json#/$defs/selectedSettings"}
  },
  "required":["profileVersion","roomId","agentId","selectedSettings"],
  "additionalProperties":false
}
```

```js
if (schema === 'resolved-settings') {
  binding(value.roomId, 'roomId', 'room-binding');
  binding(value.agentId, 'agentId', 'agent-binding');
  if (trusted.catalogRevision !== undefined) {
    check(value.selectedSettings.catalogRevision === trusted.catalogRevision, 'catalog-stale');
  }
  if (trusted.models !== undefined) {
    const model = trusted.models.find(model => model.id === value.selectedSettings.model);
    check(model !== undefined && model.efforts.includes(value.selectedSettings.reasoningEffort), 'unsupported-settings');
  }
}
```

Run `node test/validate-default-settings-fixtures.mjs` and `npm test`; append new runner to `npm test` while retaining both original suite commands. GREEN requires both retained and additive fixtures passing.

- [ ] **1.3 Write builder RED tests, then implement reproducible provenance.** Test two builds from one exact clean commit to separate empty directories produce byte-identical TAR and lock; dirty source, changed archive, changed payload, symlink/unexpected file and tampered metadata fail verification. The builder reads committed bytes via `git show`/`git archive`, never a loose working file. Its deterministic archive command is:

```python
commit = subprocess.check_output(['git', 'rev-parse', '--verify', args.commit + '^{commit}'], text=True).strip()
archive = subprocess.check_output(['git', 'archive', '--format=tar', '--prefix=thought-khoral-contracts/', commit])
archive_sha256 = hashlib.sha256(archive).hexdigest()
```

Use full repository `git archive` so the artifact can run its own fixture suites after extraction. In the candidate vendor payload include exact source-relative paths under `schemas/agent-conversation-v1/`, `fixtures/agent-conversation-v1/`, `fixtures/agent-conversation-v1.1/`, and `test/conversation-validation.mjs`, `test/validate-conversation-fixtures.mjs`, `test/validate-default-settings-fixtures.mjs`. Generate a closed `lock.json` with these fields and actual computed values:

```text
profile = thought-khoral.agent-conversation.v1
status = unreleased-local-candidate
proposedTag = thought-khoral-agent-conversation-v1.1.0
repository = https://github.com/thoughtkhoral/thought-khoral-contracts
commit = exact clean committed revision resolved by git
archiveFile = thought-khoral-agent-conversation-v1.1.0-candidate-<full commit>.tar
archiveSha256 = SHA-256 of that exact git archive TAR
archivePrefix = thought-khoral-contracts/
baseTag = thought-khoral-agent-conversation-v1.0.0
baseCommit = 85baf86e574276fcd036e53e23641af6aad602f9
baseArchiveSha256 = 0038fdbf858db013c4a269aeedbca51a5db9128a2f1d6e39754f92ba60fd8f36
files = sorted source-relative path → exact SHA-256 mapping
```

The `<full commit>` part is generated from `commit`, not a literal placeholder in produced files. Do not emit `tag`, `releaseUrl`, `archiveUrl`, `published: true`, or a fake immutable revision. Candidate directory is named by full commit and never overwritten; a changed contract produces a new revision and candidate. Emit lock digest beside, not inside, the lock. Consumers anchor that digest in code; a self-consistent mutable lock is insufficient.

Run `node --test test/conversation-candidate.test.mjs` RED first, then GREEN. The builder test may create its own temporary Git repository/commits so it is independent of uncommitted implementation changes.

- [ ] **1.4 Preserve original bytes and commit reviewed contract source.** Verify every tracked preexisting schema and fixture against the base commit (including manifests), while allowing new files:

```sh
git diff --exit-code 85baf86e574276fcd036e53e23641af6aad602f9 -- fixtures/agent-conversation-v1
node test/validate-default-settings-fixtures.mjs
npm test
git diff --check
```

Additionally enumerate `git ls-tree -r --name-only 85baf86e574276fcd036e53e23641af6aad602f9 schemas fixtures` and compare each committed base blob to current bytes. Require all equal. Review schema, fixtures, semantic oracle and builder before local commit. Commit only explicit owned paths on the new branch with `feat: add resolved defaults contract candidate`.

- [ ] **1.5 Build and verify final artifact after the source commit.** Run builder CLI above; regenerate into a second directory and compare archive and lock bytes. Extract the exact TAR into fresh `/private/tmp` storage, install from its lockfile, and run its `npm test`. Record commit/archive/lock/per-file hashes in `/private/tmp/codex-conversation-defaults/artifacts/candidate-evidence.json`. Hand downstream tasks the exact candidate directory and digests. A later source edit requires a new commit/artifact; never rewrite the reviewed candidate.

### Task 2: Broker authenticated read-only defaults endpoint

**Files:**
- Add `contracts/agent-conversation-v1.1-candidate/lock.json` plus Task 1 exact payload. Leave `contracts/agent-conversation-v1/` unchanged.
- Modify `src/conversation_service.rs`, `src/conversation_protocol.rs`, `src/conversation_store.rs` (resolver visibility only unless focused tests prove a necessary defect).
- Modify `tests/conversation_protocol_test.rs`, `tests/conversation_service_test.rs`, `README.md` and approved owning How implementation record.

**Interfaces:**
- `ConversationPolicy::resolve_settings(&self, settings: Option<&Value>) -> Result<Value, ConversationError>` becomes `pub(crate)`; resolution algorithm stays authoritative and unchanged.
- Existing `validate_catalog_selection(catalog: &dyn CatalogQuery, policy: &ConversationPolicy, selected: &Value) -> Result<(), ConversationError>` remains the bound traversal oracle.
- New Axum handler `async fn defaults(State(state): State<GatewayState>, Extension(actor): Extension<Actor>, Path((room, agent)): Path<(String, String)>, request: Request) -> Response` registered on `/rooms/{room}/agents/{agent}/defaults`, GET and OPTIONS inside `authenticate_browser`.
- `validate_profile_value("resolved-settings", &view)` registers just the new candidate schema; existing released schemas satisfy its `$ref`. Do not register duplicate schema IDs from both vendors.

- [ ] **2.1 Add candidate pin and tamper tests before using the schema.** Copy verified candidate payload from Task 1 and anchor its actual lock digest as a new independent constant in `conversation_protocol_test.rs`. Keep `vendored_files_match_the_published_artifact_lock` intact. New test verifies closed provenance fields, full regular-file set with no symlinks, exact lock hash and all payload hashes, and equal old schema/fixture bytes between the two vendors after translating the existing flattened directory layout. Verification must fail if a candidate file/lock is changed or if an extra file is added. Run `cargo test --locked --offline --test conversation_protocol_test` RED/GREEN around adding verification, using a temporary mutated copy for rejection cases.

- [ ] **2.2 Add HTTP RED coverage using existing `server`, `policy`, `http`, `post` and `TestServer`.** New `PolicyDefaultsCatalog` implements existing `CatalogQuery::page` and returns model A first, model B second, with configured B/high supported. Configure `policy.model = "model-b"`, `policy.reasoning_effort = "high"`, append B to allowlist, and keep revision `catalog-1`. Snapshot counts of `room_events`, `agent_conversations`, `conversation_tasks`, `conversation_updates`, `conversation_update_requests`, `conversation_disclosures`, `room_requests`, and `conversation_tasks WHERE lease_token IS NOT NULL` (migrations `0001_room_events.sql`, `0003_governed_requests.sql`, `0006_agent_conversations.sql`, `0007_conversation_lifecycle.sql`). Query a fresh UUID room with no stored rows. Assert exact closed result and no-store, then unchanged counts and absent conversation:

```rust
let path = format!("/api/agent-conversations/v1/rooms/{room}/agents/{CODEX_AGENT_ID}/defaults");
let (status, value, headers) = http(&server, "GET", &path, Some(&token), &[], None).await;
assert_eq!(status, 200, "{value}");
assert_eq!(value["selectedSettings"], json!({"model":"model-b","reasoningEffort":"high","catalogRevision":"catalog-1"}));
assert_eq!(value.as_object().unwrap().len(), 4);
assert!(headers.contains("cache-control: no-store"));
```

Name tests `defaults_returns_policy_pair_without_writes`, `defaults_auth_scope_and_error_mapping`, `defaults_catalog_and_expiry_bounds`, and `displayed_explicit_pair_survives_default_only_change`. RED command: `cargo test --locked --offline --test conversation_service_test defaults_ -- --test-threads=1`. Initial missing route must fail 200 expectation.

- [ ] **2.3 Implement the endpoint through existing policy and catalog.** Reject any query, malformed canonical UUID, and unavailable agent scope; use existing global authenticated-human room authority without creating membership or requiring a materialized room row. A valid never-used room is readable. Unknown canonical agent scope returns `session_unavailable`/404 on this new route; non-human/denied origin remains 403. Disabled admission or absent bridge returns 503. Invoke current authority/token expiry check before and after catalog await with zero required turn lifetime; do not require a 180-second turn deadline for a read.

```rust
let selected = policy.resolve_settings(None)
    .map_err(|_| ConversationError::RuntimeUnavailable)?;
tokio::time::timeout(
    std::time::Duration::from_secs(5),
    validate_catalog_selection(catalog, policy, &selected),
).await.map_err(|_| ConversationError::RuntimeUnavailable)?
 .map_err(|_| ConversationError::RuntimeUnavailable)?;
let view = json!({"profileVersion":PROFILE_VERSION,"roomId":room_id,
    "agentId":agent_id,"selectedSettings":selected});
validate_profile_value("resolved-settings", &view)?;
```

Hold the current integration read guard while resolving so a policy update cannot mix snapshots. Reuse `authorize_conversation_turn(&actor, token_expiry, Utc::now(), chrono::Duration::zero())` for the existing authority port; no policy mutation, reservation, event, lease or thread operation. Preserve five-second total traversal, 100 pages, at most 100 entries/page, 1 MiB/page, cursor-cycle/duplicate/revision rejection. Do not expose raw catalog/provider diagnostics. Ensure no-store covers every defaults response, including early denied-origin responses; a small middleware response-finalization correction is in scope and must retain existing CORS behavior.

- [ ] **2.4 Complete failure/bounds tests and unchanged-turn regression.** Exercise missing/invalid/expired human token, agent token, rejected origin, malformed ID, unexpected query, unknown agent, disabled policy, absent bridge, empty catalog, removed default model, unsupported default effort, catalog revision mismatch, duplicate model/effort, cyclic cursors, 101st page, oversized page, delayed catalog and expiry during await. All resolution failures return 503 `runtime_unavailable`, closed `ProfileError`, no native/private fields and zero writes; authentication/authority/scope errors retain their specified mapping. For the five-second test, increase only the test HTTP read deadline to seven seconds (its current five seconds races the production five-second bound), and assert measured completion below that deadline. No unbounded sleeps.

Capture returned B/high, change only deployment default to A/medium while preserving allowlist/catalog/disclosure revision, submit explicit captured pair and assert accepted/frozen B/high. Change catalog revision or remove pair and assert rejection before event/task creation; no fallback. Replay an already accepted request after defaults/catalog failure and assert original acceptance/pair unchanged. Retain omitted fresh-New → deployment defaults and omitted continuation → stored shared defaults tests.

Run focused tests GREEN, then `cargo test --locked --offline`, `cargo fmt --check`, `git diff --check` with the disposable database. Update README with the new route and candidate status linked to owning How. Commit explicit owned paths on the new broker branch; hand exact clean HEAD and candidate lock digest to Tasks 3/4.

### Task 3: UI discovery and independent settings capabilities

**Files:**
- Add exact Task 1 payload under `contracts/agent-conversation-v1.1-candidate/`; leave old vendor and old verifier unchanged.
- Create `scripts/check-default-settings-candidate.mjs`, `scripts/verify-default-settings-candidate-checker.mjs`; append their commands to existing `package.json` pretest/prebuild gates.
- Modify `src/features/room/conversationApi.ts`, `useAgentConversation.ts`, `CodexConversationControls.tsx`, `RoomPage.tsx` and their corresponding tests.
- Create `src/features/room/__fixtures__/resolved-settings.json`, `src/features/room/CodexDefaults.composed.test.tsx`; update UI README and owning How implementation record.

**Interfaces:**

```ts
export interface ResolvedSettingsView {
  profileVersion: typeof PROFILE_VERSION;
  roomId: string;
  agentId: string;
  selectedSettings: SelectedSettings;
}
// Returned by createConversationApi:
getDefaults(token: string, roomId: string, agentId: string): Promise<ResolvedSettingsView>;
// Options for useAgentConversation: replace loadModels with independent primitives.
modelSelection?: boolean; // default true for existing direct hook consumers
reasoningEffort?: boolean; // default true for existing direct hook consumers
// Returned by useAgentConversation:
selectedSettings: SelectedSettings | null; // actual displayed next-turn pair
settingsLoading: boolean;
refreshSettings(): Promise<void>;
```

`RoomPage` always passes booleans from validated admission, so omitted capabilities become false. Hook computes `supportsSettings = modelSelection || reasoningEffort`; effect dependencies use primitives. Controls use `selectedSettings` as a single display source. API method checks response `roomId/agentId` match its request and validates the new candidate schema through the existing strict bounded parser/request wrapper (`Authorization`, no-store, no redirects, no credentials).

- [ ] **3.1 Add candidate verifier and API RED tests.** New verifier accepts optional directory argument for temporary tamper tests, pins exact Task 1 lock hash, requires unreleased provenance and complete file set/hashes, and rejects lock/payload/extra-file/symlink changes. It checks the old subset against the unchanged release vendor; existing published verifier stays in pretest/prebuild. API tests return valid fixture, wrong room/agent, wrong profile, partial/null settings, unknown native/private field, effectiveSettings, invalid effort length, duplicate keys, invalid UTF-8, oversized response, 401 and safe 503. Assert GET URL and auth/no-store options. Run `npm test -- src/features/room/conversationApi.test.ts` RED before adding `getDefaults`, then GREEN.

- [ ] **3.2 Write lifecycle/capability regression table before modifying the hook.** Add parameterized hook and rendered `RoomPage` tests for each row below for initial absent conversation, restored ready conversation and explicit New (12 cases). Catalog contains A/medium first, B/high second; defaults endpoint returns B/high. Restored conversation has A/medium while deployment defaults remain B/high. Assert exact shown/submitted pair, appropriate editable/read-only fields, defaults call counts, no inaccessible required selection, and zero ordinary `chat.send` for a Codex submission.

| Model | Effort | Visible controls | Initial / restored / New |
| --- | --- | --- | --- |
| true | true | Both editable | B/high / persisted A/medium / B/high |
| false | true | Read-only model, editable effort | B/high / persisted A/medium / B/high; F1 regression |
| true | false | Editable model, read-only effort | B/high / persisted A/medium / B/high |
| false | false | No settings controls | No models/defaults query; no settings field for all phases |

```ts
it.each([[true, true], [false, true], [true, false], [false, false]])(
  'establishes visible initial settings for model=%s effort=%s',
  async (modelSelection, reasoningEffort) => {
    // Existing fetch mock returns absent view, ordered A/B catalog, B/high defaults,
    // and an acceptance bound to the exact submitted pair.
    const { result } = renderHook(() => useAgentConversation({ ...options, modelSelection, reasoningEffort }));
    await waitFor(() => expect(result.current.isAvailable).toBe(true));
    expect(result.current.needsSettingsSelection).toBe(false);
    if (modelSelection || reasoningEffort) {
      expect(result.current.selectedSettings).toEqual({model:'model-b',reasoningEffort:'effort-high',catalogRevision:'catalog-1'});
    } else expect(result.current.selectedSettings).toBeNull();
  },
);
```

Supply those response mocks in the actual tests; do not infer defaults from the acceptance. For F1 specifically, assert read-only model B is visible, effort selector exists/enabled, selecting a supported effort and sending succeeds on initial and New. RED command: `npm test -- src/features/room/useAgentConversation.test.tsx src/features/room/CodexConversationControls.test.tsx src/features/room/RoomPageCodex.test.tsx`.

- [ ] **3.3 Implement bounded acquisition and a single concrete pair.** After authenticated conversation restore, load the complete bounded catalog if either capability is supported. If conversation is absent, fetch defaults; validate revision/model/effort against the loaded catalog before exposing the pair or enabling submit. For restored conversation use its stored selected model/effort, never call deployment defaults. On New, clear any prior draft, fetch current catalog/defaults, display validated pair, and retain existing reset acknowledgment. Never create a conversation during discovery.

```ts
const pairInCatalog = (pair: SelectedSettings, catalog: CatalogPage): boolean =>
  pair.catalogRevision === catalog.catalogRevision && catalog.data.some(model =>
    model.id === pair.model && model.supportedReasoningEfforts.some(effort => effort.id === pair.reasoningEffort));
const supportsSettings = modelSelection || reasoningEffort;
const settingsReady = !supportsSettings || Boolean(selectedSettings && catalog && pairInCatalog(selectedSettings, catalog));
```

Factor the existing bounded catalog traversal into a local helper, retaining 100-page/repeated-cursor/duplicate-model/revision checks. A defaults/catalog mismatch produces explicit unavailable state and a **Refresh settings** action; this action performs one bounded acquisition, with no recursive retry and no automatic submit. Guard response application by existing room epoch plus a settings-request sequence so delayed New/defaults responses cannot overwrite Continue, another New, a later human edit, Leave or authentication loss. Loading disables submit. Continue restores the persisted pair and invalidates pending New discovery.

For a stored pair whose catalog revision changed, do not silently substitute deployment defaults: show unavailable/review state. On explicit Refresh settings, fetch current conversation/catalog and retain its persisted model/effort only if allowed; bind that reviewed pair to the current catalog revision for the next explicit request. If the pair is removed, remain unavailable. This changes no stored state before acceptance.

Capture the displayed pair before submit's asynchronous refresh. For any supported combination send that complete explicit pair with its catalog revision. Do not reacquire deployment defaults during submit or overwrite a displayed valid pair when only deployment defaults change. Keep existing generation/busy/auth checks and single submit lock. On stale/invalid-settings rejection, retain prompt, mark settings unavailable and expose Refresh settings; no automatic replay/fallback. Neither-capability submission continues omitting settings and catalog/default reads.

- [ ] **3.4 Gate edit actions independently and display the other field read-only.** `selectModel` returns immediately if modelSelection is false; `selectEffort` returns if reasoningEffort is false. Model-only changes always choose that model's catalog-defined default effort, display it before submit and do not expose an editable effort control; null/unsupported catalog default blocks submit. For both capabilities keep existing effort-retention/acknowledgment behavior when changing models. Reasoning-only edits use the resolved/persisted model. Render a visible labeled read-only value for unsupported counterpart when either capability is supported; no hidden control can be required to establish a pair. Initial resolved pair is selected for next turn, with active settings still unconfirmed until runtime evidence.

- [ ] **3.5 Complete stale/race/failure tests and add real-HTTP composed UI test.** Cover unavailable defaults, catalog/default revision mismatch, absent model/effort, null model-only default, 100-page bound, cyclic cursor, room switch/Leave/auth expiry during discovery, New→Continue delayed response, repeated New, changed catalog between display/submit, removed pair, default-only policy change, restored shared settings, immutable accepted pair, prompt retention, ordinary messages, and initial/restore/New for neither capability. All mismatches block until explicit human refresh; no guessed catalog[0]. Add assertions that getter failures never leave a usable stale selection.

Create opt-in `CodexDefaults.composed.test.tsx`, skipped in normal suite unless `RUN_CODEX_DEFAULTS_COMPOSED=1`. Its explicit input environment is `CODEX_DEFAULTS_BROKER_ORIGIN`, `CODEX_DEFAULTS_ROOM_ID`, `CODEX_DEFAULTS_TOKEN`, `CODEX_DEFAULTS_PHASE` (`initial|restored|new`), `CODEX_DEFAULTS_MODEL_SELECTION`, `CODEX_DEFAULTS_REASONING_EFFORT`, `CODEX_DEFAULTS_EVIDENCE_FILE`. All tokens/data are synthetic generated by Task 4. Use the existing `RoomPageCodex.test.tsx` Socket/Enter-room/participant-notification fixture pattern but use actual HTTP fetch for conversation routes and actual `RoomPage`+ChatStream+controls. Wrap real fetch only to record request and clone the real acceptance response; do not replace response content. Read visible next-turn pair from DOM before clicking Send message. For New, click New session and reset acknowledgment first. Write closed synthetic evidence `{phase, capabilities, displayedSettings, submittedSettings, acceptedTurn, defaultsReads, catalogReads}` to the supplied fresh temporary file after one accepted request. Neither capability records null displayed/submitted settings and zero defaults/catalog requests. Leave/cleanup immediately after capture; the platform fixture subsequently drives mediation.

Run focused tests GREEN, `npm test`, `npm run build`, `git diff --check`; commit only owned paths on new UI branch. Report actual test/skip counts (the opt-in composed test skip is expected here), clean HEAD and exact candidate lock digest.

### Task 4: Composed synthetic acceptance, exact pins and derived documentation

**Files:**
- Modify platform `scripts/smoke-codex-conversation.mjs`, `scripts/fixtures/codex-conversation/src/main.rs`, `scripts/tests/codex-conversation-smoke.test.mjs`, `docs/codex-verification.md`, `docs/codex-local.md` where its settings description changes.
- Add `scripts/fixtures/codex-conversation/defaults-candidate-pins.json`, `scripts/tests/codex-defaults-pins.test.mjs`.
- Update canonical plan checkboxes and approved How/index evidence status; preserve existing live operator packet and live pin requirements.

**Interfaces:**
- New runner flag `--fake --defaults-candidate` selects a distinct checked local candidate source set. Existing `--fake` and `--live` source pins remain the retained baseline; live mode explicitly rejects `--defaults-candidate` before reading credentials/starting anything.
- `defaults-candidate-pins.json` is committed after Tasks 1–3 and contains `status: "unreleased-local-candidate"`, exact contract commit/archive/lock digests, exact clean broker/UI revisions and unchanged reviewed mediator/worker revisions. Paths may be overridden by `TASK9_BROKER_REPO`, `TASK9_MEDIATOR_REPO`, `TASK9_WORKER_REPO`, new `TASK9_UI_REPO`, and new `TASK9_CONTRACTS_REPO`; revisions/hashes cannot be overridden by environment.
- Export `sourcePins({ defaultsCandidate = false } = {})` from runner, or a separate small verifier module if existing import side effects require it; validate before opening network listeners/creating a container. Candidate-mode source paths under this amendment directory; baseline defaults remain original reviewed worktrees.
- Fixture receives `TASK9_DEFAULTS_CANDIDATE=1` and exact `TASK9_UI_REPO`; uses Task 3's opt-in UI test and evidence format. Existing crate dependency templates/locks remain unchanged unless dependency resolution actually requires a reviewed adjustment.

- [ ] **4.1 Add pin/source preflight RED tests.** Verify old baseline passes unchanged; candidate mode demands broker/UI same exact Task 1 lock/archive/commit, each old published vendor still matches baseline, worker/mediator remain at their reviewed heads, and all runtime source trees are clean. Reject candidate metadata tamper, changed/new/missing vendor file, fake release URL/status, wrong contract source commit, wrong broker/UI head and dirty source. A missing UI/artifact fails before a container/listener/provider path is touched. A candidate flag combined with `--live` always fails closed. Run `node --test scripts/tests/codex-defaults-pins.test.mjs` RED, then implement and GREEN. Never solve failures by removing exact revision checks or changing live mode to accept candidates.

- [ ] **4.2 Add composed assertions and meaningful mutation controls, then compare real displayed/submitted/accepted/native pairs.** The endpoint and UI already passed Tasks 2–3, so these composed assertions may pass on their first real integrated run. Record that honestly; do not manufacture an absent-endpoint RED. Candidate-mode flag/source-preflight RED belongs to Step 4.1. Add mutation tests proving the comparison rejects changed model/effort, wrong task binding and unexpected defaults requests without modifying production behavior. Extend existing fixture with `async fn defaults_ui_matrix(client: &Client, pool: &PgPool, issuer: &Issuer, gateway_state: &GatewayState) -> Result<()>`. Call only in candidate mode after the retained baseline scenarios and their fixed native-count assertions, before service shutdown. Preserve original baseline/delta, recovery, denial and cleanup scenarios and their assertions; matrix counts are additional separately scoped evidence.

For each of four capability pairs, use a dedicated fresh room and run three UI phases. Configure deployment default B/high with A first in native model catalog. Initial: verify zero rows/turns before and after direct defaults query, then run actual opt-in RoomPage test, read its evidence and mediate the accepted task. Restored: after initial completion change only deployment defaults to A/medium, reload UI, assert shared B/high remains and defaultsReads=0; mediate. New: explicit reset discovers A/medium, acknowledges reset, submits, and creates a distinct native thread. For neither capability all phases omit UI settings and catalog/default requests; assert the broker's normal initial/default and persisted continuation resolution. Use actual safe query helpers and real worker receipts/native JSON capture already present.

```rust
let supported = evidence["capabilities"]["modelSelection"] == true
    || evidence["capabilities"]["reasoningEffort"] == true;
let selected = if supported {
    assert_eq!(evidence["displayedSettings"], evidence["submittedSettings"]);
    assert_eq!(evidence["submittedSettings"], evidence["acceptedTurn"]["selectedSettings"]);
    &evidence["displayedSettings"]
} else {
    assert!(evidence["displayedSettings"].is_null());
    assert!(evidence["submittedSettings"].is_null());
    assert_eq!(evidence["defaultsReads"], 0);
    assert_eq!(evidence["catalogReads"], 0);
    &evidence["acceptedTurn"]["selectedSettings"]
};
// After run_mediator() and exact task-correlated native capture lookup:
let native_model = match selected["model"].as_str().unwrap() {
    "model-a" => "gpt-6.1-sol", "model-b" => "gpt-6-sol", _ => panic!("unexpected synthetic model"),
};
let native_effort = match selected["reasoningEffort"].as_str().unwrap() {
    "effort-medium" => "medium", "effort-high" => "high", _ => panic!("unexpected synthetic effort"),
};
assert_eq!(native["request"]["params"]["model"], native_model);
assert_eq!(native["request"]["params"]["effort"], native_effort);
```

These mappings are the existing synthetic fake_app_server.py model/list IDs and the unchanged worker option-to-native translation, not deployment defaults. The profile selects opaque IDs; native captures use native IDs, so compare through this exact fixture mapping. Correlate capture by task/conversation/thread boundaries, never merely the last response text. For neither capability compare accepted/frozen/native expected server pair; no claim that a hidden pair was displayed.

Run UI child with `npm test -- src/features/room/CodexDefaults.composed.test.tsx` using Tokio child process `.env(...)`, bounded completion, captured logs and inherited existing synthetic cleanup process group. The child evidence path is inside owned temporary fixture state; fail if existing. Do not make the UI test call a provider or use a production token. Native mediation follows the accepted UI submission so the UI child can exit without depending on terminal polling. All phase outputs record pass/fail and exact task ID; failure retains synthetic evidence and cleans owned processes/container.

- [ ] **4.3 Add composed mutation/replay acceptance.** In a separate fresh room, discover B/high then change deployment default alone to A/medium while B/high remains allowed and same catalog revision; submit the displayed explicit B/high and assert accepted/frozen/native B/high, one task/turn. In separate fresh rooms, invalidate catalog revision or remove pair between display and send: rejection has zero new native turns/events/tasks, UI retains prompt and requires explicit refresh, and no A fallback appears. Replay a prior accepted request after catalog/default unavailability returns original acceptance/pair without a native turn. Direct defaults auth/no-store/closed-response tests run through the real broker; full negative/catalog-bound matrix stays in broker component tests.

- [ ] **4.4 Run composed GREEN and retained regression gates.** Execute in candidate platform worktree after exact pin commit dependencies are clean:

```sh
node --test scripts/tests/codex-defaults-pins.test.mjs
node --test scripts/tests/codex-conversation-smoke.test.mjs
node --test scripts/tests/codex-conversation-cleanup.test.mjs
node scripts/smoke-codex-conversation.mjs --fake
node scripts/smoke-codex-conversation.mjs --fake --defaults-candidate
git diff --check
```

The candidate runner records actual contract/broker/UI revisions and archive/lock hashes, unchanged mediator/worker revisions, 12 UI lifecycle results, exact task-bound displayed/accepted/native pair comparisons, immutable legacy byte checks and safe rejection evidence. Existing fixture uses loopback ports 8080/9091/9092 and local postgres:16; check availability before running. Retain no-provider boundaries and cleanup on assertion failure/SIGINT/SIGTERM. Do not claim this is a packaged Compose/browser/real Keycloak/provider test: rendered UI uses jsdom and a synthetic retained socket, while conversation HTTP and broker/mediator/worker/native-adapter path are real with fake native executable.

- [ ] **4.5 Update derived docs and exact checkpoint, then independent review.** Document the defaults route, read-only semantics, independent capability behavior, candidate-mode commands/provenance, explicit stale refresh, and retained publication/package/live gates. Link every changed doc to approved local How. Preserve baseline evidence and explain why mediator/worker stay v1.0 compatible. Record actual logs/counts and all source IDs in `/private/tmp/codex-conversation-defaults/defaults-reviewed-checkpoint.json`; only mark F1/default discovery closed after relevant tests and independent review pass. Run controller root specification/reference/identity gates and worker scaffold `python3 scripts/check_docs.py` plus `git diff --check`. Commit owned platform files locally, and report publication as pending.

## Review gates and stop conditions

Each task receives specification review followed by code/verification review before its result is consumed. Review the exact committed candidate once; later candidate source edits invalidate downstream pins and require a newly identified artifact. This is the approved defaults amendment, not a second broad correction wave. If review uncovers an unrelated preexisting issue, record it with evidence and retain the baseline instead of expanding this work silently. Final controller report separates synthetic acceptance from still-gated contract publication, packaged deployment and live provider acceptance.
