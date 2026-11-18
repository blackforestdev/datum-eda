# Shared Library Asset Specification

Status: governed target specification; owner-directed 2026-11-16, persistent database architecture corrected 2026-11-18; target mechanisms are not runtime delivery claims.
Authority: [PM066](../docs/decisions/PRODUCT_MECHANICS_066_SHARED_ENGINEERING_LIBRARY.md).
Task: `dat-shared-library-spec-vvtz`. All requirements below describe target behavior.

Architecture correction task: `dat-library-database-architecture-study-t9fj`.
The owner directs a persistent SQLite native library containing engineering JSON
and its reusable relationships, with native symbol/footprint editors and resolved
placed engineering snapshots. SQLite as an index of authoritative external JSON
is not the destination. Section 9 is the implementation contract for that target
and takes precedence over earlier V1-only storage/materialization restrictions.
Sections 2–3 retain the exact V1 interchange and manifest-V2 compatibility profile;
their eight-kind and single-active-revision limits do not constrain the declared
V2 interchange/project-manifest-V3 target. Existing projects retain their actual
format until explicit adoption. No current code capability or migration completion
is established by this amendment.

## 1. Scope and authority

This specification owns the common asset record, portable references, contribution
exchange, dependency closure, local acquisition, validation integration and safe
migration contract. Existing domain schemas own engineering geometry, electrical
meaning, process policy and gate-aware mapping. Existing engine operations own
mutation; this specification creates no second writer or approval authority.

SL-001: PM066 supersedes conflicting target interpretations. Existing package-
embedded land patterns, legacy Part maps, raw JSON editing and unbound-symbol
placement may remain disclosed compatibility capabilities; they are not a
substitute for normal complete-component selection or the new exchange contract.
Do not delete compatibility behavior under a specification-only grant.

SL-002: A user can select a component by engineering properties and supported
physical choice during schematic capture, use the same component in PCB, and
inspect or correct its shared library definition without manually constructing
internal relationship records. A symbol can serve many physical implementations;
a footprint can serve many compatible devices with separate explicit mappings.
No terminal-number equivalence or electrical suitability is guessed.

Ordinary authoring, editing, inspection, local copying/export and QA of Datum
work product proceed under the existing execution grant. The project manager
selects the test inputs and copies needed for that work. This specification
requires no separate per-asset clearance record or renewed owner permission for
those operations. Technical schema, integrity and dependency checks still apply.
Section 8 retains specific outside-material and publication decisions; it is not
a general hold on ordinary Datum work.

## 2. Identity, revision and common records

SL-010: UUID identifies an asset, independent of pool, filename, name and MPN.
A portable reference is `AssetRevisionRefV1 {asset_id: UUID, revision_id: Digest}`.
`Digest` is the string `sha256:` followed by exactly 64 lowercase hexadecimal
digits. V1 uses existing SHA-256 support; unsupported algorithms refuse acquisition.
Section 2.1 fixes the byte profile. Local technical revision counters and human
revision labels do not replace this ref.

SL-011: `AssetRevisionV1` is the immutable common envelope:

| Field | Required semantics |
| --- | --- |
| `schema_version` | Integer `1`; envelope version, independent of payload version |
| `asset_id` | Stable UUID; agrees with payload root identity |
| `kind` | Existing library kind: unit, symbol, entity, package, padstack, footprint, part, pin_pad_map, or model_attachment |
| `payload_schema_version` | Explicit supported version of the existing domain payload |
| `payload_ref` | Digest of exact domain JSON bytes in the blob inventory, preserving ordered geometry, UUIDs, units and unknown extension fields |
| `parents` | Ordered exact predecessor refs; empty for a baseline; multiple parents require explicit reviewed merge |
| `dependencies` | Ordered reference slots, each with role, target ref and domain field path |
| `provenance_ref` | Digest-qualified reference to the immutable provenance record |

`model_attachment` identifies an envelope for existing owned attachment/detail
content. It does not introduce a ninth Pool/browser kind, an independent editor,
or a new command family; the existing eight-kind graph and attachment ownership
remain controlling. Attachment owner back-references follow SL-024's contextual
ownership rule rather than recursive content dependencies.

The envelope is stored/addressed by its digest; `revision_id` is computed by the
domain-separated byte profile below, not a self-referential field in those bytes.
Fingerprint verification includes parent and dependency refs and provenance.
No save-time clock/random value is injected. Re-saving the same authored envelope
produces the same bytes. Dates are persisted authorship facts, not serialization
side effects. Unknown payload fields are retained in exact blob bytes; a reader
unable to preserve them refuses mutation rather than silently discarding them.
New profiles require owning specification reconciliation and interoperability proof.

SL-012: References in the domain payload remain existing UUID fields. The
dependency table qualifies each outgoing library reference with its exact
revision; payload UUID and target UUID must agree. Nested pad/pin/gate IDs remain
owned subobjects, not automatically new assets. An omitted dependency is permitted
only for a documented internal, optional-absent or non-library reference. All
required cross-asset references must resolve against the same closure. Unsupported
or ambiguous reference paths refuse; no rebinding to current pool contents.

SL-013: `AssetProvenanceV1` contains `schema_version: 1`, known original authors
and creation time, change actor/time/summary, origin kind (`native`, `import`,
`derived`, `migration`, `unknown`), source refs and fingerprints, derivation refs,
rights/attribution declarations and supporting evidence refs. Actor records carry
recorded identity plus optional verification evidence; missing values are explicit
null/empty records, not invented names/dates. Source documents distinguish title,
issuer, document revision, locator and optional retained content digest. A URL or
hash alone is not authenticity or redistribution permission. ISO-8601 timestamps
retain offset/precision as recorded; unknown times stay null. Imported-by and
migrated-by actors do not become original authors. Ancestry is retained on forks.
These fields record declarations and evidence; they do not create a clearance
workflow. Permitted nulls and empty lists do not block ordinary local work.
Identify the specific retained material and supporting evidence before asserting
an outside attribution requirement; an import path, storage format or number of
refactors alone establishes neither that requirement nor its absence. Preserve
existing source evidence and notices while a particular question is examined.

SL-014: `AssetReviewV1` is a separate immutable record: version, review ID, exact
subject ref, dependency-closure digest, reviewer identity, evidence refs,
applicable rule/profile refs, result, recorded time and supersession/disposition
refs. Existing approved/deprecated/imported/unknown/needs-review semantics remain
owned by the library contract; no new approval taxonomy is implied. Reviews and
availability/lifecycle notices do not rewrite immutable asset content. A changed
subject or dependency basis requires affected revalidation; prior approval does
not transfer automatically. Authenticated actor, declared author, engineering
review, rights and content integrity are separate claims.

SL-015: Editing creates a successor revision with the same asset ID. Relocation
preserves identity. Deriving a distinct asset creates a new ID with source ancestry.
Name/MPN matches never deduplicate identities automatically. Same ID/different
histories requires explicit conflict disposition; search priority cannot decide
which incompatible revision a pinned design means. An unsupported multi-parent
merge is refused, not simulated by replacing a file.

### 2.1 V1 byte profile and exchange layout

SL-016: Profile ID is `datum-library-json-v1`. Metadata objects use UTF-8 without
BOM, compact JSON, recursively sorted object keys in Unicode scalar order and
one trailing LF. Arrays preserve authored order; fields designated sets are
sorted and deduplicated before serialization. Strings are valid Unicode scalars,
with no Unicode normalization: distinct spellings remain distinct bytes. Escape
quote/backslash and use JSON short escapes for backspace, tab, LF, form feed and
CR; other U+0000–001F controls use lowercase `\u00xx`; other scalars emit UTF-8.
Do not escape solidus or other Unicode scalars unnecessarily. Metadata numbers
are base-10 integers with no leading zeros or plus sign; zero is `0`, never `-0`.
Metadata rejects floating numbers and duplicate keys at every depth. UUIDs use
lowercase hyphenated spelling. Null and absent are distinct; required fields may
not be omitted. A V1 reader rejects unknown top-level metadata fields; extensions
are explicit namespaced entries under optional `extensions`, with the same
integer-only metadata profile. New required meaning needs a schema/profile bump.

Domain payload is separate exact-byte JSON of the counter-free engineering
projection defined in SL-024, not a directly loadable native shard. Validate its
engineering fields against the owning domain schema; where that native schema
requires local counters, materialization supplies and validates them under existing
engine rules before native-shard admission. Validate portable bytes for valid
UTF-8/Unicode/finite JSON numbers with no duplicate keys. The blob digest
is SHA-256 of those exact bytes. Parsing/reformatting an acquired blob is not
permitted to change its referenced bytes. New payload writes use existing
deterministic domain serialization over that engineering projection, excluding
only the classified local counters, not an unmodified native-shard serialization.
Fractional number spelling belongs to that payload, not the common metadata
profile. Semantically equivalent alternate
payload encodings may have different fingerprints; fingerprints never assert
semantic equivalence. An unchanged save reuses its acquired bytes. An explicit
normalization is a new proposed revision shown in the semantic/byte diff.

SL-017: Metadata record digest is SHA-256 of `domain_prefix || canonical_bytes`.
Prefix is UTF-8 `datum.library.<type>.v1` followed by LF; `<type>` is exactly
`asset-revision`, `provenance`, `review` or `release-manifest` for the respective
V1 records. Bytes stored in the record file exclude this prefix. The digest field
is always external to the hashed record. Source and blob fingerprints use raw
bytes with no metadata prefix; type and schema are checked separately. A record
claiming another type/digest/profile refuses, even if JSON is parseable.

SL-018: An exchange collection uses `library.json` as its discovery pointer
(`schema_version: 1`, `collection_id: UUID`, `profile: "datum-library-json-v1"`,
`manifest_ref: Digest`),
`objects/sha256/<64hex>.json` for immutable metadata and
`blobs/sha256/<64hex>` for immutable payload/evidence bytes. Manifest lists every
object's type and digest and every blob's digest, byte length and media type;
metadata digests bind type through SL-017. Collections do not infer authority
from paths, tags or hosting accounts. Discovery pointers are candidates until
their manifest is verified. Absolute/parent paths, links escaping the bundle,
duplicate inventory entries and conflicting bytes under one digest refuse.
Transport must impose declared resource limits before loading content; unsupported
sizes/versions are explicit refusals, not partial success. No defaults are invented
for absent required records. Persist source/destination journals through existing
engine transaction boundaries; exchange objects do not replace live pool shards.

### 2.2 Common field shapes

SL-019: The following field names/types fix V1 interchange. `?` denotes nullable,
not optional; every shown field is present. `extensions` is the sole optional
top-level field and is an object keyed by nonempty organization-qualified names.
All records carry `schema_version: 1`. Nonempty identifiers/roles are strings;
UUID, Digest and AssetRevisionRef have SL-010 meanings. Empty lists mean no
recorded facts, not inferred approval. Validation rejects duplicate IDs/slots.

```text
Actor = {recorded_id: string?, display_name: string?,
         verification_evidence: [Evidence]}
Evidence = {title: string, issuer: string?, document_revision: string?,
            locator: string?, retained_blob: Digest?, access_note: string?}
Rights = {license_expression: string?, attribution: [string],
          restrictions: [string], evidence: [Evidence]}
Dependency = {role: string, payload_path: [string], target: AssetRevisionRefV1}
AssetProvenanceV1 = {schema_version, authors: [Actor], created_at: string?,
  change_actor: Actor?, changed_at: string?, change_summary: string?,
  origin_kind: native|import|derived|migration|unknown,
  source_assets: [AssetRevisionRefV1], sources: [Evidence],
  rights: Rights, evidence: [Evidence]}
AssetReviewV1 = {schema_version, review_id: UUID, subject: AssetRevisionRefV1,
  closure_digest: Digest, reviewer: Actor, evidence: [Evidence],
  rule_profile_refs: [Digest], result: passed|failed|needs_review,
  recorded_at: string?, supersedes: [Digest], disposition_refs: [Digest]}
ObjectEntry = {digest: Digest, type: asset-revision|provenance|review}
BlobEntry = {digest: Digest, byte_length: nonnegative integer, media_type: string}
LibraryReleaseManifestV1 = {schema_version, collection_id: UUID,
  release_id: UUID, publisher: Actor, profile: "datum-library-json-v1",
  roots: [AssetRevisionRefV1], closure_digest: Digest,
  objects: [ObjectEntry], blobs: [BlobEntry],
  publication_record_refs: [Digest]}
```

`AssetRevisionV1` has exactly the fields in SL-011. In Dependency, `payload_path`
is an ordered array of decoded field/index tokens from the domain payload root;
array index tokens use canonical nonnegative decimal spelling. Its endpoint must
be the corresponding UUID reference. Roles are validated against the owning
domain schema. Required dependency slots are schema-derived, not trusted from
the sender's declared table. Sort slots lexicographically by token sequence and
then role; reject two entries for one slot. Parents retain authored order with
no duplicate ref. Sort set-like manifest refs by asset UUID then revision digest;
object/blob lists by digest, rejecting duplicates before encoding.

A review result is evidence only, not the existing library approval state.
`rule_profile_refs` and disposition/publication refs identify existing Datum
evidence records, not newly invented authority. Their schema and authorization
remain owned by those systems. Evidence retention can be null where unavailable
or restricted; show that limitation. A retained blob must be inventoried/verified.
Rights strings are declarations, not a license parser or permission grant.

`closure_digest` hashes the canonical sorted unique array of active engineering
AssetRevisionRefV1 values with prefix `datum.library.closure.v1` plus LF, using
the SL-016 canonical bytes. This excludes review/publication/manifest records,
avoiding a circular review-of-release hash. The manifest excludes its own digest
from `objects`; inventory entries cover all active envelopes, provenance and
retained payload/evidence blobs. Historical parent/source refs remain traceable
but their entire ancestry need not ship to instantiate the active closure.
Unavailable historical evidence is disclosed rather than advertised as retained.

## 3. Release and project dependency closure

SL-020: `LibraryReleaseManifestV1` contains version, collection ID, release ID,
publisher record, exact root asset refs, the complete transitive dependency and
evidence/blob inventory, supported schema/encoding profiles and publication
record refs. Paths are transport locators, not identity; inventories use stable
ordering. Repeated identical refs are deduplicated, missing or mismatched records
refuse. Cyclic required revision dependencies must be diagnosed explicitly; the
first implementation may refuse them and must never invent cycle-breaking refs.
An embedded model is transported as inert data under existing opaque-model policy.

SL-021: `ProjectLibraryLockV1` records exact selected component and dependency
refs, acquired content fingerprints, local resolver bindings and source release
provenance. It is authored project data committed through the engine; no UI or
Git client writes it privately. It complements the current LibraryBinding and
ComponentInstance, not a competing resolver. Local `object_revision`/model guards
still prevent stale edits. A portable ref is not a counter reset when a project
opens elsewhere. SL-024 fixes native field placement and the version barrier. Legacy readers must
explicitly refuse that version rather than silently ignoring the lock.

SL-022: Acquisition retains the content needed for offline open, editing and
manufacturing reproduction, including available review/evidence records and
required blobs. Remote deletion, withdrawn release or branch/tag movement cannot
change pinned bytes. Withdrawal is visible advisory/governed state and follows
the applicable project policy; it does not replace content silently. Garbage
collection must preserve project-referenced and retained published revisions.

SL-023: Library definition edit, library release publication and project adoption
are distinct actions. Update review shows changed geometry, mappings, properties,
provenance and check basis, affected placed users and dependencies. Accepting an
update uses existing proposal/commit guards, records before/after refs and supports
canonical Undo/Redo. Cancel/refusal makes no authored change. Existing designs
are never migrated merely by refresh, search, download or application startup.

SL-024: Explicit, separately accepted format adoption establishes project manifest
target schema version `2`, which declares `library_lock: {path: "library/lock.json",
schema_version: 1}`. `library/lock.json` is `ProjectLibraryLockV1`:

```text
{schema_version: 1, profile: "datum-library-json-v1",
 bindings: [{component_instance_id: UUID, source_path: string,
             instance_path: [UUID], component: AssetRevisionRefV1}],
 assets: [{asset: AssetRevisionRefV1, local_object_id: UUID,
           source_path: string, payload_ref: Digest}],
 source_manifests: [Digest], closure_digest: Digest}
```

Paths are project-relative normalized source-shard paths under native path rules.
Bindings use the existing source/instance-qualified component identity; sort by
source path, instance path then component ID. Asset entries sort by portable ref;
source manifests sort by digest. Local object IDs retain existing engine UUID
meaning; the portable asset ID must agree. The lock stores no timestamps, transient
search paths or current remote heads. Retained metadata/blobs live under project
`library/objects/sha256/` and `library/blobs/sha256/` using SL-018 filenames.

Existing pool shards remain the engine projection/materialization through
canonical operations. A mismatch with the lock is a diagnostic and blocks affected
adoption/production; it must not be repaired by selecting whichever bytes are
newer. The transaction stages verified immutable objects first, then changes
pool materialization, binding and lock together through journaled commit. Partial
staging is unreferenced/quarantined, never an adopted lock. Undo/Redo restores
matching prior/next lock, bindings and materialization within the adopted v2
manifest baseline; retained objects are not
deleted merely because the active head changes. Recovery must reconstruct the
same combined state before exposing it as valid.

Projection verification does not rehash reserialized live pool JSON as though it
were the acquired blob. First verify retained raw blob bytes against `payload_ref`;
then compare its domain projection with the live object at the lock's source path
and ID. The projection includes root `schema_version`, `uuid`, every authored
domain field and every unknown extension field recursively. Object key ordering,
whitespace and string escape spelling are immaterial; array order, missing versus
null, scalar types and exact values remain significant. Do not use lossy floating
casts for numeric comparison; schema-permitted representational normalization
must preserve the exact domain value and is otherwise refused. Validation cannot
drop unknown fields merely because a typed deserializer ignores them.

Only root `object_revision` and `model_revision`, when classified by the owning
native schema as local engine counters, are excluded from this authored projection.
New V1 portable payloads omit those counters; the source snapshot/evidence may
retain the original source bytes. Imported local counters do not become portable
revision identity. Local counters remain in existing journal/resolver bookkeeping
or current native materialization where required. File paths, journal transaction
IDs and common asset-envelope metadata live outside the domain payload. There is
no wildcard exclusion for unknown keys. If an owning schema gives one of those
names authored engineering meaning, it is not excluded. An unsupported field
classification refuses conversion rather than guessing. The same projection
contract is used for staging, adoption, check-input validation and cold reopen.

Format adoption is forward-only under NATIVE_FORMAT_SPEC section 8 and the
existing guarded profile-adoption pattern. Before a reversible asset adoption,
show and separately accept the v1-to-v2 format change and save a coherent v2
baseline manifest/declaration plus valid lock through a profile-specific adoption
implementation on the existing canonical mutation/journal route. PM057's guarded
adoption is a pattern only; no generic project-manifest-v2 or library-lock writer
is claimed to exist. Reader, writer and crash-recovery proof is required before
enabling this profile. Initial lock may have empty `bindings`, `assets` and
`source_manifests`, with the canonical empty-closure digest; that means zero assets
are profile-managed, not that existing native pool data has gained provenance or
approval. Existing library data and bindings retain their actual legacy status.
The resolver reports which component/library dependencies are unmanaged and the
resulting coverage gaps; no empty lock is advertised as a fully migrated project.
Only asset entries covered by the lock use the exact projection check above.

Preview/Cancel/refusal before format-adoption acceptance changes nothing. Once
format adoption is accepted/saved, later cancellation/refusal of acquisition or
asset edits retains that disclosed baseline. Undo/Redo of asset edits restores the
corresponding valid v2 lock, source materialization and bindings; it never restores
v1 or removes the profile declaration. Retained unreferenced immutable content
remains inert cache/recovery data, not adopted assets. Recovery restores a coherent
v2 baseline or completed v2 asset transaction before exposing authored state.
Failed format adoption itself recovers the last complete pre-adoption state under
the required profile-specific writer/recovery implementation; it is not a
user-invoked backward migration. There is
no general manifest downgrade, historical inverse across the format boundary or
new exception to existing forward-only rules.

V1 requires one active revision per asset ID in a project closure. Conflicting
versions refuse with affected users shown; the user may deliberately update the
dependent set or derive a distinct asset. Priority/aliases may not silently mask
the conflict. No automatic asset ID rewriting or simultaneous distributed merge
is introduced. This bounded refusal is explicit, not a claim of multi-version
resolver support.

`PinPadMap.part` is a contextual owner back-reference in current domain schema,
not a recursive dependency edge. Resolve it to the selecting Part in the same
closure and validate its ID, mapping endpoints and physical choice. Likewise,
owned gate/pin/pad IDs resolve through that explicit owner context. Every actual
forward asset dependency remains revision-qualified; cyclic forward inheritance
or ambiguous owner contexts refuse. This prevents a Part/default-map ownership
cycle from requiring impossible self-referential content hashes.

Profile adoption writes manifest version 2 only after that separate explicit
preview/acceptance; ordinary opening/saving legacy version 1 does not upgrade it. Before enabling the
writer, prove supported readers resolve/recover version 2 and unsupported readers
refuse it without writes, including the lock declaration. An older reader that
ignores versions is incompatible and cannot be presented as safe. Removing the
lock, downgrading the manifest or exporting legacy shards is an explicit lossy
conversion with revision/provenance-loss disclosure, never automatic Save.

## 4. Editing, contribution and publication

SL-030: Library Editor uses shared viewport/editor services and engine operations
for create/edit/derive/inspect/compare/validate. Source-qualified relationships are
shown together with the component's symbol, physical choice and mapping. Existing
component-first placement and local Properties Apply/Cancel remain controlling.
New History/Contribution UI states require their own visual review before native
implementation; this text does not invent approved pixel layouts.

SL-031: Contributors may submit candidates and supporting evidence. The exchange
flow is `candidate -> validation -> distinct engineering review -> publication
disposition -> available release`; a failed or disputed candidate stays explicit.
Automated checks are evidence, not engineering approval. AI follows existing
proposal guards and cannot silently approve. Policy defines who may publish into
a collection; participation and voting do not establish factual accuracy.

SL-032: Vendor identities may be independently verified with verification scope
and evidence. A vendor can maintain its definitions; the community can propose
evidence-backed corrections. A vendor badge implies neither correctness nor
warranty/liability/certification. Conflicting source documents remain traceable;
maintainers record evidence and rationale rather than counting votes. Unknown
source rights or unverified claims cannot be promoted by popular agreement.

SL-033: GitHub can host deterministic portable asset exchange, review discussions
and candidate release bundles. The persistent native SQLite library is local
authored storage; JSON/Git exchange is transport, not a competing live writer.
Derived search/preview caches are not authored content. A PR merge, tag, account permission or GitHub release does not
issue a Datum engineering approval or project release. Offline contribution
bundles use the same schema, checks and local authority path. Git is optional;
no core authoring/check/open operation requires a GitHub login or network.

SL-034: Acquired candidate content is quarantined until schema, identity,
integrity, dependency and compatibility checks complete; reviewable semantic diff
then canonical operations adopt it. A different directory or transport does not
turn Datum work product into third-party material. Apply a source-rights condition
to the specific identified outside material and action it actually covers, rather
than treating missing declarations as a blanket acquisition or local-QA hold.
No direct checkout writes into a live project and no executable repository hooks
run during acquisition. Reject unsafe paths and unresolved/unsupported blobs
before promotion. Existing egress and import boundaries remain; this specification
grants no new dependency.

## 5. Checks and manufacturing

SL-040: Reuse CheckRun/CheckFinding, severity, basis and deviation machinery.
Library checks cover schema/relationships, pin mapping and applicable geometry;
ERC/DRC also checks actual design context. Search, published status or passing
library checks alone do not prove circuit suitability or manufacturing compliance.

SL-041: A finding binds subject asset ref, affected object/feature, dependency
basis, exact rule/profile revisions, check coverage, severity and source evidence.
Actual/expected values retain units when meaningful. Open in Library Editor must
reach the owning asset/feature. Editing and then adopting a corrected revision
invalidates affected checks, not all historical proof. Missing/unsupported checks
are disclosed as coverage gaps, never PASS.

SL-042: Manufacturing publication evaluates the exact resolved design snapshot,
library lock, process basis, rule revisions, findings and valid dispositions.
Blocking errors prevent publication; advisory warnings follow explicit policy.
Preview/export inspection and formal project release remain distinct existing
workflows, with no new mandatory document-control UI. Recheck if any input changes
between validation and output publication. A manifest traces output to that basis.
No new waiver mechanism or automatic unknown-basis blocker is introduced.

SL-043: PM061 governs process geometry: zero mask expansion and mask-defined
choices retain their meanings; tented means no opening. Unknown standards basis
is disclosed separately from unresolved geometry, which blocks affected output.
An applicable mask rule can flag an incorrect opening, but no universal positive
expansion, numerical IPC limit or certification claim is authored here. Standards
evidence names the applicable document/revision and actual supported coverage.

## 6. Persistence, segmentation and search

SL-050: The target native library is persistent SQLite storage containing the
actual reusable engineering JSON, asset/revision identity, exact relationships,
history and query projections. It is not an index over authoritative external
library files. One engine-owned resolver/DesignModel and canonical typed mutation
route govern semantics. Section 9 defines the SQL/JSON authority, meaningful
geometry/process composition, writer/recovery protocol and project snapshot
boundary. Existing file libraries and the in-memory PoolIndex are compatibility
and migration inputs, not fulfillment of this requirement. Project design/journal
persistence is not wholesale replaced by this library amendment.

SL-051: Search indexes identify the database generation and exact revisions they represent. Build
or update transactionally into a complete generation; readers see the prior valid
generation or the new complete one, never a partial rebuild labeled current.
After a crash, mismatch or source mutation, detect staleness and rebuild/recover
or report unavailable/incomplete results. Never silently omit assets as though
the catalog were complete. Search and subsequent adoption verify the same subject
revision; changed content requires renewed validation, not stale row acceptance.

SL-052: Index replacement cannot mutate authored SQL content/history/reviews. Engine
mutations publish a complete valid change or preserve/recover the preceding
state. Verify interrupted writes/acquisition/index rebuild and backup restoration
with exact revision identity retained. Search corruption is not repaired by
rewriting source facts or refreshing approval timestamps.

SL-053: Logical segmentation follows existing asset boundaries; physical library
partitioning follows ownership/distribution when needed. It must not force pad,
symbol and map updates into uncoordinated stores or make pool priority an identity
rule. Personal/team favorites are optional future preferences, not asset history;
search-frequency surveillance and ranking analytics are outside this model.

SL-054: Search engineering properties with explicit units and package identity;
show predictable results and source/revision/status. Load geometry/history on
demand, with bounded result paging and visible loading/failure states. Establish
latency and scale requirements from representative real libraries and measured
tasks before adopting numeric budgets; no unsupported speed claim is acceptance.

## 7. Migration and implementation handoff

SL-060: Inventory native/imported/fixture/user/vendor assets, their actual rights,
existing bindings and compatibility fallbacks before planning bulk migration.
Do not redistribute imported or fixture content on assumed permission. This does
not thaw PM011 compatibility conversion or authorize external library acquisition.
This preparation for bulk migration is not a prerequisite for separately granted
ordinary native editing, local copying/export or QA. Broader compliance audits
require their own scope; they are not an implicit step in every local operation.

SL-061: Conversion preview identifies each old ID/source and new baseline ref,
dependency mapping, retained unknown fields, geometry/property/mapping differences,
and validation findings. Preserve IDs, known provenance and engineering content.
Record the migration actor/time separately; never invent prior history or approval.
Baseline revision means current known content, not proof of original creation.
Ambiguous mappings, missing dependencies and identity collisions remain findings,
not heuristic repairs. Existing designs remain resolvable under old bindings.

SL-062: Preserve originals and provide deterministic restart/rollback. Partial
conversion is quarantined until the complete validated closure can be committed.
Repeated conversion of unchanged input is idempotent and cannot mint duplicate
identities. A conversion that changes engineering content is a separate explicit
repair proposal. Refusal/Cancel leaves the original project/library unchanged.

SL-063: Before broad conversion, a separately granted first slice exercises one
lawfully usable resistor family: shared symbol, multiple physical choices,
property-driven selection, recorded baseline/provenance, correction/review,
versioned exchange, deliberate project adoption, affected checking, Undo/Redo,
Save/reopen and offline reproduction. Fixed done statement, real-data proof and
independent review precede expansion. This list proposes a slice; it does not
select a milestone or authorize runtime work.

SL-064: Full current-library migration remains required implementation scope.
The initial family is a proving slice, never completion of migration. Account for
all native/user/legacy catalog classes, unavailable/refused entries, dependencies,
models, mappings and unmanaged bindings. Preserve exact engineering meaning,
identity and unknown fields; classify actual unsupported coverage rather than
silently dropping data. Section 9.I fixes restart, cutover, recovery and completion.

## 8. Decisions retained outside this amendment

The following are explicit boundaries, not invitations for agents to choose:

| Decision | Blocked scope until disposition |
| --- | --- |
| Shared repository owner, maintainer/reviewer authority, verified-vendor process and publication policy | Public collection publishing/verification |
| A concrete unresolved condition on identified third-party material or a retained outside document | The specific acquisition or distribution action covered by that condition |
| Multi-pool shadowing, ranking and placement gate defaults retained by PM062 | New behavior in those areas |
| Evidence-supported numerical resource/performance budgets | New numerical acceptance claims; unsupported figures remain undecided and do not block ordinary implementation |
| Exact first integration scope and done statement | Runtime integration and asset migration |

Ordinary Datum work-product editing, local copies/exports, inspection and QA are
PM-selected execution work. Do not route them to this table merely because a
candidate contains provenance fields or has an import history. A stopped action
must identify the concrete outside-material question and its affected scope;
continue other authorized work.

Owner-only decisions remain owner-only; an appointed PM may dispose only within
its verified appointment. Existing behavior and separately authorized work do not
acquire new blockers from unrelated policy decisions. For every implementation
task, cite applicable SL requirements and unsettled dependencies; retain current
behavior vs target truth. No agent may treat this spec as approval to invent
publisher authority, source rights or standards limits. SL-016–024 fix the V1
byte, envelope and lock profile; implementation still requires interoperability,
version-barrier and ordinary workflow proof before claiming support.


## 9. Native SQL library implementation target

This section is normative target architecture under PM066, not a claim of current
implementation. It supersedes conflicting earlier V1-only target assumptions;
legacy formats retain their original declared semantics. Sections J–N make the
wire types, contextual native projection, operations and validation cases explicit.

### 9.A. Authority and format boundaries

A1. A native shared library is a SQLite store containing its authored engineering payloads, immutable revisions, relationship records, history and searchable projections. A file index over external authoritative JSON is not a conforming target. JSON payloads and portable exchange remain supported; the database transaction is the persistence boundary for library edits. The resolver-owned DesignModel and canonical typed mutation route remain semantic authority.

A2. Store schema V1 identifies itself with SQLite application_id `0x44544c42` (ASCII DTLB), user_version `1` and a store_metadata row with stable store UUID, role `workspace|installed|project_retention`, schema version and active supported encoding profiles. Reusing a legacy PoolIndex file as a native library without explicit conversion refuses. Foreign-key enforcement is enabled and verified on every connection. Use parameter binding and validated read/write APIs; user expressions are not executable SQL. Installed stores open read-only; edits create revisions in a writable workspace without changing portable asset identity.

A3. Database schema version, portable envelope version, domain payload version, generation algorithm version and project manifest version are separate. Store V1 supports legacy portable profile `datum-library-json-v1` without rewriting its sealed bytes and new `datum-library-json-v2`. V2 retains SL-016 exact payload and canonical metadata rules, with domain-separated metadata prefixes `datum.library.<type>.v2\n`; new metadata fields/kinds require declared V2. A V1 record retains its V1 prefix/digest even inside a V2 collection. Inventory identifies record type/profile/version so hashes are never interpreted under another profile. Exact metadata/blob bytes are retained and exported unchanged.

A4. Project manifest target V3 declares `library_store:{path:"library/library.sqlite", store_uuid:UUID, schema_version:1, profile:"datum-library-json-v2", binding_schema_version:2}`. This declaration is authored format metadata, not a bare existence check. Existing manifest V1/V2, V1 exchange, BoardRoot profiles and legacy locks remain read-compatible under their original rules; opening/saving does not adopt V3. Unsupported readers refuse before any write. V3 adoption is a separate explicit forward-only format transaction with a coherent baseline and recovery proof. It changes neither placed engineering geometry nor library revision selection by itself.

A5. Project-retained SQLite contains immutable adopted closure data and immutable adoption receipts; it is a self-contained source for offline resolution, not a cache of remote latest data. Active project bindings/occurrence poses and committed adoption transaction membership are owned by the existing canonical project journal/revision-authority protocol. SQL does not have an independently writable active-project head. Current active state is derived from authenticated project transactions and their preimages/inverses. No shadow lock.json becomes a competing target writer; V2 lock remains a legacy format. The project SQL store UUID is declared/pinned by accepted format metadata and adoption receipts, preventing substitution of a different DB at the same path.

A6. A format-only V2-to-V3 adoption retains the exact existing `library_lock` declaration, lock bytes, immutable records/blobs and their native materializations as a read-only legacy provider. The V3 manifest additionally carries `legacy_library:{profile:"datum-library-json-v1",lock_path:"library/lock.json",manifest_schema_version:2}` when that provider exists; it is absent for an original V1 project without a managed lock. The retained declaration/lock are not an independently advancing V3 writer. An occurrence with a canonically committed BindingV2 uses the SQL/context provider; an untouched occurrence uses its retained exact legacy provider. No empty new DB invalidates an existing managed binding or declares it migrated. New editing/adoption of such an occurrence proposes an explicit V2-payload/context conversion first; read/inspect/history/offline resolution retain the original provider meanwhile. Retirement of a legacy provider requires no active or retained-history user to need it and an explicit canonical source-ownership disposition. Undo restores the recorded provider, never a current-head fallback.

### 9.B. Logical SQL schema and invariants

All UUIDs have canonical spelling or a specified fixed-width internal representation with lossless canonical export. Digests retain explicit algorithm. Stable labels are TEXT; authored integer dimensions retain signed 64-bit nanometres; booleans/enums are checked. Nullable and absent JSON fields remain distinguishable in canonical payloads. Internal rowids are never portable identity.

| Relation | Key and required content | Constraints |
| --- | --- | --- |
| store_metadata | singleton store UUID/role/schema/profiles and committed catalog generation | Role/version validated before queries; generation advances only on committed workspace mutations |
| content | digest PK, media type, byte length, exact bytes | Digest/length verified on insertion/acquisition; same digest with different bytes refuses; immutable |
| records | record digest PK, type/version, raw content digest | Record digest hashes its domain prefix and bytes; raw content digest hashes bytes alone; do not conflate them |
| assets | asset UUID PK, asset kind | Kind immutable for an existing identity; names are revision data |
| revisions | `(asset_uuid, revision_digest)` PK; envelope profile/version; envelope content digest; payload digest/schema; provenance digest | Referenced content exists; envelope UUID/kind agrees; every sealed field verified; immutable |
| revision_parents | owner revision + ordered ordinal; exact parent revision | No duplicate parent; parent asset identity agrees; no parent cycle; unavailable transported ancestry can be explicitly external, never falsely retained |
| dependencies | owner revision + canonical payload-slot path PK; role; exact target revision | Required slot coverage/kind checked by engine; UUID in payload agrees; FK targets retained before active publication; no guessed mapping |
| collection_members | collection UUID + exact root revision | Collection membership is not identity or latest-wins priority |
| workspace_heads | workspace/branch identity + asset UUID; exact selected revision; generation guard | Mutable through canonical transaction only; advancing head does not mutate prior revision or project users |
| properties | exact revision + property key + ordinal; type, units/domain and value | Derived from typed payload/metadata when projected; unit-aware numeric ordering; no arbitrary string-to-number coercion |
| search_documents | exact revision PK + generation; searchable text/availability | Derived, rebuildable and generation-qualified; never payload authority |
| library_transactions | transaction UUID PK; idempotency key UNIQUE; expected/new generation; typed operation, preimage/inverse refs and disposition | Same idempotency key/different request refuses; accepted operation+head/projections/history committed together |
| local_ids | project-retained store + exact revision + owned feature path | Stable local UUID UNIQUE; portable asset UUID remains unchanged; needed only for native object registration |
| resolved_snapshots | digest PK; format/resolver version; closure digest; exact geometry/process/mapping/basis content refs | Immutable, validated complete source basis; pixel/mesh content cannot satisfy snapshot |
| adoption_receipts | adoption UUID PK; project/store UUID; request/idempotency identity; expected project revision; before/after binding context and snapshot refs; complete inventory digest | SQL retention proves availability, not project acceptance; active iff canonical project journal authorizes exact receipt |

`content` retains all required engineering/model blobs; this target has no mandatory external blob store. Large blobs use bounded streaming reads/writes; transaction scope and retained-byte admission apply. Optional locator-only evidence is labeled unavailable offline and cannot masquerade as required retained engineering content. Optional derived preview/mesh caches may live outside SQLite and are discardable. Checks/reviews are immutable exact-subject records; their disposition does not rewrite asset geometry or history.

The schema permits many revision rows per UUID. Dependencies use full composite keys, not foreign keys to a mutable head. Record ancestry and engineering dependency relationships are separate: export need not recursively include all historical ancestors to instantiate an active closure. Every required current dependency and required model blob must be present before publication.

One fact has one authoring owner. Indexed SQL columns mechanically projected from payloads are written only by the engine transaction and checked/rebuilt from their source revision. JSON references plus relational slot qualification are validated as a single revision; a UI cannot patch an edge without constructing a matching successor payload. DDL checks/FKs are necessary but do not replace shape/electrical/geometry validators.

### 9.C. Engineering composition and payload contracts

C1. Add meaningful reusable asset kinds `geometry` and `process_definition` to the V2 profile, alongside existing unit/symbol/entity/package/padstack/footprint/part/pin_pad_map/attachment semantics. They are engine DomainObjects with the common identity/provenance/revision envelope, not a compulsory new top-level picker workflow. An owned shape/contour/segment remains an identified subobject until explicitly promoted for reuse. No independent asset is created automatically for every point/line.

C2. `geometry` payload V2 has the exact AssetHeader plus Shape/Construction fields in section J. Coordinates are local to `(0,0)`; no separate anchor field shifts them. Asymmetric contours and offset holes preserve that chosen origin rather than silently shifting a centroid. Shapes permit exact circle, rectangle, rounded/chamfered rectangle, capsule and explicit closed contour sets. Each tag has a closed required-field schema; unsupported fields/tags refuse edits while preserved original bytes remain inspectable/exportable.

C3. Circle uses positive diameter; rectangle positive width/height; rounded/chamfered rectangle adds explicit corner selections and absolute radii/chamfers, with no unspecified default. Capsule specifies positive overall width/height and its defined major-axis construction. Contours specify an ordered set of closed outer/hole loops, stable segment IDs, explicit line endpoints or arc center/endpoints and direction, winding and topology. Arc sweep is determined from those endpoints and direction under section J, not a second independently editable angle. Segment continuity, closure, nondegeneracy, hole containment and intersections are validated. The same tag may not mean different geometry in another version. Re-entrant fillets and Boolean combinations require an explicit versioned construction rule and valid resulting exact contour; unsupported construction is an error, not a renderer approximation.

C4. Authored coordinates/lengths use existing native exact integer conventions; angle serialization follows the governing canonical IR (integer tenths of a degree). Exact derived rational/algebraic geometry retains its supported tagged arithmetic representation, not SQLite REAL or rounded integer substitution. Every transform records angle/reflection semantics. A consumer unable to preserve a declared exact transformation/result refuses its affected action. Display tessellation error is a presentation property and cannot alter source, DRC or CAM geometry. This target does not claim the current transform/output adapters support every new contour or angle.

C5. `process_definition` V2 specifies process role `mask|paste|thermal|antipad`, mode `explicit|derived|suppressed|unresolved`, selected basis and overrides, algorithm ID/version, and parameters. Explicit mode has ordered identified aperture geometry references/transforms, allowing zero/multiple apertures with explicit semantics. Derived mode names exact input geometry/dependencies and rule parameters; suppression is explicit; unresolved carries the preserved input and reason. Negative/zero offsets are interpreted by the declared rule and never globally repaired. Mask aperture, paste aperture, copper connectivity, thermal spokes and clearance are different roles.

C6. Padstack V2 contains identified copper layer-role entries with exact geometry refs/local transforms; identified hole/drill definitions with offsets, plating/span and connection facts; exact process_definition refs or explicit owned process values; thermal/antipad and engineering basis. Semantic layer roles resolve against the selected board stackup, including front/back/reflection and reached exterior faces. The effective-geometry service validates all resulting copper/hole/process geometry together. A through-hole, nonplated hole, SMD pad or slot is not inferred solely from width/height or name. Complex custom hole support is capability-qualified and refuses unsupported output.

C7. Footprint V2 contains package revision ref and owned pad uses `{feature_id, terminal_label, padstack_ref, position, rotation, mirrored, layer_binding, overrides}` plus typed courtyard/fab/silk/assembly/mechanical geometry, origin and placement constraints. Terminal labels are not portable asset identity. Overrides name the exact field/value and scope; they do not mutate the referenced padstack. Arrays may be an authoring construction, but the saved engineering result retains stable explicit pad-use identities/positions and declared generator basis. Reordering does not renumber electrical terminals. Package V2 owns body/terminals/tolerances and models; embedded legacy land patterns are migration inputs only.

C8. Unit owns electrical pin identity/types/alternates; Symbol owns graphics, pin anchors, text/fields and style basis; Entity owns gate-to-Unit/Symbol assignments; Part owns selectable properties and explicit supported physical implementation choices. A physical implementation pins Package, Footprint and gate-aware PinPadMap revisions. Reusable families may support many choices, but a selected component occurrence records one explicit supported choice. Values, ratings and manufacturer/orderable data are typed distinct properties; no suitability inferred from common package names. Several physical pad uses may explicitly map to one logical pin; custom copper outline alone never determines that mapping.

A sealed V1 Part is not a ContextV2 component merely because it has an asset UUID. Keep its legacy provider or create an explicitly previewed V2 successor retaining that UUID/parent ancestry and recording a new stable physical-choice UUID in the migration plan. A choice is constructed only from the actual complete declared Package/Footprint/default-map pair with validated gate/pad correspondence; no guessed package/map/rating fills a gap. A V1 incomplete Part remains inspectable/exportable/editable as an incomplete authoring record, and cannot be advertised as a qualified component until explicitly completed. The conversion preserves existing electrical/geometry meaning and reports all changed representation/IDs. Creating a new choice ID is a recorded owned-feature allocation, not new original authorship.

C9. Standards-driven construction records actual standard/document revision, applicable rule identifier, inputs/units, generator version, evaluated values and explicit overrides/deviations. No particular IPC radius/expansion or IEEE/IEC compliance is created by this storage contract. Declared basis, available evidence, evaluated geometry and checking coverage remain distinct. Future standards updates create reviewed successors; they do not recompute old adopted snapshots. Exact numerical rule implementations require the separately applicable source/basis authority.

### 9.D. Revision context and native object resolution

D1. A revision context contains selected component root revision, selected physical choice, and the verified dependency graph digest. Edge resolution is keyed by `(owner exact revision, schema-owned reference slot)`, not by bare target UUID. This permits two slots/occurrences to pin different revisions of the same asset without shadowing. Required forward dependency cycles refuse; contextual owner back-references remain validated contextual links, not self-referential content hashes.

D2. Project component identity is source/instance-qualified: project UUID, source-shard identity, hierarchy instance path and ComponentInstance UUID. A binding pins its revision context and exact snapshot. Library handles are `(exact asset revision, owned feature path)` within that occurrence. Bare UUID lookup is allowed only with an unambiguous explicit revision context; ambiguous lookup returns a typed error and never chooses latest.

D3. Where the native engine requires globally unique local ObjectIds, V3 uses persisted `local_ids` mapping from exact portable revision/owned feature path to project-local UUID. Repeated uses of an identical revision can share the same registered definition; component/pad occurrences retain separate existing identities. Different revisions allocate distinct local definition UUIDs. These are disclosed local bindings, never new portable asset identities. Migration preserves existing local IDs where unambiguous and persists every new allocation in the adoption receipt/journal; Undo/Redo/replay restore the recorded IDs rather than allocate again. Native queries expose both local object and portable exact identity.

D4. V1/V2 legacy one-active-revision rules remain enforced until V3 adoption. A reader advertising V3 must resolve coexisting revisions in selection, editor, connectivity, checks, rendering, CAM, undo/replay and offline reopen; SQL rows alone are insufficient. Export to a legacy profile refuses unsupported coexistence or uses a separately explicit disclosed lossy conversion, never automatic ID rewriting.

### 9.E. Engine read/write and editor contracts

Reads expose paged SearchAssets, GetAssetRevision, ResolveClosure, GetWhereUsed, CompareRevisions, PreparePreview and ResolvePlacedSnapshot semantics through the existing lean query surface. Requests include source/store identity and exact subject/generation; results identify completeness, capability and current generation. Search can read indexed metadata without geometry; explicit fetch is bounded. No new verb per shape is necessary.

Writes use existing prepare/propose/commit semantics with scope `library_workspace|project`. Each command has request/idempotency UUID, expected store/project generation, exact subject refs and preconditions. Semantic intents include CreateAsset, EditOwnedFeature, ReviseSharedDefinition, DeriveAsset, SetPhysicalChoice, PrepareAdoption, AcceptAdoption and AdoptProjectLibraryFormat. Editing through native GUI/CLI/automation reaches the same validator/transaction; no SQL editor private writer. Automated-proposal/review boundaries remain unchanged.

Editor draft state is transient and not a sealed revision. Preview validates and returns a generation-qualified result plus changed fields/dependencies/affected users. Cancel drops draft state; it does not change database history, project binding or output. Commit creates a revision and changes only explicitly selected workspace heads. Editing a shared pad shows where-used and distinguishes revise-shared from localize-this-use; localizing derives a new identity or explicit owned override, preserving ancestry. Footprint transformations operate on owned pad uses, not shared shape coordinates. Symbol pin/gate changes require affected mapping validation and expose unresolved users before adoption.

Library Undo/Redo is an engine transaction restoring the selected workspace head/preimage while retaining all immutable revisions. Project Undo/Redo restores recorded binding contexts, local IDs, geometry/process snapshots, net/pad correspondence and existing canonical receipt semantics; it never reloads latest definitions. A committed library revision cannot be retroactively deleted by Undo.

Required structured errors: unsupported store/profile/schema/kind/capability; invalid geometry/topology; unresolved effective process geometry; missing/corrupt content; missing/ambiguous dependency; invalid mapping; stale draft/preview/project; preview_expired; ambiguous revision context; conflicting identity; read-only store; resource admission failure; writer busy/retryable; cancelled-before-commit; commit-outcome-uncertain. Each includes subject/operation and whether authored state is known unchanged, committed or outcome-uncertain. Unknown standards basis alone is not unresolved geometry. An uncertain write is reconciled by idempotency/transaction identity before retry.

### 9.F. Library transaction state machine

`DRAFT -> PREVIEWED -> VALIDATING -> COMMITTED` or `REFUSED/CANCELLED`. Only COMMITTED changes persistent authored state. Read snapshot/generation used for preview is revalidated under the short writer transaction. One SQL transaction inserts all sealed content/revisions/dependencies/history and updates heads/projections/generation; rollback leaves prior authored state. Content acquisition may stage privately before validation. Committed transaction row/idempotency record is the definitive library outcome after connection loss; a retry with identical request returns that result. Same key/different content refuses.

One engine writer queue per store plus interprocess SQLite locking/generation checks serializes committed writes. Busy waits are bounded by existing job cancellation policy and never block redraw. Long computations happen outside the write lock. Readers use bounded coherent transactions; no UI session holds a read transaction indefinitely. The initial store durability profile is DELETE/FULL under section L; readers/writes use bounded transactions so lock waits do not block the viewport. A later WAL optimization must satisfy the fixed-build/configuration constraints in L. No shared-network-filesystem authoring store or attached-database set-atomicity assumption is allowed.

### 9.G. Project adoption and cross-store recovery state machine

The canonical project journal/revision authority owns adoption commitment, not a separately mutable SQL status flag. Reuse the existing project lease and authenticated expected-model/transaction-tip guards.

1. `PREVIEW`: obtain exact before context and candidate closure; compute semantic/mapping/geometry/process diff, affected placed users and check invalidation. No authored changes.
2. `RETAIN`: under the project lease verify source/project generations, copy required immutable content/revisions/models into the project DB and insert a sealed adoption receipt plus complete inventory digest in one durable SQL transaction. Required rows are now retained but not adopted. Precompute exact resulting snapshots/ID mappings; unsupported geometry/mapping refuses before commitment.
3. `STAGE`: compose the canonical project operation batch using that receipt, existing native preimages, local-ID allocations, new contexts, schematic/board correspondence and frozen physical/process geometry. Stage existing revision-authority postimages before publication. A stage error leaves the previous project active and retained content inert.
4. `COMMIT`: canonical project transaction durably appends/publishes using existing journal/revision authority. The transaction contains the exact adoption receipt identity/digest and before/after binding/snapshot refs. This is the logical commitment point; no later SQL head update is needed to decide adoption.
5. `MATERIALIZE/RECOVER`: existing project recovery reconstructs the complete postimage and active contexts from authenticated journal/revision authority. It verifies every receipt/content ref against the project DB before exposing the changed state as valid. Any derived SQL active-use index is rebuilt from this authority.

| Interruption | Required outcome |
| --- | --- |
| Before RETAIN SQL commit | Previous project remains active; incomplete SQL transaction rolls back |
| After RETAIN, before canonical COMMIT | Complete retained content/receipt remains inert; no component adopts it; resume/cancel uses idempotency identity |
| During canonical publication | Return uncertain outcome; reopen/reconcile existing authority before retry, recover complete prior or committed project state |
| After canonical COMMIT, before projections/cursor finish | Recovery completes authenticated postimage; do not select old library heads or recompute from current standards |
| Committed receipt references missing/corrupt DB content | Report project integrity unavailable and block affected mutation/production; restore retained bytes from coherent backup; never substitute current/remote content silently |

After COMMIT, Cancel is not a rollback; explicit canonical Undo applies recorded inverse. Same adoption identity cannot mint a second set of pad/component IDs. No SQL transaction is held across file journal publication. SQL and files have different fact ownership with one project commitment authority, not an unsupported two-phase atomicity claim.

GC must retain every closure/snapshot/local-ID mapping referenced by the canonical current state, undo/redo history, retained published releases, prepared active jobs and readers. Inert abandoned receipts/content can be collected only after the canonical outcome is known and no retained roots refer to them. The initial implementation may retain rather than collect; a quota failure is explicit, never permission to delete history.

### 9.H. Runtime, checks and rendering

Resolved snapshots contain exact local geometry, stable feature/electrical mappings, process results and raw selected basis, stackup/profile/source pins, construction/resolver versions and relevant transform semantics. Persistent shared project snapshots are keyed by content/context; per-occurrence pose/net/text/selection remain separate. Native symbol/footprint editor views work on authoring definitions; board/schematic views use their accepted snapshots.

Camera redraw/pan/zoom uses resident prepared scene data. Require zero library SQL reads, zero library dependency-graph re-resolution and zero exact geometry regeneration solely because camera position changes. Rebuild camera-dependent presentation/culling as necessary; this requirement does not forbid camera-dependent drawing work. Moves/rotations change instance transform/spatial bounds without resolving library heads. Explicit placement/load/update prepares missing exact geometry in bounded jobs.

Renderer mesh/glyph buffers and GPU instancing are optional derived representations of the same exact snapshot. Cache keys include exact source/content/context, process basis, renderer/style version and relevant presentation parameters; stale entries cannot qualify a different revision. Memory admission/eviction follows existing document/resource policies and shares immutable allocations where beneficial; deduplication in SQL is not a claimed RAM/VRAM result.

Hit testing retains feature identities; connectivity/ERC uses electrical interface/mapping. DRC/CAM use the same exact snapshot/process/layer meaning and actual per-instance transforms, with capability-specific refusal before partial output publication. Cached display meshes never replace exact verification/output. A library update changes no placed result until accepted adoption. History/reopen can rebuild caches from pinned snapshots without remote library access or current-rule reinterpretation.

### 9.I. Search, backup, migration and implementation completion

Search uses typed unit-qualified properties, explicit supported physical choices and exact revision results. Text FTS is optional after supported-build verification; ordinary exact/range indexes are required for advertised queries. Results are paged with stable generation-qualified continuation; a catalog mutation invalidates/restarts a stale page token explicitly. Preview fetch loads requested geometry on demand. Derived projections are transactionally current or visibly stale/incomplete; reindex never changes asset content.

Backups use supported SQLite snapshot/backup operations, retain project canonical revision metadata/receipt reachability coherently and verify restore. Do not copy only an active DB main file without its committed WAL state. Multi-store backup manifests identify each store UUID and retained content/transaction generation; restoration verifies exact adopted receipts and project output basis. Encryption/account/service additions are not implied by this local persistence contract; existing filesystem/egress policy applies.

Migration proceeds source-snapshot -> private destination -> complete validation -> explicit cutover. Record source fingerprints, every asset/status/ID/extension mapping, retained failures, legacy Package/Footprint decisions and ImportMap keys. Stable IDs and semantic exactness are invariants, not record-count equality alone. Refused/unsupported catalog entries remain explicit unavailable inventory with retained source evidence; they cannot be advertised as converted working assets. Distinguish full inventory accounting, successfully usable asset migration and genuinely unsupported engineering coverage. No engineering correction or guessed mapping hides inside storage conversion.

Destination schema and import checkpoint transactions are idempotent. Every batch is durable but private until complete cutover; stale source snapshot refuses/resumes against a new explicit plan. Preserve originals/coherent backup. V3 project adoption retains existing component/pad/wire/net/pose/history identity and records all local binding changes; V1/V2 existing projects remain usable under their declared formats until explicit migration. A single resistor demonstration is one proving slice, not full-library completion.

Implementation completion requires actual persistent SQL acquisition/search/edit/history/backup/recovery, both native editors and component-first selection, exact project snapshots and multirevision contexts, explicit update/undo/offline reopen, supported checks/CAM and complete full-catalog migration accounting. A delivered increment states its exact subset; no index-only fallback, empty lock, compile pass or metadata UI is labeled completion. Accepted revision/inspection/exchange workflows retain compatible behavior or have an explicit reviewed compatibility disposition. Disputed product UI is neither silently mandated nor deleted by storage migration.

Performance validation compares representative cold/warm library and design tasks, records data/machine/build/cache conditions, and separates SQL/decode/resolve/geometry/scene/GPU costs and disk/RAM/VRAM peaks. Architectural no-per-redraw-query and bounded-resource requirements are directly testable. No invented numerical threshold or new mandatory global campaign is introduced. Numeric release claims require the existing external-budget/provenance rules and actual evidence.

### 9.J. Closed wire types, geometry and process grammar

The following notation defines closed required records, not optional illustrative fields. `T?` means a required nullable field; `[]` means an ordered array; optional extension data is allowed only in a declared `extensions` map. Integers are signed 64-bit unless explicitly constrained otherwise. Uuid/Digest/Actor/Evidence retain SL-010/019 spelling/types. `Ref={asset_id:Uuid,revision_id:Digest}`. `Point={x_nm:Int,y_nm:Int}`. `Transform={position:Point,rotation_tenths:Int,mirrored:Bool}` has rotation 0..3599; reflection negates local X before positive counterclockwise rotation in a local X-right/Y-up coordinate system, followed by translation. Adapter conversions to native legacy angle units must be explicit and lossless. Overflow refuses. Migration may normalize equivalent angle spelling only with a disclosed representation revision.

`AssetHeader={schema_version:2,uuid:Uuid,name:String,description:String,extensions:Map}` is common to every new payload; extensions are required, possibly empty. Additional fields listed below are all required. Assets retain common provenance through their envelope, not duplicated author arrays in every subobject. Unknown namespaced extensions remain exact; unknown required semantic features cannot be silently ignored.

```text
CornerValues = {nw_nm:NonnegativeInt,ne_nm:NonnegativeInt,
                se_nm:NonnegativeInt,sw_nm:NonnegativeInt}
LineSegment = {id:Uuid,kind:"line",start:Point,end:Point}
ArcSegment = {id:Uuid,kind:"arc",start:Point,end:Point,
              center:Point,direction:"cw"|"ccw"}
Loop = {id:Uuid,role:"outer"|"hole",segments:[LineSegment|ArcSegment]}
Shape = {kind:"circle",diameter_nm:PositiveInt}
      | {kind:"rect",width_nm:PositiveInt,height_nm:PositiveInt}
      | {kind:"round_rect",width_nm:PositiveInt,height_nm:PositiveInt,
         corners:CornerValues}
      | {kind:"chamfer_rect",width_nm:PositiveInt,height_nm:PositiveInt,
         corners:CornerValues}
      | {kind:"capsule",width_nm:PositiveInt,height_nm:PositiveInt}
      | {kind:"contours",loops:[Loop]}
GeometryPayloadV2 = AssetHeader + {shape:Shape,construction:Construction?}
Construction = {algorithm_id:String,algorithm_version:PositiveInt,
  inputs:[Ref],parameters:Map,basis:Basis,output_geometry_digest:Digest}
Basis = {status:"documented"|"declared"|"unknown"|"import_preserved",
         sources:[Evidence],rule_id:String?,rule_revision:String?,
         input_values:Map,overrides:Map}
GeometryUse = {id:Uuid,geometry:Ref,transform:Transform}
ProcessValue = {mode:"explicit",apertures:[GeometryUse],basis:Basis}
             | {mode:"derived",input_geometry:Ref,algorithm_id:String,
                algorithm_version:PositiveInt,parameters:Map,basis:Basis}
             | {mode:"suppressed",basis:Basis}
             | {mode:"unresolved",reason:String,preserved_input:JSON,basis:Basis}
ProcessPayloadV2 = AssetHeader + {
  role:"mask"|"paste"|"thermal"|"antipad",value:ProcessValue}
LayerRole = {kind:"front"|"back"|"all_copper"}
          | {kind:"inner",ordinal:PositiveInt}
          | {kind:"span",start:"front"|"back"|PositiveInt,
             end:"front"|"back"|PositiveInt}
CopperEntry = {id:Uuid,layer_role:LayerRole,geometry:Ref,transform:Transform}
Hole = {id:Uuid,shape:Shape,transform:Transform,
        connection:"non_plated"|"plated_through"|"plated_span"|"unknown",
        span:LayerRole,basis:Basis}
ProcessSelection = {source:"definition",definition:Ref}
                 | {source:"owned",value:ProcessValue}
                 | {source:"inherit",process:"mask"|"paste"}
PadstackPayloadV2 = AssetHeader + {copper:[CopperEntry],holes:[Hole],
   mask:ProcessSelection,paste:ProcessSelection,
   thermal:ProcessSelection,antipad:ProcessSelection,basis:Basis}
ProcessBasis = {source:"inline",mask:ProcessValue,paste:ProcessValue}
             | {source:"profile",profile_id:Uuid,expected_revision:NonnegativeInt,
                expected_value_hash:Digest}
             | {source:"ipc_basis",basis:Basis,mask:ProcessValue,paste:ProcessValue}
             | {source:"none"}
PadUse = {feature_id:Uuid,terminal_label:String,padstack:Ref,
          transform:Transform,layer_binding:LayerRole,
          overrides:{mask:ProcessValue?,paste:ProcessValue?,
                     thermal:ProcessValue?,antipad:ProcessValue?}}
Graphic = {feature_id:Uuid,geometry:Ref,transform:Transform,
           role:"courtyard"|"fab"|"silk"|"assembly"|"mechanical",
           layer_role:LayerRole,stroke_width_nm:NonnegativeInt,filled:Bool}
FootprintPayloadV2 = AssetHeader + {package:Ref,pads:[PadUse],graphics:[Graphic],
   origin:Point,process_basis:ProcessBasis,basis:Basis,models:[Ref]}
PhysicalChoice = {id:Uuid,package:Ref,footprint:Ref,pin_pad_map:Ref,
                  constraints:[PropertyConstraint],label:String}
PropertyConstraint = {key:String,type:"text"|"bool"|"integer"|"decimal",
   unit:String?,relation:"equal"|"one_of"|"range",values:[JSON]}
PartPayloadV2 = AssetHeader + {entity:Ref,physical_choices:[PhysicalChoice],
   default_choice:Uuid?,properties:[Property],models:[Ref],base:Ref?,
   retained_details_ref:Digest?}
Property = {key:String,type:"text"|"bool"|"integer"|"decimal"|"range",
            unit:String?,value:JSON}
```

Shapes are centered on local `(0,0)` except explicit contours, whose coordinates preserve the chosen anchor. Rect corners are at `(±width/2,±height/2)` exactly, including half-nanometre boundaries; do not round an odd dimension. NW/NE/SE/SW are defined in X-right/Y-up coordinates. Rounded corners replace the corresponding 90-degree vertex with an internally tangent circular arc of the given radius; zero is sharp. Require adjacent radii sums to fit the shared edge; each radius cannot exceed the shorter side. Chamfers replace the corner with a line at the specified equal setbacks, with the same edge-sum constraints. A capsule has straight sides along its larger dimension and semicircular ends of diameter equal to the shorter dimension; equal dimensions produce a circle. Custom contours can express re-entrant fillets without assuming every corner is convex.

Every contour has at least one outer loop; segment IDs are unique within the asset. Successive endpoints including the closing pair agree exactly, zero-length segments refuse, arcs have distinct endpoints and a positive equal squared radius to their center. Arc direction selects the unique non-full sweep between endpoints; a full circle uses the circle shape or multiple explicit arcs. Outer loops are counterclockwise, holes clockwise; overlapping/self-intersecting loops, tangent/crossing holes or holes outside an outer loop refuse. Multiple disjoint outer loops are representable, but conductive/electrical suitability remains an explicit pad/check question. Higher-order curves require a new shape capability/version, not tessellation substituted into this schema.

`Construction.parameters` is validated against the named algorithm/version's registered closed schema; unknown algorithms can be retained/inspected with their explicit resulting Shape, but regeneration refuses. No expression/script is executed from imported JSON. Initial registered algorithms are the explicit primitives above and declared rule-driven process expansion/reduction; their exact supported input/output combinations are capability-qualified. Boolean-composite provenance is retained only with an explicit validated resulting Shape; regeneration is unsupported until its algorithm is implemented. This prevents an unbounded arbitrary-code/constraint-solver promise from hiding in `parameters`.

Per-layer copper entries cannot overlap ambiguously in layer-role selection; resolve layer roles to actual stackup IDs before validation/materialization. An inner ordinal is counted from the front exterior copper, and cannot name an exterior/missing layer. Explicit spans include both endpoints in physical stackup order. Hole connection facts remain distinct from geometric hole/span presence. Unsupported profile/layer/shape combinations are diagnosed by subject, not assigned defaults.

Process precedence is exact: a nonnull PadUse override for that process wins; otherwise an explicit/definition Padstack value wins; `inherit` selects that Footprint's declared ProcessBasis. An absent/none basis cannot choose a project profile implicitly. Process definition role must agree with the selected field. Unknown standards basis with known explicit geometry remains usable/disclosed; unresolved geometry is a separate state and blocks the affected output/materialization. Suppressed geometry is a resolved empty result, not an error. Legacy PM061 raw missing versus explicit Inherit remains retained in migration/source evidence, even if both require the same selected basis. Explicit rules never borrow unrelated profile parameters.

Thermal/antipad fields do not accept inherit in this version because ProcessBasis/profile selection defines only mask/paste. They require an explicit owned value or role-matching definition, including explicit suppression or preserved unresolved state. New inheritance for those roles requires a versioned contract rather than an invented default.

`models` refs retain attachment role/format/content semantics from PM008 and native model contracts; an attachment envelope is an owned-detail kind, not a new top-level geometry editor. Unit/Symbol/Entity/Package/PinPadMap V2 retain their full owning schemas and source meanings with every outgoing library ID qualified through exact dependency slots. Unit owns logical pin IDs; Entity gates select Unit/Symbol; map entries use gate/pin and footprint pad-use feature IDs. Package V2 removes target land-pattern fields while preserving them in the migration record. Domain schema revisions enumerate that change explicitly; an unknown legacy field remains preserved and is not guessed away.

Property text/bool/integer values have their JSON scalar type; decimal is `{coefficient:String,exponent10:Int}` with canonical signed decimal integer coefficient, no leading zeros/plus, zero exponent normalized to zero for zero value; range is `{minimum:decimal,maximum:decimal}` with minimum<=maximum. Units are explicit canonical engineering unit identifiers; absent units are permitted only for dimensionless/text properties. Search normalizes compatible units without treating display spelling as identity. Numeric indexing may use a bounded conservative projection with exact postfilter; REAL never decides exact engineering equality/range acceptance.

A physical-choice constraint compares the selected component property of its key and declared type/unit: equal has one value; one_of has one or more distinct scalar values; range has exactly two ordered inclusive numeric endpoints and cannot be used for text/bool. Values use the declared Property scalar encoding. Missing/incompatible/unknown-unit input is unresolved and cannot be advertised as a qualified choice; no arbitrary expression/script or implicit package-name suitability is evaluated. Empty constraints mean no additional declared parametric restrictions, not an assertion that engineering validation passed.

Part `retained_details_ref` retains the exact existing native Part descriptive/model/supply-chain detail record, validated by its declared native payload schema and preserved with all unknown fields. It is not a second Part identity or geometry authority. V1 field mapping is explicit: value/manufacturer/mpn/datasheet/lifecycle become reserved text properties of those names; tags become ordered `keyword` text properties; parametric entries become `parametric.<original-key>` text properties without inferred units/coercion. Native description maps to the header; orderable/packaging/model/thermal/supply-chain details retain their current typed semantics through retained_details_ref until explicitly structured equivalents are authored. Consumers show those retained fields rather than silently hiding them. Newly authored data can use the same supported details schema. Base inheritance is an exact optional Part revision dependency; a required inheritance cycle refuses. Existing model attachments preserve their declared format/content/role and can be promoted to exact model refs without duplication of authorship. A transformation that cannot preserve a known field's engineering behavior is a migration finding/refusal, not success merely because the bytes were archived.

### 9.K. Portable V2 records, contexts and adoption receipts

All hashes use SHA-256. `H(type,version,obj)` is SHA256 of UTF-8 `datum.library.` + type + `.v` + decimal version followed by exactly one byte 0A, then SL-016 canonical metadata bytes. This notation means a real LF byte, not two backslash/n characters. Raw blob digests hash bytes alone. Canonical object keys and designated sets follow SL-016; arrays below marked ordered retain their order. Envelopes, inventories and receipts reject duplicate keys, unknown required fields and non-finite/lossy numeric parsing.

```text
ObjectEntryV2 = {digest:Digest,type:"asset-revision"|"provenance"|"review",
                 record_version:1|2}
DiscoveryV2 = {schema_version:2,collection_id:Uuid,
               profile:"datum-library-json-v2",manifest_ref:Digest}
ManifestV2 = {schema_version:2,collection_id:Uuid,release_id:Uuid,publisher:Actor,
  profile:"datum-library-json-v2",roots:[Ref],closure_digest:Digest,
  objects:[ObjectEntryV2],blobs:[BlobEntry],publication_record_refs:[Digest]}
ContextV2 = {schema_version:2,component:Ref,physical_choice_id:Uuid,
             closure_digest:Digest}
OccurrenceV2 = {project_id:Uuid,source_shard_id:Uuid,
                instance_path:[Uuid],component_instance_id:Uuid}
BindingV2 = {occurrence:OccurrenceV2,context_ref:Digest,snapshot_ref:Digest}
LocalIdV2 = {owner:Ref,feature_path:[Uuid],local_object_id:Uuid}
FeatureRef = {owner:Ref,feature_path:[Uuid]}
ResolvedGeometryUse = {feature:FeatureRef,pad_use:FeatureRef?,
   role:"copper"|"drill"|"mask"|"paste"|"thermal"|"antipad"|
        "courtyard"|"fab"|"silk"|"assembly"|"mechanical",
   layer_role:LayerRole,shape_ref:Digest,transform:Transform}
RationalV2 = {numerator:String,denominator:String}
Quadratic3V2 = {a:RationalV2,b:RationalV2}
EffectiveShapeV2 = {kind:"circle",diameter_nm:Quadratic3V2}
                 | {kind:"rect",width_nm:Quadratic3V2,height_nm:Quadratic3V2}
                 | {kind:"round_rect",width_nm:Quadratic3V2,
                    height_nm:Quadratic3V2,radius_nm:Quadratic3V2}
                 | {kind:"explicit_geometry",apertures:[ResolvedGeometryUse]}
EffectiveProcessV2 = {resolution:"resolved",apertures:[EffectiveShapeV2]}
                   | {resolution:"suppressed"}
                   | {resolution:"unresolved",reason:String}
FrozenProcessV2 = {version:2,mask:EffectiveProcessV2,paste:EffectiveProcessV2,
  basis_status:"documented"|"declared"|"unknown"|"import_preserved",
  membership:{mask_layers:[Int],paste_layers:[Int]},basis_pins:[BasisPin],
  raw_inputs:{padstack_payload_ref:Digest,footprint_payload_ref:Digest,
              profile_payload_ref:Digest?,stackup_payload_ref:Digest,
              source_capability_ref:Digest},
  transform_basis:Transform}
ProcessSnapshot = {kind:"legacy_v1",pad_use:FeatureRef,raw_snapshot_ref:Digest}
                | {kind:"native_v2",pad_use:FeatureRef,value:FrozenProcessV2}
SnapshotV2 = {schema_version:2,context_ref:Digest,resolver_version:PositiveInt,
   source_content:[Digest],shapes:[{digest:Digest,value:Shape}],
   local_geometry:[ResolvedGeometryUse],
   symbols:[{gate_id:Uuid,symbol:Ref,unit:Ref}],
   logical_physical_map:[{gate_id:Uuid,pin_id:Uuid,pad_feature_id:Uuid}],
   process_snapshots:[ProcessSnapshot],basis_pins:[BasisPin],capabilities:[String]}
BasisPin = {role:"stackup"|"profile"|"rule"|"source",
            object_id:Uuid?,object_revision:NonnegativeInt?,value_hash:Digest}
ReceiptV2 = {schema_version:2,adoption_id:Uuid,request_id:Uuid,
   request_hash:Digest,project_id:Uuid,store_uuid:Uuid,
   expected_model_revision:String,before:[BindingV2],after:[BindingV2],
   contexts:[{digest:Digest,value:ContextV2}],
   snapshots:[{digest:Digest,value:SnapshotV2}],local_ids:[LocalIdV2],
   source_changes:[SourceChangeV2],
   inventory:{revisions:[Ref],records:[ObjectEntryV2],blobs:[BlobEntry]},
   inventory_digest:Digest}
AdoptionJournalLinkV2 = {adoption_id:Uuid,receipt_ref:Digest,store_uuid:Uuid,
                         before_binding_digest:Digest,after_binding_digest:Digest}
SourceLocatorV2 = {kind:"legacy_shard",source_shard_id:Uuid,relative_path:String,
                   local_object_id:Uuid,source_bytes_digest:Digest}
                | {kind:"library_store",store_uuid:Uuid,asset:Ref,
                   feature_path:[Uuid],local_object_id:Uuid}
SourceChangeV2 = {local_object_id:Uuid,before:SourceLocatorV2?,after:SourceLocatorV2?,
                   before_membership:[String],after_membership:[String]}
```

AssetRevisionV2 has exactly SL-011 fields with schema_version=2 and declared supported payload versions/kinds, including geometry/process_definition; no omitted required field. V2 provenance/review have SL-019 fields with schema_version=2. V1 records in a V2 manifest keep V1 fields, prefix/digest and limitations; entry record_version controls decoding, never a guessed payload tag. Duplicate entries claiming different type/version for one digest refuse. A manifest is hashed with H(release-manifest,2,...). Discovery points to that verified manifest; paths retain SL-018 safety rules. V1 exporter emits only a closure representable in V1, otherwise refuses without partial output. ModelAttachment retains its documented contextual-owner rule.

V2 Dependency.payload_path selects the `asset_id` scalar inside an exact Ref object, for example `["pads","0","padstack","asset_id"]`; its sibling revision_id must equal the declared target revision. V1 UUID-only fields retain their old direct scalar endpoint. Registered slot schemas fix allowed roles/target kinds and enumerate all required Ref occurrences, including construction inputs, process definitions, shape uses, models, base Parts and physical choices. Array index tokens identify immutable payload positions; stable feature paths for native identity use owned UUIDs instead. PinPadMap.part remains an owner UUID/contextual back-reference, not an exact forward Part dependency that would create a content-hash cycle; gate/pin/pad owned IDs likewise validate in the chosen owner context. Missing/duplicate/extra reference slots or mismatching Ref fields refuse sealing/acquisition.

The V2 closure digest is H(closure,2, sorted-unique-array-of-exact-Refs); sorting is by UUID then revision digest, with no UUID-only uniqueness requirement. Every forward dependency is followed through its owning exact revision/slot. Context digest is H(context,2,ContextV2); physical_choice_id must select an actual owned choice in its component revision. Snapshot digest is H(resolved-snapshot,2,SnapshotV2). Receipt inventory digest is H(retained-inventory,2,inventory); receipt_ref is H(adoption-receipt,2,ReceiptV2). Before/after binding-set digest is H(binding-set,2,sorted-binding-array). Binding sets sort by source shard, hierarchy path and component UUID; no duplicate occurrence. Object/blob inventories retain their deterministic sorting and duplicate refusal; feature_path uses stable owned UUIDs, never array positions or display names.

`SnapshotV2.logical_physical_map` is the exact gate/pin-to-owned-pad-use relation from the chosen PinPadMap, sorted by that tuple; duplicate tuples refuse. Legacy process snapshots retain the exact declared PM061/native-format bytes (version,effective,membership,capability,stackup,stackup_value_hash,profile,raw_padstack,raw_footprint,transform_basis); `raw_snapshot_ref` must decode and validate under that legacy profile, not reinterpret its fields as V2. FrozenProcessV2 states all effective outputs and retains raw selected inputs/profile/stackup through the named content digests and basis pins. Known geometry/unknown standards remains distinguishable from unresolved geometry. Membership IDs are actual adopted stackup process layers and retain PM061 face/span semantics. RationalV2 strings are canonical signed decimal i128 numerator and positive i128 denominator, reduced by GCD, with zero represented as 0/1; Quadratic3V2 means a+b*sqrt(3). Unsupported exact arithmetic/consumer results refuse, never round. These are derived result types; authored Point coordinates stay integer nanometres.

Resolved shape digest is H(resolved-shape,2,Shape). Snapshot shapes sort by digest, contain no duplicates and cover every ResolvedGeometryUse. This stores generated output as snapshot-owned geometry, not fictitious newly authored library assets. `source_content` is a sorted unique complete retained raw blob set needed by the snapshot. FeatureRef names an exact owning revision and stable owned UUID path; ResolvedGeometryUse.pad_use names the exact footprint pad instance when applicable, while feature can name the underlying padstack copper/hole/process feature. Their composition is validated against the context, never inferred from array order. Graphics have null pad_use and their explicit role. Symbols sort by gate ID; local geometry sorts by canonical `(pad_use,feature,role,layer_role,transform,shape_ref)` tuple; process snapshots by canonical pad_use. Duplicate use tuples/process pad owners refuse. All source/shape references load during preparation; they are not per-frame DB traversal instructions. Snapshot shape sharing permits reuse while source provenance remains in the exact closure and source-content inventory.

Construction.output_geometry_digest uses the same H(resolved-shape,2,Shape), computed from the explicit resulting Shape without including its Construction record. ReceiptV2.adoption_id equals its request_id; no hidden identifier is minted while sealing. Source changes sort uniquely by local_object_id, and their exact before/after locators are included in the receipt and preview digest.

SQL relation constraints implement these records: content raw digest!=metadata record identity by definition; records points to raw content; revisions points to its asset-revision record and verified payload/provenance records; required dependencies FK retained composite revision keys. Parent/source ancestry may remain external: revision_parents stores immutable target_ref plus availability `retained|external`; a separate nullable resolved revision FK is present iff retained. Missing ancestors never satisfy required forward edges. Store integrity verifies all content/digest/type/version agreement, not just SQL foreign keys.

For native materialization, retain portable payload bytes unchanged. Build a contextual projection: root/owned object IDs resolve through persisted LocalIdV2 entries, outgoing reference slots through their exact dependency target's local ID, and owned feature references through the owner revision+UUID feature path. Rewrites occur only in the native projection, never sealed payload or portable envelope. Store inverse mappings for query/diff/source identity. Instance bindings and frozen pad/process evidence carry the portable context plus native local IDs so history validates the same source. A migrated unambiguous existing local ID is retained; a newly needed second-revision ID is allocated once, sealed in receipt and reused on replay. Hash/byte comparison never mistakes this projection for acquired raw payload.

### 9.L. Requests, outcomes and durable publication details

```text
WriteRequest = {request_id:Uuid,target_store:Uuid,
  expected_generation:NonnegativeInt,actor:Actor,purpose:String,intent:Intent}
Intent = {kind:"create",asset_kind:String,asset_id:Uuid,payload_ref:Digest,
           dependencies:[Dependency],provenance_ref:Digest}
       | {kind:"revise",base:Ref,payload_ref:Digest,dependencies:[Dependency],
           provenance_ref:Digest,expected_head:Ref?}
       | {kind:"derive",source:Ref,new_asset_id:Uuid,payload_ref:Digest,
           dependencies:[Dependency],provenance_ref:Digest}
       | {kind:"move_head",asset_id:Uuid,before:Ref?,after:Ref?,
           compensates_transaction:Uuid?}
PreviewResult = {request_hash:Digest,generation:NonnegativeInt,
                  candidate:Ref?,changed_subjects:[Ref],affected_users:[Ref],
                  findings:[Finding],capabilities:[String]}
WriteResult = {request_id:Uuid,transaction_id:Uuid,generation:NonnegativeInt,
                revisions:[Ref],head_changes:[{asset_id:Uuid,before:Ref?,after:Ref?}]}
PrepareAdoption = {request_id:Uuid,adoption_id:Uuid,project_id:Uuid,store_uuid:Uuid,
  expected_model_revision:String,expected_journal_tip:Uuid?,
  selections:[{occurrence:OccurrenceV2,before:BindingV2?,after_context:ContextV2}],
  expected_sources:[{store_uuid:Uuid,generation:NonnegativeInt}],
  local_id_plan:[LocalIdV2],source_change_plan:[SourceChangeV2],actor:Actor,purpose:String}
PreparedAdoption = {request_id:Uuid,request_hash:Digest,preview_digest:Digest,
                    receipt_ref:Digest?,findings:[Finding],affected:[OccurrenceV2]}
AcceptAdoption = {request_id:Uuid,adoption_id:Uuid,prepared_request_hash:Digest,
                  preview_digest:Digest,expected_model_revision:String,
                  expected_journal_tip:Uuid?}
AdoptFormatV3 = {request_id:Uuid,project_id:Uuid,expected_model_revision:String,
                 expected_journal_tip:Uuid?,manifest_preimage_hash:Digest,
                 target_store_uuid:Uuid,actor:Actor,purpose:String}
Finding = {code:String,subject:String,severity:"error"|"warning"|"info",
            message:String,details_ref:Digest}
```

Dependency retains SL-019 `{role,payload_path,target}`; V2 domain exact-ref fields must agree with its asset UUID and revision. Each create intent uses `payload_ref` (the same raw digest field as revise/derive); the payload bytes and provenance record are supplied as separately staged immutable input content. Requests hash digest-qualified inputs, never embedded fractional domain JSON under the integer-only metadata profile. `provenance_ref` names the verified provenance metadata record; finding details are separately byte-digested JSON content so retained legacy fractional values/unknown extensions are not rounded or rejected merely for metadata hashing. EditOwnedFeature and ReviseSharedDefinition are editor intents compiled to a full guarded revise request; Derive compiles to derive. They are not extra SQL patch writers. SetPhysicalChoice compiles to adoption selection for project use or revise of a reusable Part; a caller cannot conflate these scopes. Parent list for create is empty, revise has base as parent, derive has new asset identity and source ancestry in provenance. Unsupported multi-parent merge refuses unless supplied through a separately defined merge operation. Move_head validates same asset, exact before and retained after; inverses use this to implement workspace undo/redo without erasing revisions.

The engine assigns no hidden clock/UUID to sealed content: request includes stable IDs/recorded actor/purpose and the candidate metadata has recorded change facts. Idempotency request_hash is H(write-request,2,canonical request) or H(adoption-request,2,PrepareAdoption); retries reuse the same request ID and full request hash. A preview digest is H(adoption-preview,2,{request_hash,expected_model_revision,expected_journal_tip,contexts,snapshots,findings,local_ids,source_changes}). Its findings are the stable projection `{code,subject,severity,details_ref}` sorted by that tuple; response message text is excluded. Contexts/snapshots sort by digest; local_ids by `(owner,feature_path)`; source_changes by local_object_id; selections/bindings by occurrence; expected_sources by store UUID; roots and revision inventories by `(asset_id,revision_id)`; basis pins by `(role,object_id,object_revision,value_hash)`; set-valued capabilities/content digests lexicographically. Duplicate set entries refuse. Authored order inside contours, instance hierarchy paths and dependency field paths is retained. Preview/acceptance rechecks the exact same inputs, including changed source generations, before retention/staging. New or changed findings/context/ID plan requires a new preview. WriteResult/PreparedAdoption are responses, not unjournaled authority.

The prepare planner returns the complete local-ID plan once; the caller's guarded PrepareAdoption carries that plan as input and retries reuse it byte-for-byte. Existing mappings are reused; missing mappings are assigned fresh unused local UUIDs in the transient plan, checked under the project lease before retention. A preview lost before acceptance expires explicitly rather than silently generating a different accepted plan. ID allocation is not an authored write until the prepared plan is sealed/retained and canonically committed. Receipt/context/snapshot hashes are computable before retention from that complete plan; SQL publication records those exact bytes. The receipt does not contain its own digest or canonical project transaction ID, avoiding circular hashes. The journal contains the receipt link and transaction identity; outcome lookup joins them by adoption/request identity and verifies the request hash.

PrepareAdoption, AcceptAdoption and ReceiptV2 use the same logical request_id/adoption_id, explicitly equal to each other. A transport RPC correlation ID is outside this logical request and cannot mint another adoption. Accept names that original identity and prepared_request_hash; a mismatch refuses. This convention makes both IDs stable caller input rather than a hidden seal-time allocation.

After reconnect/cold reopen, first reconcile whether the original request already committed using the SQL idempotency/receipt and authenticated canonical journal link. If committed, return that outcome. If not committed and the complete transient preview/request/guard plan was lost, return `preview_expired` and require a new prepared preview; a stored digest or inert retained receipt alone cannot reconstruct consent/current guards. No automatic adoption resumes from hash-only state. Reuse retained immutable bytes without selecting their old uncommitted plan. This applies even when RETAIN succeeded before the interruption.

Create refuses an existing identity of conflicting kind/history. Revise checks base exists and expected_head/generation matches; deriving never steals source identity. Actor/purpose/provenance input validation preserves unknown prior original-author facts without inventing them. Stale preview fails with no project adoption; retained verified content, if already staged, remains inert. Cancel before canonical commit abandons the draft/preparation; cancellation after commit returns committed identity and requires explicit Undo. AcceptAdoption request retries reconcile its original request/receipt/canonical transaction link, not a newly created operation.

Durability policy: local writable library/project-retention stores use journal_mode=DELETE with synchronous=FULL and foreign_keys=ON, verified on every writer connection before authored writes. Only supported SQLite builds/filesystems satisfying the existing durable-write assumptions are admitted. A read-only installed store uses a checkpointed standalone DB with no required writable sidecars. A mismatching journal/durability configuration refuses the writer; there is no silent NORMAL/OFF fallback. The initial DELETE/FULL profile avoids relying on concurrent WAL behavior in the currently cached SQLite 3.46.0 source. Optional later WAL support requires a separately declared, verified configuration with the documented WAL-reset fix (3.51.3 or applicable fixed backport) or a verified serialized write/checkpoint policy; no dependency upgrade is authorized here. WAL is host-local and supplies no cross-database atomicity. Busy/lock timeout follows existing cancellable job policy; long waits do not run on the viewport thread.

RETAIN SQL commit completes before journal publication; only then can a canonical journal link reference its sealed receipt. Project format adoption creates/checkpoints the new DB and durably publishes its containing-directory entry through the existing canonical staging/recovery protocol before activating the manifest declaration. Store UUID/schema/profile are verified at read, write and recovery. An I/O error with ambiguous SQL COMMIT result is reconciled using request ID/hash and the committed SQL transaction/receipt record before proceeding to project journal. A library SQL commit result is likewise looked up on reconnect; no duplicate revision/head transition is minted by retry. Project receipt journal links are authenticated alongside the complete native pre/postimage and canonical inverse, retaining existing uncertain-publication semantics.

Backup protocol: acquire project lease, reconcile all uncertain publications, fix the authenticated project journal tip/model revision and store UUID, and hold adoption/format/GC mutation for that project. Use SQLite backup API to a private destination for a coherent project-retention snapshot while library reads continue. Copy the corresponding canonical project authored files/journal/revision-authority/cursor and store declaration under that same fixed revision. Write a backup manifest `{schema_version:1,project_id,model_revision,journal_tip,store_uuid,store_schema_version,retained_inventory_digest,files:[{path,byte_length,digest}]}` with paths sorted and safe. Verify every referenced receipt/content and file digest before atomically publishing the backup directory. Source library workspace backup is a separate SQL snapshot; it need not share a global transaction with consuming projects.

Restore verifies the backup manifest, store identity and canonical project authority before exposing a private restored directory. A retained SQL superset is allowed; missing referenced content/receipt or a mismatched store UUID refuses. Active state is recovered from the backed-up journal, never the newest SQL record or remote head. Original files are not overwritten during inspection/validation. Interrupted backup/restore leaves an unpublished private stage. A new reader never silently downgrades a version; an unsupported reader refuses before writes. This is operational backup restoration, not an engine undo across an adopted format barrier.

### 9.M. Native binding owner, operation and profile transition

M1. Managed project library definitions are resolved from `library/library.sqlite`, registered in the one DesignModel through typed `LibrarySourceLocatorV2={store_uuid:Uuid,asset:Ref,feature_path:[Uuid],local_object_id:Uuid}`. This is a declared database-backed source locator, not a fictitious pool JSON file. Local UUID registration uses K/D projection rules; reads/query results expose both locator and portable ref. A compatibility export of pool shards is not active native authority. Existing unmanaged pool shards retain their real legacy source locator until explicitly migrated.

M2. ComponentInstance shard target schema V2 retains the complete existing PersistedComponentInstance fields and adds required `revision_context_bindings:[BindingV2]`, sorted/unique by occurrence. Its enclosing record is `{schema_version:2,component_instance:<existing fields plus revision_context_bindings>}`. The component UUID in each binding agrees with its owner; source/instance paths distinguish repeated hierarchy occurrences. Existing library_bindings/part_ref become the contextual local-object projection for a managed occurrence, not independent UUID-to-latest authority. If one legacy singleton field cannot express different occurrence contexts, V2 consumers use revision_context_bindings and return an explicit ambiguity on an unqualified legacy query. No arbitrary first occurrence wins. Untouched schema-V1 unmanaged components keep legacy semantics; inspection reports coverage rather than assigning empty contexts as migrated assets.

M3. The target canonical operation is `ApplyLibraryAdoptionV2={link:AdoptionJournalLinkV2,direction:"forward"|"inverse",expected_occurrences:[BindingV2],result_occurrences:[BindingV2],source_changes:[SourceChangeV2]}`. Its batch includes existing typed native object/correspondence/pad/process changes and guards; it is not a standalone SQL UPDATE of bindings. Forward verifies receipt.before/after/source_changes and current expected project revision; inverse is allowed only as the canonical compensation of the authenticated original transaction and swaps the recorded before/after, source locators, exact local IDs, snapshots and native pre/postimages. Direction never changes sealed receipt bytes. Undo/replay use recorded result rather than regenerating geometry or IDs. The journal transaction and revision authority authenticate the whole composed batch, including the receipt link, consistent with existing physical-history rules.

The resolver gives each project-local ObjectId exactly one active source owner. SourceChangeV2.before must match its actual current locator, pool membership and exact legacy source-byte preimage under the project lease; after names the declared retained SQL source or restored legacy source. Canonical adoption atomically changes that source-owner map and component bindings. When Project.pools still enumerates an old shard, the resolver suppresses only the explicitly transferred object's old registration, retains its bytes as inert history/compatibility data and does not pick a winner by pool priority. Unaffected objects in the same shard remain active. Reuse an old definition's local ID only when no untouched legacy occurrence still needs that old source/revision and the owner transfer is unambiguous; otherwise keep its provider/local ID and allocate a distinct new definition local ID, preserving portable asset UUID and placed component/pad/wire identities. A second active owner, missing source preimage or unsatisfied remaining user refuses before publication. Recovery and leased backup retain source-change receipts/journal and all before/after locator content; Undo restores the exact prior source owner and membership without reallocation.

Membership fields are sorted unique normalized project-relative source/pool paths for this object's active provider, not a command to delete the entire Project.pools entry. SQL ownership uses `library/library.sqlite`; a null owner has empty membership. The complete legacy source bytes named by source_bytes_digest are retained in the receipt inventory before retiring that registration, so later edits to another object in that shard cannot destroy historical recovery evidence. Registration suppression is derived from authenticated canonical source_changes, not mutable file timestamp, pool priority or the newest SQL row. New source allocations use before=null; retirement uses after=null only when retained users/history rules permit it.

M4. Manifest-V3 adoption is separately prepared/accepted, with stable target_store_uuid and the exact original manifest bytes/hash. Under the project lease, prepare a private schema-V1 retention DB, verify supported reader/writer/integrity behavior, durably publish its UUID-declared destination, then use the existing canonical staged manifest/revision-authority publication to activate the V3 declaration. The existing canonical format-adoption transaction with its exact manifest pre/postimages, target store UUID and guarded AdoptFormatV3 request is the format receipt; there is no undefined second SQL format-receipt record. A crash before commitment preserves/reconstructs the last complete legacy baseline; a crash after canonical commitment recovers the coherent V3 baseline. An unused prepared DB is inert. This operation does not adopt a component revision or silently convert all legacy assets. It is forward-only and has no design Undo to manifest V2/V1. The accepted baseline can have zero new SQL-managed bindings while retaining old managed users through A6; coverage remains explicitly incomplete until those assets actually migrate.

M5. V3 project-root library declaration and ComponentInstance schema V2 are distinct from BoardRoot geometry profiles 1-4. The amendment neither renumbers those profiles nor removes PM057/058/061 exact geometry, frozen process, source/instance and compensation requirements. New arbitrary-shape effective process values require a declared versioned process snapshot capability; old readers refuse it rather than interpreting unknown shapes as rectangles. Required output adapters must qualify the declared subset with explicit unsupported-result refusal; a new library schema is not evidence of arbitrary-shape CAM support.

V2 snapshot capability identifier is `datum.process.snapshot.v2`, required in SnapshotV2.capabilities when a native_v2 process record is present. Its source_capability_ref retains the exact PM061 face/span/source-capability declaration used during calculation; the stackup reference alone is insufficient. Legacy_v1 records keep their original capability/version. Capability absence or unknown required capability refuses the affected preparation/output. Manifest V3 is the library/context reader barrier; it does not silently adopt a different BoardRoot geometry profile or new process basis.

### 9.N. Concrete implementation acceptance scenarios

These scenarios define behavior, not a new global qualification campaign or invented numerical bar. They are exercised through ordinary engine/UI/CLI workflows at the implementation phase; no execution occurred in this study.

1. Create a generic resistor component family with one electrical/symbol definition and multiple explicitly mapped chip/through-hole physical choices. Search/select a value+choice, place it, edit Properties, transfer to board, save/reopen offline. Verify one source-qualified component identity, correct map, net-bearing pads and preserved symbol/footprint choices without manual graph assembly.
2. Compose a fine-pitch array/perimeter footprint from repeated padstacks plus an explicit thermal land. Change pitch/row/group placement while preserving pad identities. Copper can be continuous while paste has several apertures. Compare localize-one-pad versus revise-shared-pad operations; unrelated users remain unchanged until selected updates.
3. Author centered rectangle/circle/capsule/rounded-corner and custom T contour geometry, including a re-entrant corner and separate mask/paste. Verify exact bounds, topology, half-nanometre centers, anchor/rotation/reflection and supported output. Invalid radius/topology/offset/capability returns the specific error and publishes no partial result. Unknown standards basis alone does not invalidate known geometry.
4. Place two component occurrences using different exact revisions of one shared symbol or padstack. Retain one portable asset identity, separate correct local projection IDs/contexts, unambiguous selection/map/check/CAM behavior and exact Undo/Redo/replay. A bare ambiguous UUID query refuses. Another project remains pinned to its original revision.
5. Revise a symbol gate/pin or footprint pad/process definition. Review where-used and exact diff, including mapping and affected checks. Stale preview/changed source generation refuses adoption. Accept updates only selected occurrences; preserve unrelated component/pad IDs, pose, wires/nets and board copper. Undo/reopen restore exact prior snapshots, including process/raw-policy/basis pins.
6. Instrument camera redraw/pan/zoom after preparation: library SQL read count, closure-resolution count and exact geometry-regeneration count remain unchanged solely because of camera movement. Evict/rebuild derived display caches without changing authoritative snapshot. Report actual cold/warm resource costs separately; no unmeasured speed claim.
7. Inject interruption before/during/after SQL retention and each canonical project publication point. Reconcile request/receipt/transaction identity, recover complete prior/new project state, never expose uncommitted retained content as adopted and never duplicate allocated IDs. Repeat/cancel/uncertain retry semantics match G/L. Disk-full/busy/corrupt-data faults produce truthful unchanged/committed/uncertain outcomes.
8. Back up a project with retained old/new revisions and undo/redo roots while a separate library workspace advances. Restore offline, verify exact backed-up journal tip/store UUID/receipts/content and reproduce supported outputs. A SQL superset is allowed; a missing referenced blob, wrong store UUID or unsupported schema refuses rather than fetching latest silently.
9. Migrate complete inventory including converted/refused catalog entries, native/user/legacy objects, package-embedded pads, maps, models, unknown fields and unmanaged V1/V2 bindings. Interrupt/resume repeatably, compare semantic IDs/content/status and preserve source originals. Refused inventory stays visibly unavailable, not falsely usable. A small QA closure or unchanged bundled catalog is not full migration success.
10. Open legacy formats without mutation; explicitly adopt V3, create a coherent baseline, then adopt assets. Unsupported readers refuse before writes; Cancel after accepted format adoption retains that baseline. Legacy export of unrepresentable new shapes/coexisting contexts refuses or requires an explicit disclosed conversion. Preserve supported inspection/revision/local exchange behavior with explicit dispositions for remaining compatibility limits.
