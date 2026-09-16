# Global Preferences Product-Surface and Project-Genesis Contract

Status: Owner-approved GP-CM03 implementation contract. GP-CM03 execution is
authorized through the synchronized Frontier; new dependencies, GP-CM04,
GP-CM05, and production acceptance remain unauthorized.

Tracker: `dat-global-preferences-completion-f84`
Frontier: `GLOBAL-PREFERENCES-COMPLETION / GP-CM02R`
Mechanism: Product Mechanics 037, amended by Product Mechanics 038 through 040

<!-- CONTRACT:GLOBAL-PREFERENCES-COMPLETION:GP-CM03 -->
<!-- REQ:GLOBAL-PREFERENCES-COMPLETION:GP-CM02R -->

## 1. Purpose and authority boundary

This contract closes the nine GP-CM03 build-readiness findings. It defines the
one engine-owned machine-preference product service, its typed public requests,
the exact CLI and MCP adapters, proposal safety, the only Project seed allowed
in this slice, Project-genesis atomicity, stable failures, and completion proof.

It does not activate a descriptor. GP-CM03 operates only on the eleven entries
in `active_v1_registry()`: three Appearance preferences and eight Units defaults
for future Projects. The other 45 identities remain reserved candidates. A
reserved identity is invalid on every product request even when its schema is
known by `reserved_v1_registry()`.

The active key inventory is exact and ordered:

1. `datum.console.feedback_duration`
2. `datum.accessibility.reduced_motion`
3. `datum.accessibility.high_contrast_noncolor`
4. `datum.units.system`
5. `datum.units.board_length`
6. `datum.units.board_length_precision`
7. `datum.units.drill_hole`
8. `datum.units.drill_hole_precision`
9. `datum.units.schematic_geometry`
10. `datum.units.schematic_geometry_precision`
11. `datum.units.angle_precision`

Items 4–11 are the exact ordered Project Units seed inventory. Their typed
value tokens, labels, defaults, validators, and resolved Automatic/follow-system
behavior come only from the active descriptor catalog and the shared Units
engine; this contract does not duplicate or loosen them.

Global preference writes mutate only `PreferenceRepository`. They are not
`DesignModel` operations and never enter a Project journal. The one exception is
copy-once Project genesis: the eight resolved Units values become Project-owned
`ProjectDisplayUnits` through the engine's canonical genesis transaction. After
publication there is no live link to Global Preferences.

## 2. One engine-owned application service

`GlobalPreferencesProductService` is the only product entry point. It owns one
`GlobalPreferencesService`, the active catalog, repository inspection, query,
mutation planning, proposal validation, and Units-seed resolution. GUI, CLI,
daemon, and MCP code may serialize requests and render responses but may not:

- instantiate `PreferenceRepository` directly;
- parse or write repository manifests, head files, generations, receipts, or
  unknown envelopes;
- call `resolve_preference()` with a separately assembled source set;
- duplicate descriptor defaults, enum inventories, labels, aliases, section
  placement, search logic, or refusal mapping;
- read environment variables to create a rival preference precedence layer; or
- create a Project Units profile except from a product-service seed response or
  the explicitly requested factory profile.

The service root comes from one `PreferenceLocationProvider`. The installed
Linux provider resolves `$XDG_CONFIG_HOME/datum/preferences` when
`XDG_CONFIG_HOME` is set to an absolute path, otherwise
`$HOME/.config/datum/preferences`. An invalid or unavailable base returns
`repository_io`; it never falls back to a working directory. Tests may inject a
temporary provider. Product CLI flags and MCP arguments cannot select an
arbitrary repository path. `DATUM_GUI_PREFERENCES_PATH` is read only as the
legacy single-file migration source already governed by GP-F01; it cannot
relocate the repository. A future administrative relocation operation belongs
to Manage Preferences and is not GP-CM03.

The repository's persisted identity, not a caller path string, defines the
idempotency and writer domain. Directory creation uses user-private platform
permissions and rejects a root that resolves outside the selected configuration
base through a symlink. Existing more-restrictive permissions are preserved;
Datum never makes them broader. These location rules are shared by the daemon
and standalone CLI provider.

### 2.1 Process and writer ownership

When a Datum engine daemon owns the local repository, GUI and MCP calls and any
CLI invocation that discovers that daemon use its typed RPC. The daemon owns the
single long-lived product service and writer instance. A CLI invocation with no
reachable owning daemon may construct the product service in-process, but the
same repository writer lease and expected-generation checks apply. It never
falls back from a reachable daemon refusal to a second local writer.

MCP is always a daemon adapter. The Python MCP bridge never opens the repository
or Project files. Two daemons or a daemon and standalone CLI racing for the same
root are ordinary competing writers: one commits and the other receives
`writer_conflict` or `stale_generation`; neither retries invisibly.

The repository uses one permanent user-private `writer.lock` file and a
nonblocking kernel advisory exclusive lock held on its open descriptor for the
whole transaction. File existence alone never means “locked”; descriptor close
or process death releases ownership, so a crash cannot leave an unprovable stale
sentinel. Readers use immutable generations without this lock. A filesystem
that cannot provide the required local lock and atomic-rename semantics returns
`repository_io` and cannot mutate. Network/shared-folder multi-writer mode is
not supported. After acquiring the lock, every writer re-reads and validates
the full head expectation before staging.

## 3. Versioned common envelope

Every engine adapter uses `PreferenceProductRequestV1` and returns
`PreferenceProductResponseV1`. JSON-facing surfaces use this shape:

```text
PreferenceProductRequestV1 {
  schema: { name: "datum.preferences.<verb>", version: 1 },
  payload: verb-specific request
}

PreferenceProductResponseV1 {
  ok: bool,
  schema: { name: "datum.preferences.<verb>", version: 1 },
  context: PreferenceContextV1,
  result?: verb-specific result,
  error?: PreferenceErrorV1
}

PreferenceContextV1 {
  scope: "global_this_device",
  repository_status: "defaults_only" | "ready" |
                     "preserved_unreadable" | "migration_required",
  generation: GenerationRef?,
  active_catalog_digest: sha256,
  active_descriptor_count: 11,
  reserved_descriptor_count: 45
}
```

Exactly one of `result` and `error` is present. `GenerationRef` is the existing
engine type in full; clients may not reduce it to a generation number. Reads
report the generation actually inspected. A defaults-only read reports `null`.
Mutation requests use `HeadExpectationV1`, exactly one of:

```text
missing
generation(GenerationRef)
```

An unreadable digest is recovery authority and is not accepted by GP-CM03
setting mutations.

Canonical digests use Datum's existing canonical-JSON and SHA-256 utilities;
adapters never recalculate them. `active_catalog_digest` covers the ordered
active section and descriptor presentation schema (key, descriptor schema
version, section/order, control kind, option tokens and labels, default, and
effect timing). `proposal_digest` covers every `PreferenceProposalV1` field
except the digest itself. `genesis_request_digest` covers normalized Project
name and requested identity, normalized creation options, Units source, and the
pinned seed digest; the destination path and request id are deliberately
excluded. `units_seed_catalog_digest` covers only the ordered eight active Units
descriptor schemas and resolver recipes, never Appearance labels.
`seed_application_digest` covers Project id, the eight ordered copied values,
their descriptor schema versions, source reference, and Units-seed catalog
digest. Every digest domain includes its schema name and version.

Actor, session, repository identity, and authorization facts are trusted
transport context and never fields accepted from `payload`. Unknown request
fields in a supported schema are refused as `invalid_request`; an unsupported
schema name/version is `unsupported_schema_version`. Clients cannot smuggle a
future authority field into V1. Response fields are additive only within V1;
changing a field's meaning, removing it, or adding a new operation variant
requires a new schema version.

## 4. Exact query inventory

`PreferenceQueryV1` has exactly six variants:

| Variant | Public name | Required input | Result |
|---|---|---|---|
| `Describe` | `datum.preferences.describe` | none | active sections, controls, enum labels, defaults, timing, catalog digest |
| `List` | `datum.preferences.list` | optional active section id | ordered active `PreferenceValueViewV1[]` |
| `Get` | `datum.preferences.get` | active `PreferenceKey` | one value view |
| `Search` | `datum.preferences.search` | nonempty UTF-8 query | ordered matches plus match kind |
| `Explain` | `datum.preferences.explain` | active `PreferenceKey` | complete `PreferenceExplanationV1` |
| `PreviewProjectUnitsSeed` | `datum.preferences.preview_project_units_seed` | `ProjectUnitsSourceV1` and optional expected generation | validated pinned seed preview and receipt preview; no write |

`PreferenceValueViewV1` contains key, human label, description, section id and
label, row order, control presentation, effective value, optional User value,
effect timing, writable state, provenance summary, and the inspected generation.
It never reports a reserved descriptor as an unavailable setting.

Search invokes `PreferenceSurfaceCatalog::search()` over active entries only.
Its `match_kind` is `label`, `description`, `stable_key`, or `registered_alias`.
Key/alias-only results disclose the matching vocabulary. Search never changes
state, never writes migration acknowledgment, and never indexes reserved keys or
aliases.

`Describe` and `Search` remain available from the active catalog when the
repository is unreadable or needs migration. `List`, `Get`, and `Explain`
return the session's safely resolved view with `writable:false`: unreadable
state uses descriptor defaults plus independently valid sources and discloses
the preserved unreadable digest; migration-required state exposes only values
the registered reader can interpret and marks every unresolved retained value.
They never claim those values were stored. Mutation refuses with the matching
repository code. Global seed preview/genesis also refuses; only an explicitly
new factory-mode request can proceed independently.

`PreferenceExplanationV1` is a versioned serialization of the engine resolver's
one explanation result. It includes the effective value and reason, every
eligible source in precedence order including explicit absences, retained value,
control/directive state, disclosure/redaction, writable state, effect timing,
and generation. Adapters cannot summarize away a refusal or invent a source.

The six result variants are exact: `DescribeResultV1 {sections, controls,
active_catalog_digest}`, `ListResultV1 {values}`, `GetResultV1 {value}`,
`SearchResultV1 {query, matches}`, `ExplainResultV1 {explanation}`, and
`ProjectUnitsSeedPreviewV1 {source, pinned_generation, profile,
receipt_preview, units_seed_catalog_digest, seed_application_digest}`. A search match contains one value
view, one `match_kind`, and optional `matched_vocabulary`; the optional field is
required for stable-key or alias matches and absent for label/description
matches. A receipt preview has no Project id, creation actor, or genesis digest
and cannot be submitted as a receipt.

## 5. Exact mutation and Reset inventory

`PreferenceMutationRequestV1` has exactly two direct variants:

```text
SetUser {
  key: active PreferenceKey,
  value: typed JSON value,
  expected: HeadExpectationV1,
  request_id: UUID,
  reason: nonempty string
}

ResetUser {
  key: active PreferenceKey,
  expected: HeadExpectationV1,
  request_id: UUID,
  reason: nonempty string
}
```

`SetUser` validates the value through the active descriptor before planning the
one User-partition mutation. `ResetUser` removes the User contribution. Reset
never writes the descriptor default, copies an Installation value, changes an
unknown envelope, or touches a Project. Both commit through the same
`GlobalPreferencesProductService::commit()` path and return the resulting full
`GenerationRef`, affected value view, resolver explanation, and repository
`MutationReceipt`.

The first `SetUser` against a missing repository uses expectation `missing` and
publishes generation zero containing that value in one atomic repository
commit. It must not publish an empty generation followed by a second
generation. `ResetUser` against a missing repository is a successful
`changed:false` no-op: it creates no directory, head, generation, or receipt.
After validating the expectation, setting the already-identical User value or
resetting an already-absent User value is likewise `changed:false` with no new
generation or receipt. A no-op never skips authorization, schema, repository,
or expected-generation validation.

Direct mutation is accepted only from `human_gui` and `human_cli` actors.
`mcp_agent`, `script_agent`, and unknown actors receive `proposal_required`.
This rule cannot be overridden by a request field or ambient environment
variable.

`human_gui` exists only for a focused local control activation in the owned
Preferences window. `human_cli` exists only when the CLI owns a foreground
controlling terminal and the person confirms the fully rendered key, current
value, proposed value/Reset, scope, and effect timing through `/dev/tty` (not
request stdin). GP-CM03 provides no `--yes`, environment, pipe, or config bypass.
A headless/noninteractive CLI caller is `script_agent` even when it runs under
the same operating-system user and must use proposals. Failure to establish the
interactive human ceremony returns `human_presence_required` without writing.

### 5.1 Actor, provenance, idempotency, and audit

```text
PreferenceActorV1 {
  kind: "human_gui" | "human_cli" | "mcp_agent" | "script_agent",
  session_id: nonempty stable session identity,
  local_actor_id: nonempty local identity or "unavailable",
  invocation_id: UUID
}
```

The trusted adapter derives `kind` from its authenticated local entry point;
caller JSON cannot supply or override it. The repository receipt records
operation, actor kind/id/session,
invocation and request ids, reason, expected and resulting full generations,
affected keys, and before/after digests or redaction. No secret value is copied
into a log message.

`request_id` is idempotent within one repository identity. Repeating the same
canonical request against the same expected generation returns the original
receipt and result without a new generation. Reusing it with different canonical
content returns `idempotency_conflict`. GP-CM03 extends the repository receipt
index to preserve this fact across restart; a process-local cache is insufficient.

Mutation success is exactly `PreferenceMutationResultV1 {changed, generation?,
value, explanation, receipt?}`. The receipt is required when `changed:true` and
absent for every validated no-op. The canonical request used for
idempotency contains operation, key, typed value if any, expectation, reason,
and trusted actor identity; it excludes transport framing.

## 6. Exact proposal inventory and security

`PreferenceProposalV1` proposes exactly one `SetUser` or `ResetUser` payload and
contains proposal schema/version, proposal UUID, canonical proposal digest,
prepared-against full `GenerationRef` or `missing`, active catalog digest,
mutation, requesting actor, rationale, and creation session. It is a
side-effect-free portable value, not a repository contribution and not a Project
proposal. Preparing, inspecting, validating, rejecting, or letting one expire
does not write the preference store.

The proposal family has exactly four actions:

| Action | Effect |
|---|---|
| `Prepare` | validate one proposed mutation against current active authority; write nothing |
| `Validate` | revalidate digest, schema, active key, value and generation; write nothing |
| `AcceptAndApply` | trusted human GUI/CLI acceptance and commit as one service call |
| `Reject` | return a nonpersistent rejection result; write nothing |

Their results are `PreparedProposalResultV1 {proposal, current_value,
explanation}`, `ProposalValidationResultV1 {valid, proposal,
current_generation, explanation}`, `AcceptedProposalResultV1 {proposal_id,
mutation_result, acceptance_id}`, and `RejectedProposalResultV1 {proposal_id,
rejected:true}`. Preparing or validating never returns an acceptance handle.
The handle is issued only by the separate trusted human acceptance event; the
daemon's session broker attaches it to the next matching `AcceptAndApply` from
the originating MCP session. It is never a public tool argument.

That event has one typed host-only request,
`AuthorizeMcpPreferenceApplyV1 {proposal_id, proposal_digest,
originating_mcp_session}`. The owned GUI action or foreground-TTY CLI renders
the exact proposed change and invokes it after human confirmation. The result
is `McpPreferenceAuthorizationResultV1 {proposal_id, authorized:true,
expires_at}`; it deliberately omits the opaque handle. The daemon delivers the
authorization only into the named MCP session's private broker slot. This
broker request is not an MCP tool and never commits by itself.
It refuses unless that open local session prepared the identical proposal and
is owned by the same local Datum daemon; a typed session id alone grants
nothing.

There is no rebase. A stale proposal is refused and must be prepared again so
the human sees the new effective value and explanation.

MCP may call `Prepare` and `Validate`. It may call `AcceptAndApply` only when the
bridge can attach a
single-use opaque `PreferenceAcceptanceHandleV1` minted server-side after a
trusted local GUI/CLI human accepts the exact proposal. It is a daemon-local
capability, not caller-authored JSON and not a portable credential. The daemon
binds it to proposal digest, repository identity, prepared generation,
accepting human actor/session, originating MCP session, and one invocation. It
expires after five minutes or session close, whichever occurs first; daemon
restart invalidates every unconsumed handle. Only the originating MCP session's
bridge can retrieve the opaque id from its broker slot. MCP clients cannot
mint, refresh, transfer, or inspect its binding. The handle id is transport context excluded from the public MCP input
schema and from model-visible output.

Handle validation and repository commit form one service transaction. The
durable mutation receipt records the acceptance id, so a crash after commit
cannot permit replay even though daemon memory is lost. Success consumes the
handle. A refusal does not broaden or retarget it. This bounded local capability
uses existing process memory, UUID, receipt, and digest facilities; GP-CM03
adds no cryptographic or credential dependency. There is no MCP direct
`SetUser` or `ResetUser` tool. Persistent unattended preference authority
remains excluded pending its separate security decision.

For an accepted proposal the audit receipt records both the requesting agent
and accepting human actor/session plus proposal, acceptance, invocation, and
request ids. Retry checks the durable request-id index before live-handle state:
the identical committed request returns its original result after response loss
or restart; a different request using that consumed handle returns
`acceptance_consumed`, and changed request content returns
`idempotency_conflict`. No retry re-executes the mutation.

## 7. Stable refusal inventory and surface mapping

`PreferenceErrorV1` contains stable `code`, human message, safe structured
details, current context, and preserved draft/proposal where applicable. The
exact GP-CM03 codes are:

| Code | Meaning and required preservation |
|---|---|
| `invalid_request` | required fields missing, unknown fields present, or mutually exclusive inputs combined |
| `invalid_query` | empty search or malformed section/query input |
| `unknown_section` | section id is not one of the two active sections |
| `unknown_preference_key` | key is neither active nor recognized as reserved |
| `reserved_preference_key` | known reserved identity; never activated by request |
| `invalid_preference_value` | descriptor validation failed; preserve submitted draft |
| `ineligible_source` | requested source/partition is not allowed |
| `generation_required` | mutation omitted its required expectation |
| `stale_generation` | head differs; preserve draft/proposal and return current generation |
| `writer_conflict` | exclusive lease unavailable; no retry or write |
| `repository_unreadable` | preserve unreadable bytes; mutation disabled |
| `migration_required` | read remains inspectable; mutation disabled until governed migration |
| `repository_io` | no commit acknowledged; return safe operation stage |
| `unauthorized_actor` | actor cannot perform the requested class |
| `human_presence_required` | direct CLI write lacks foreground TTY confirmation |
| `proposal_required` | nonhuman direct mutation refused |
| `proposal_invalid` | proposal schema, digest, key, or value invalid |
| `proposal_stale` | prepared generation/catalog no longer current |
| `missing_acceptance` | apply lacks a valid human acceptance handle |
| `acceptance_mismatch` | handle does not bind this proposal/context/session |
| `acceptance_expired` | handle expired, its session closed, or daemon restarted |
| `acceptance_consumed` | durable receipt proves single-use acceptance already committed |
| `idempotency_conflict` | request id was reused with different content |
| `seed_source_unavailable` | requested Global source cannot yield eight valid values |
| `seed_incomplete` | seed/receipt is not the exact eight-key profile |
| `project_target_exists` | genesis destination already exists |
| `genesis_publish_failed` | staged Project was not published; destination remains absent |
| `unsupported_schema_version` | request, proposal, receipt, or repository version unsupported |

Required `details` fields are stable: request/query refusals carry `{field,
reason}`; key/section refusals carry `{key}` or `{section}`; value and
source refusals carry `{key, reason}` and preserve the submitted value only in
the response's protected draft; generation failures carry `{expected,
current}`; writer failures carry `{repository_identity}`; repository failures
carry `{repository_identity, stage, preserved_digest?}`; actor failures carry
`{actor_kind, required_authority}`; proposal failures carry `{proposal_id,
proposal_digest, prepared_generation, current_generation?}`; acceptance
failures carry `{acceptance_id?, proposal_id, state}`; idempotency failures
carry `{request_id, original_request_digest, submitted_request_digest}`; seed
failures carry `{units_source, missing_or_invalid_keys}`; Project-target
failures carry `{request_id, destination_identity}`; and schema failures carry
`{schema_name, supported_version, received_version}`. No filesystem path,
secret value, raw unreadable byte, or unredacted managed value is disclosed.

JSON CLI and MCP return the same symbolic code and detail-field names. Human CLI
text begins with the symbolic code. CLI exits `0` on success and `2` for every
typed refusal, preserving Datum's current command convention; automation uses
the symbolic code, not an inferred exit-code taxonomy. Canonical MCP tools
return `ok:false` in the standard envelope; JSON-RPC transport errors are only
for malformed transport calls or unavailable transport.

## 8. Exact CLI and MCP inventory

CLI group: `datum-eda preferences`.

| CLI command | Canonical MCP tool | Authority |
|---|---|---|
| `describe` | `datum.preferences.describe` | read |
| `list [--section <id>]` | `datum.preferences.list` | read |
| `get <key>` | `datum.preferences.get` | read |
| `search <query>` | `datum.preferences.search` | read |
| `explain <key>` | `datum.preferences.explain` | read |
| `set <key> --value-json <json> --expected <ref> --reason <text> --request-id <uuid>` | none | foreground-TTY human confirmation, then direct write |
| `reset <key> --expected <ref> --reason <text> --request-id <uuid>` | none | foreground-TTY human confirmation, then direct Reset |
| `proposal prepare --request-json <file-or-stdin>` | `datum.preferences.proposal.prepare` | no write |
| `proposal validate --proposal-json <file-or-stdin>` | `datum.preferences.proposal.validate` | no write |
| `proposal accept-apply --proposal-json <file-or-stdin>` | `datum.preferences.proposal.accept_apply` | human CLI applies directly; MCP requires its delivered acceptance handle |
| `proposal authorize-mcp --proposal-json <file-or-stdin> --mcp-session <id>` | none | foreground-TTY human confirmation; mint and privately deliver one handle; no write |
| `proposal reject --proposal-json <file-or-stdin>` | `datum.preferences.proposal.reject` | no write |
| `preview-project-units-seed --source global\|factory [--expected <ref>]` | `datum.preferences.preview_project_units_seed` | read |

All accept `--format json`; commands that have an MCP twin are
byte-schema-equivalent after removal of transport framing. Text is a rendering
of the same response. Direct CLI `set` and `reset` deliberately have no MCP
twin: schema parity does not grant equal mutation authority. MCP agents use the
same mutation payload inside the proposal lifecycle. MCP tool schemas are
generated from the engine request types or compared field-for-field by a parity
test; the Python bridge contains no handwritten semantic defaults.

## 9. Project Units genesis modes

`ProjectGenesisRequestV1` is exact:

```text
ProjectGenesisRequestV1 {
  request_id: UUID,
  destination: platform path,
  project_name: nonempty string,
  project_id?: UUID,
  units_source:
    global { expected_generation: GenerationRef? } |
    factory { profile_id: "datum.units.factory.v1" }
}
```

Adapters may default name from the destination basename and may generate
`project_id` and `request_id`, but they must show/return the normalized values
and submit this complete request. Explicit identities are available for
reproducible automation and tests.

GUI New Project defaults visibly to `global`. CLI `datum-eda project new`
defaults to `--units-source global` and reports that normalized choice. Canonical
MCP `datum.project.new` requires `units_source`; omission is invalid. Its two
accepted values are `global {expected_generation?}` and
`factory {profile_id:"datum.units.factory.v1"}`—the same typed variants the GUI
and CLI submit after their defaults are made explicit. All three call the same
engine genesis service and return `ProjectGenesisResultV1 {project_id,
request_id, genesis_request_digest, project_root_identity, units_receipt,
published_manifest_digests}`. Tests and reproducible automation may request
`factory`; it is never an implicit fallback.

The adapter mapping is exact: GUI New Project submits the normalized form after
its ordinary review action; CLI is `datum-eda project new <path> [--name
<text>] [--project-id <uuid>] [--request-id <uuid>] [--units-source
global|factory] [--expected-preferences <generation-ref-json>] [--json]`; MCP
is `datum.project.new` with the fields of `ProjectGenesisRequestV1`. The expected
preference generation is valid only with Global mode. Factory mode always names
the fixed V1 profile and refuses a Global expectation.

The GUI form labels this choice **Working units for this new Project** with two
radio choices: **Use my Global Units defaults** (default) and **Use Datum factory
Units**. Global mode shows the eight resolved values and the pinned generation
or “factory defaults; no preference file yet.” Factory mode shows the same eight
resolved values and the factory profile version. The summary is keyboard
reachable and announced as one labelled group with choice, source, and all
resolved values. A stale, unreadable, or migration-required Global source keeps
the entered Project name/location and focus, explains the refusal, and offers
the factory radio choice; it never selects it automatically. The ordinary
Create action remains the sole publication trigger. This bounded form contract
must be reconciled into the Claude-owned New Project visual truth before the GUI
portion of GP-CM03 can be accepted; GP-CM02R itself edits no prototype.

`global` resolves exactly the eight active `datum.units.*` keys from one
validated repository snapshot. The six other reserved `ProjectPolicySeed`
identities are `datum.publish.title_block_template_seed`,
`datum.publish.sheet_format_seed`, `datum.publish.scale_fraction_style_seed`,
`datum.projects.seed_profile`, `datum.projects.template_set`, and
`datum.projects.unit_policy_seed`. All six, every non-seed preference, and in
particular the aggregate `datum.projects.unit_policy_seed` are absent. If the repository is
defaults-only, the service pins the named factory contribution as the Global
resolution and records `factory-defaults` in the source. If it is unreadable or
migration-required, Global mode refuses. The user may retry with explicit
factory mode; Datum never silently changes modes.

Factory mode reads no preference repository, creates no preference repository,
and performs no preference migration or recovery acknowledgment. It resolves
the versioned built-in `datum.units.factory.v1` profile through the same Units
validator, records that exact identity/version and no Global generation, and
otherwise follows the identical staging and publication transaction. Any other
factory profile id returns `seed_source_unavailable`.

## 10. Pinned snapshot, atomic genesis, and receipt

Genesis idempotency is scoped to `(normalized destination identity,
request_id)`. It requires no machine-global Project-creation ledger; the
published Project's immutable evidence is the durable index. Reusing a UUID for
a different destination is a distinct request and is discouraged but valid.

Genesis follows this order:

1. Normalize the complete request and compute `genesis_request_digest`. The
   destination normally must be absent. If it exists, resolve it and return the
   original result only when its immutable genesis evidence contains both the
   same request id and request digest; otherwise return `project_target_exists`
   without altering either tree.
2. Acquire or validate the selected preference snapshot. If an expected Global
   generation was supplied, mismatch returns `stale_generation`.
3. Resolve and validate all eight Units values from that one snapshot and build
   `ProjectUnitsSeedReceiptV2`; no later repository read participates.
4. Build all canonical Project shards in a unique sibling staging directory
   named by request and invocation ids. Its plain local marker repeats those
   ids and the request digest. Validate it by `ProjectResolver`, including the
   exact Units profile and receipt, before publication.
5. Durably flush every staged file and directory, then atomically rename the
   complete staging directory to the absent destination and flush its parent.
   Publication is the sole Project-genesis commit point.
6. Resolve the published Project and return its identity, manifest digests,
   Units receipt, and request id.

A Global write after step 2 does not alter the pinned seed and does not make the
Project stale. A failure before publication leaves the destination absent and
the staging directory non-authoritative. Cleanup may remove only a staging
directory whose request id and incomplete marker match. A failure after atomic
rename is recovered by resolving the published Project; retry returns it and
never creates a second Project. Repeating an identical request id and digest
returns the original published result. Reusing a request id with a different
digest returns `idempotency_conflict`; using a different request id against the
same destination returns `project_target_exists`.

If concurrent creators pass the initial absence check, atomic publication is
the arbiter. Exactly one rename can publish. Every loser resolves the winning
destination and applies the same request-id/digest rules; it never replaces the
winner. Staging cleanup may delete only the caller's exact marked directory and
cannot follow symlinks or sweep sibling Projects.

`ProjectUnitsSeedReceiptV2` contains schema version, Project id, genesis request
id, source kind, full Global `GenerationRef` or factory profile id/version,
Units-seed catalog digest, exactly eight ordered items (key, descriptor schema
version, copied value, effective source and disclosure-safe provenance), profile
digest, `genesis_request_digest`, `seed_application_digest`, and creation actor.
The final genesis response may carry published manifest digests outside the
embedded receipt; the receipt never hashes a manifest containing itself. The
Project owns this immutable receipt. Machine preferences may retain only their
normal mutation receipts; they do not become authority for the Project receipt.

V1 receipts remain readable and immutable as V1. They are never rewritten or
relabeled V2 because V1 lacks facts such as the original normalized genesis
request. A separately invoked Project-owned migration may append a
`ProjectUnitsSeedReceiptMigrationV1` companion that references the V1 digest and
contains only facts derivable from the receipt and Project; unavailable fields
are explicitly `legacy_unavailable`, never invented. Merely opening, querying,
or changing Global Preferences performs no Project migration.

## 11. No live following and interchange boundary

After publication, Project display and bare numeric input read only
`ProjectDisplayUnits`. Global queries and mutations cannot obtain a Project
writer, mutate Project files, enqueue Project operations, refresh an open
Project's Units, or change Publish/document units. Explicit-suffix parsing and
interchange import/export remain preference-independent as required by PM-040.

## 12. Clause-by-clause GP-CM03 acceptance matrix

<!-- REQ:GLOBAL-PREFERENCES-COMPLETION:GP-CM02V -->
<!-- REQ:GLOBAL-PREFERENCES-COMPLETION:GP-CM03 -->

The nine readiness findings close at these normative anchors:

| Finding | Closed by |
|---|---|
| 1. exact product inventory and schemas | sections 3–7 |
| 2. actor, provenance, authorization, idempotency, audit | sections 5–6 |
| 3. exact CLI/MCP surface and errors | sections 7–8 |
| 4. one service, configuration root, daemon, multi-process ownership | section 2 |
| 5. exact eight-key seed scope and three creation adapters | sections 1 and 9 |
| 6. pinned snapshot, concurrent write, crash, and publication | section 10 |
| 7. receipt, retry/replay, degraded state, factory mode | sections 7, 9, and 10 |
| 8. stale PM-037 search authority | section 4 and PM-037 amendment |
| 9. cross-surface and deterministic real-Project proof | matrix below |

| Contract clause | Engine proof | GUI proof | CLI proof | MCP/daemon proof | Negative/real-Project proof |
|---|---|---|---|---|---|
| 11 active / 45 reserved | registry/catalog exact-set test | two sections, 11 rows | reserved get/set refused | reserved query/proposal refused | no reserved seed or default serialized |
| one service/path | private-constructor and raw-load gates | controller uses product client | no repository parser/import | bridge contains translation only | filesystem-write inventory has one preference writer |
| queries/explanation | golden response schemas | row/explanation equality | JSON golden | envelope golden | absences/refusals identical |
| Set and Reset | descriptor validation and remove-not-default | immediate save/Reset | expected-generation tests | direct MCP mutation absent | Project bytes and journal unchanged |
| proposals/security | prepare/validate/apply state tests | explicit accept focus/action | human accept-apply | no token mint; mismatch/consume tests | stale proposal never writes |
| process/concurrency | writer lease and full-ref CAS | daemon-owned run | daemon routing/no fallback | two-client race | exactly one resulting generation |
| eight-key seed scope | exact ordered-key/digest tests | Global default selection stated | global/factory modes | required MCP mode | other six plus aggregate absent |
| atomic genesis/receipt | staged-publish fault injection at every step | GUI new Project | CLI new Project | daemon/MCP create | destination absent-or-complete; V1 migration deterministic |
| no live following | seed snapshot pin test | open Project unchanged | mutate Global then inspect Project | same through daemon | three fixed-identity Projects from one canonical request are byte-identical in separate roots; later Global edits change none |
| errors/idempotency | every code constructed and round-tripped | draft preserved | code/exit mapping | code/envelope mapping | retry, crash, stale, unreadable, migration corpus |
| accessibility | typed names/states in response | AT-SPI focus/name/value/state | structured text/JSON | descriptions in tool schema | non-color and keyboard proof |

Completion additionally requires:

1. exact request, response, error, CLI, MCP, active-key, seed-key, and receipt
   inventories registered in `specs/spec_parity_manifest.json` and compared to
   the implementation;
2. `scripts/check_resolver_raw_loads.py`, daemon-write parity, private-writer,
   MCP taxonomy, source-health, dependency, evidence, governance, project-state,
   and cargo-resource gates green;
3. guarded full workspace tests and strict Clippy with no new dependency;
4. durable fixtures for missing, ready, stale, competing-writer, unreadable,
   migration-required, V1 receipt, global-seed, and explicit-factory states;
5. a proof report listing every fault-injection point and confirming no partial
   Project or preference generation became authoritative; and
6. no claim of production acceptance, which remains GP-CM05 after GP-CM04.

## 13. Explicit exclusions

GP-CM03 does not implement Manage Preferences, portable import/export, backup or
restore UI, provider/account/synchronization, managed organization packages,
credential storage, persistent unattended agent authority, guided setup, Start
page activation, non-Units Project Preferences, Publish, Revision, or any new
dependency. It does not activate the 45 reserved candidates or expose the Units
aggregate. Any such need returns to its owning decision rather than widening
this slice.

<!-- EVIDENCE:GLOBAL-PREFERENCES-COMPLETION:GP-CM02R-EXACT-BUILD-CONTRACT -->
