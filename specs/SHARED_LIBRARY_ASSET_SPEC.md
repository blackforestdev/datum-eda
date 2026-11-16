# Shared Library Asset Specification

Status: governed target specification, owner-directed 2026-11-16; not implemented.
Authority: [PM066](../docs/decisions/PRODUCT_MECHANICS_066_SHARED_ENGINEERING_LIBRARY.md).
Task: `dat-shared-library-spec-vvtz`. All requirements below describe target behavior.

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

SL-033: GitHub hosts proposed deterministic asset files, review discussions and
candidate release bundles. Binary local search databases/caches are not shared
source authority. A PR merge, tag, account permission or GitHub release does not
issue a Datum engineering approval or project release. Offline contribution
bundles use the same schema, checks and local authority path. Git is optional;
no core authoring/check/open operation requires a GitHub login or network.

SL-034: External material is quarantined until schema, identity, integrity,
dependency, rights-policy and compatibility checks complete; reviewable semantic
diff then canonical operations adopt it. No direct checkout writes into a live
project and no executable repository hooks run during acquisition. Reject unsafe
paths and unresolved/unsupported blobs before promotion. Existing source-rights,
egress and import boundaries remain; this specification grants no new dependency.

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

SL-050: One resolver-owned graph remains authority. Existing JSON/journal storage
is retained until a separately reviewed native-format migration decision changes
it. SQLite is a derived search facility, not a second approval/history authority.
The source/index split is acceptable only with the consistency requirements here;
this specification does not declare the current implementation sufficient.

SL-051: Search indexes identify the source snapshot/closure they represent. Build
or update transactionally into a complete generation; readers see the prior valid
generation or the new complete one, never a partial rebuild labeled current.
After a crash, mismatch or source mutation, detect staleness and rebuild/recover
or report unavailable/incomplete results. Never silently omit assets as though
the catalog were complete. Search and subsequent adoption verify the same subject
revision; changed content requires renewed validation, not stale row acceptance.

SL-052: Index replacement cannot mutate authored content/history/reviews. Engine
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

## 8. Decisions retained outside this amendment

The following are explicit boundaries, not invitations for agents to choose:

| Decision | Blocked scope until disposition |
| --- | --- |
| Shared repository owner, maintainer/reviewer authority, verified-vendor process and publication policy | Public collection publishing/verification |
| Asset licensing and retained document rights | Fetching, bundling or public redistribution of that content |
| Multi-pool shadowing, ranking and placement gate defaults retained by PM062 | New behavior in those areas |
| Persistent catalog architecture/resource/performance budgets | Storage replacement or claims beyond measured existing behavior |
| Exact first integration scope and done statement | Runtime integration and asset migration |

Owner-only decisions remain owner-only; an appointed PM may dispose only within
its verified appointment. Existing behavior and separately authorized work do not
acquire new blockers from unrelated policy decisions. For every implementation
task, cite applicable SL requirements and unsettled dependencies; retain current
behavior vs target truth. No agent may treat this spec as approval to invent
publisher authority, source rights or standards limits. SL-016–024 fix the V1
byte, envelope and lock profile; implementation still requires interoperability,
version-barrier and ordinary workflow proof before claiming support.
