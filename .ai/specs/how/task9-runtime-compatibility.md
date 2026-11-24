# Task 9 runtime compatibility preparation

## Approved loopback health diagnosis — 2026-11-24

The maintainer approved the root [loopback health-policy amendment](https://github.com/thoughtkhoral/thought-khoral/blob/main/.ai/specs/how/task9-loopback-health-policy-amendment-2026-11-23.md)
for bounded local fixtures and a conditional correction. First reproduce original
kernel-policy teardown and correlate the blocked ACK's state, direction and
socket ownership with nonterminating counter predicates. Permit an exception
only for confirmed socketless, zero-payload pure TCP ACKs on IPv4 127.0.0.1
loopback, ESTABLISHED reply direction with original destination port 3128.
Application-owned socket UID rules, initial connection rules, one-second timeout,
whole-policy comparison and fail-stop behavior remain unchanged. Do not admit
NEW/RELATED/INVALID, other ports/destinations or data-bearing packets. If evidence
does not match, or this predicate cannot be represented safely, stop without
widening and prepare a separate amendment. Existing profiles stay frozen.
Any eligible correction requires actual kernel RED/GREEN and retained negative
egress/fail-stop tests, independent review and new clean platform evidence.
Existing-data upgrades, publication, activation and provider calls remain gated.

The original-policy regression on 2026-11-24 completes three HTTP health reads
and leaves three clients in LAST_ACK and three server TIME_WAIT sockets. Counter-
only rules identify six rejected ACKs matching the approved socketless
ESTABLISHED reply predicate; none match INVALID/UNTRACKED/original direction.
The same six also match a stricter zero-payload representation: IPv4 total length
52, IPv4 header exactly 20 bytes, TCP header exactly 32 bytes, reserved bits zero
and flags exactly ACK. The conditional allow rule is limited to that observed
header form, using length/u32 checks in addition to the approved predicates and
original destination 127.0.0.1:3128. Other header forms stay refused; no general
ESTABLISHED, INVALID or data-carrying exception is permitted.

### Local kernel/package checkpoint — 2026-11-24

Correction source is `a9c3ff5c79e7d3525bed0b5968ab1d8be6b2cc23`. Real-kernel
GREEN completes all three client teardowns (LAST_ACK zero), with a positive
eligible ACK counter delta and zero INVALID/UNTRACKED/original-direction deltas.
Nine real-kernel raw probes show the header predicate matches only the exact empty
timestamped ACK; payload, disguised same-length data and header/options/flags/
reserved-bit variants do not match. All raw packets target refused port 3130;
actual health teardown is the separate allow-rule proof. The raw sender is separately owned, network-none and
NET_RAW-only; it adds no production capability or allow verdict.

The exact local ARM64 proxy/namespace-owner image is
`sha256:54dc5c7d649e486bff7f5f8821050b800ae5cb454cd8496a9f93ce0047d07cf1`,
built from clean correction source above. Source/policy labels and the installed
policy hash `023203915acb11fe9b66183bc65426e52783fa934a89f252c15be5a147ae01dc`
are verified. Its teardown and header tests pass without a mounted override.

The retained real-kernel suite passes IPv4/IPv6/direct-provider/DNS denials,
controlled local verified TLS, proxy-loss/readiness expiry and policy-tamper
fail-stop. All 54 profile/historical/schema/compatibility guards, five Compose,
one input and one startup check pass. Temporary historical inputs had missing
files; exact local commits and the candidate archive/lock were restored without
changing their pins/hashes, preserving previous partial directories privately.
Both frozen profile JSON files remain byte-identical.

An initial actual-watchdog run using the same policy bytes on the unchanged base
image passed 660.003 seconds, then proxy loss stopped namespace and dependent.
The exact packaged image passed 660.005 seconds; proxy loss then stopped its
namespace and dependent. All owned containers were removed. Independent
review finds the runtime bounds correct; the regression now requires positive
eligible and zero forbidden-class deltas rather than merely recording them.
All source/image publication, existing-data upgrade and changed-input activation
gates remain. These checks establish no public-provider or full Task 9 acceptance.

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

## Native adapter package regression

The approved root proposal requires actual production-image adapter evidence.
Add an owned network-none fixture accepting only an immutable image ID. Create
fresh owned engine volumes for native state, receipts and synthetic distinct
credential files; prepare ownership with an isolated setup container, then run
the real worker service as UID/GID 10003 with all capabilities dropped, read-only
root filesystem, no host ports and a bounded admission expiry. Set isolation
verification only inside this owned network-none fixture, never as evidence that
a deployment's firewall is ready. GET only the authenticated control/v1/models
endpoint via container-local bash TCP. Require HTTP 200, normalized closed
catalog shape, gpt-6.1-sol and its exact expected reasoning effort options. Send
no room, thread or turn request and no direct-CLI substitute. Assert fresh
receipts contain no task rows. Always remove all owned containers and volumes;
keep failures generic and report only safe image/catalog evidence. Demonstrate
HTTP 503 on the original immutable image before HTTP 200 on the corrected image.

## Readiness baseline and local verification

The expected catalog metadata was read from the root-owned restored isolated
PostgreSQL clone after the approved 0006/0007 migrations. The broker migration
source is the unchanged frozen `673294e66fed2b6e98d73e08cfe4a23a0e5e4d48`:
0006 SHA-256 `0704c17c443d583c0b60557598ee94945a8380b82e4e0ea146187095a5f6952c`;
0007 SHA-256 `e36a2102a8a09c9d7456831261ea4b5a935fa07d229de4a95722697197ddc766`.
The retained JSON contains only schema metadata: seven required tables,
86 columns, 47 constraints and 16 indexes. Its normalized definitions agree with
the source migrations, including active-conversation partial uniqueness,
deferrable task binding, generation bounds, replay uniqueness and foreign keys.
This deliberately exact PostgreSQL 17 baseline fails closed on structural drift.

Three offline readiness tests pass, including missing/wrong columns/defaults,
constraints and indexes and generic transport/output failures. Actual bounded
authenticated read-only preflight passes on the migrated clone and refuses the
unchanged original database, which lacks conversation tables. Neither check
changes database state. Existing 46 profile/historical guards pass. Compose
RED showed the bare extra-host value and missing owner alias; GREEN passes five
checks with the actual installed generator. Input/startup, host admission,
mediator-egress and four proxy-policy tests pass. Optional historical cleanup
checks cannot reach assertions while the current stack occupies fixed port
8080; the stack and historical fixtures remain untouched.

## Separate compatibility profile binding

After actual adapter evidence, prepare the separately named fixed profile
`packaged-v1.1-runtime-compat`. The approved corrected worker source is
`423d4e84985eb535caa66c7522c4191c6dbccad6`, immutable ARM64 image
`sha256:1ac229b25167845b4d7cec217304f3064b02d65d60c53f35b07c5ae3e3683b3b`.
Preserve every other original profile field and all original profile/fixture
bytes; anchor the new profile's own exact bytes separately. Permit only these
two explicit profile selectors with the existing live opt-in. Cross-profile
operator packets must fail before engine access. The new operator packet must
also specify a closed `postgresContainer` name, validated before any engine
command. Its platform revision remains the actual clean runner HEAD, never a
self-referential embedded commit.

Only the new profile invokes the read-only schema hook before any provider
request, retaining only safe schema readiness evidence. Existing original and
historical runner behavior stays unchanged. Verify source/package/schema
helpers directly against the corrected owned fixture and migrated clone without
activating a live workflow or providing actual human/provider credentials.
Unknown selectors, mixed/cross-profile identities, absent/malformed database
names and missing schema remain refused. The independently reproduced watchdog
health failure remains under root investigation; this profile preparation grants
no changed-input activation or provider authority and closes no Task 9 gate.

Actual production-image adapter regression now demonstrates RED HTTP 503 on
original image `sha256:2dd78cdf5eddb9ca271611998ea44fe6d6cfde6cfca8ba7d1d6c8858240b540d`
and GREEN HTTP 200 on the corrected image above. The real normalized catalog
contains gpt-6.1-sol with low/medium/high/xhigh/max/ultra options (native default
low); configured platform defaults remain gpt-6.1-sol/medium. The corrected
fixture verifies zero conversations/receipts and requests no model turn.
Network-none fixture containers and volumes were removed. During harness
iteration a remote cp layout issue was corrected using safe archive streaming;
an unsupported volume-cleanup flag briefly left six owned synthetic volumes,
which were explicitly removed before the final passing repetition. No current
stack or operator data was affected.

The separate compatibility profile's SHA-256 is
`14c5a784a1f6d8c944f25570f1b0dda741831178348ad7052ecb7301529d2f0c`.
Five new compatibility regressions demonstrated RED before implementation and
now pass, covering independent anchoring, explicit selection, cross-profile
refusal, database-name validation, schema-only-new-profile behavior and actual
CLI synthetic preflight before the terminal gate. All 54 combined
profile/historical/schema/compatibility tests pass, alongside five Compose
checks and the retained input/startup/host-admission/mediator-egress checks.
The original profile remains byte-identical at its original SHA-256. No provider
workflow or changed-input activation was run. Final actual clean source/package/
clone-schema helper binding is recorded separately by root after this commit.
