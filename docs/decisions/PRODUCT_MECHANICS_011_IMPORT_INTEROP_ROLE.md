# Product Mechanics Decision 011: Import And Interop Role

Status: draft for owner review; implementation mechanisms woven 2026-06-18.
Date: 2026-06-18

Driven by:
- `docs/DATUM_PRODUCT_MECHANICS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_RESEARCH_SYNTHESIS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_REVIEW_AGENDA.md`
- `specs/STANDARDS_COMPLIANCE_SPEC.md`
- `docs/STANDARDS_AUDIT_BATCH_1_GUIDANCE.md`
- `docs/STANDARDS_COMPLIANCE_INTEGRATION_GUIDANCE.md`
- `docs/IPC_FOOTPRINT_SYSTEM.md`
- `research/standards-audit/STANDARDS_AUDIT.md`

## Decision Scope

Define the product role of import, export, interoperability, migration, and
reverse-engineering workflows.

This decision covers product identity and mechanics. It does not define every
format parser, exporter, or fidelity guarantee.

## Product Intent

Datum is a professional EDA application with optional first-class AI
collaboration.

Datum is not a KiCad importer, board viewer, format converter, or visualization
shell around other EDA files.

Import and export are interoperability infrastructure. They help users migrate
projects, inspect existing work, reverse engineer boards, exchange artifacts,
and collaborate with manufacturing or adjacent tools. They must not become the
center of the product model.

## Decision

Import shall be treated as interop, migration, audit, and reverse-engineering
support over Datum's native `DesignModel`.

Imported data must be converted into Datum primitives with explicit source
provenance, fidelity notes, unknown-basis markers, and relationship states.
Datum should preserve source geometry and metadata where possible, but it
should not let the source format define native architecture.

Export shall emit interchange artifacts or snapshots from Datum projections at
a known model revision. Exported artifacts are not the source of truth.

Native conversion is an edit-model operation, not a side-channel file rewrite.
An importer constructs Datum `DomainObject`s, `ImportProvenance`,
`ImportMap`, check findings, and optional migration proposals, then persists
the accepted conversion through the same `commit()` primitive and append-only
journal used by manual edits. Import-created IDs are stable Datum IDs. Source
IDs, paths, names, and hashes are provenance and import-map keys; they are not
native identity.

## User-Visible Behavior

Users should understand whether they are working with native Datum-authored
data, imported data, inferred data, or repaired data.

Expected behavior:
- import wizard/review surfaces list source format, source files, parser
  version, unsupported features, warnings, and lossiness
- imported objects carry source provenance and stable Datum object IDs
- imported board-first relationships may start with `RelationshipKind`
  `ReverseEngineered` or `BoardOnly`
- imported geometry is preserved unless the user accepts a repair proposal
- standards-aware import audits can compare imported footprints and process
  apertures against declared, inferred, or user-selected basis
- export jobs show model revision, projection, selected format, settings, and
  validation state
- users can continue editing imported projects as Datum projects after
  conversion

## Manual Workflow Requirements

Datum must support import and interop workflows without AI.

Manual workflows must include:
- import supported source files into a Datum project or project-local review
  session
- inspect import report and source provenance
- navigate from warnings/findings to affected objects
- assign missing library bindings, symbols, footprints, parts, packages, or
  pin/pad maps manually
- declare unknown basis or select an intended standards/process basis for audit
- accept, reject, or defer import repair proposals
- mark physical-first data as reverse-engineered, board-only, or pending
  electrical reconstruction
- export manufacturing and interchange artifacts from live projections with
  validation reports

## Optional AI And Tooling Behavior

AI and tools may help interpret imported data, but import must remain a
deterministic, reviewable workflow.

Allowed behavior:
- summarize import warnings and likely root causes
- infer candidate package families or standards basis for user review
- propose library bindings and pin/pad maps
- propose process-aperture corrections for imported host-tool defaults
- assist reverse-engineering by suggesting schematic reconstruction from board
  connectivity
- compare exported artifacts with prior revisions

Required guardrails:
- AI may not silently repair imported geometry
- AI may not present inferred schematic intent as authored truth
- AI may not erase source provenance or lossiness markers
- AI may not use external services for imported controlled data unless the
  project's data-egress policy allows it
- all AI-assisted import fixes must produce proposals with diffs and
  provenance

## Core Primitives

### ImportSession

One import attempt from one or more source files.

Required fields:
- source format and version where known
- source files and hashes
- importer version
- import settings
- created Datum project/model revision
- warnings, errors, and unsupported features
- lossiness summary
- provenance map

Implementation schema concept:

```text
ImportSession
  id: ObjectId
  source_format
  source_format_version?
  source_files[]
  source_hashes[]
  importer_name
  importer_version
  import_settings
  target_project_id
  created_model_revision?
  committed_transaction_id?
  warnings[]
  errors[]
  unsupported_features[]
  lossiness_summary
  import_map_ref
  provenance
```

An import session may be preview-only or committed. Only a committed session
contributes to the resolved `DesignModel`, and it does so by recording a
transaction in the journal; neither the session nor its source files is a
source authority. Preview findings and proposals are review state until
accepted.

### ImportProvenance

Source metadata retained on imported objects.

Minimum fields:
- source format
- source file
- source object identity/path where available
- import session ID
- original values where needed for audit
- conversion notes
- unknown-basis flags

Implementation schema concept:

```text
ImportProvenance
  id: ObjectId
  import_session_id
  source_format
  source_file
  source_object_key?
  source_path?
  source_hash?
  original_values{}
  conversion_notes[]
  lossiness_flags[]
  unknown_basis_flags[]
  preserved_geometry_refs[]
  repaired_by_transaction_ids[]
```

Import provenance remains attached after a project becomes native Datum data.
Repair operations append provenance; they do not erase the preserved source
record.

### ImportMap

Stable mapping from source identities to Datum object identities.

Implementation schema concept:

```text
ImportMap
  id: ObjectId
  import_key
  source_format
  entries[]
    source_object_key
    datum_object_id
    object_kind
    source_revision_or_hash?
    first_seen_session_id
    last_seen_session_id
    status: Active | MissingInSource | Replaced | Split | Merged
```

`import_key`, not `source_hash`, is the durable map key. A changed source file
should reuse stable Datum IDs when the source object identity still represents
the same object. `source_hash` records evidence about a specific source
version; it must not force all imported IDs to churn after a source edit.

### LossinessRecord

Structured record of unsupported, approximated, inferred, or intentionally
omitted source data.

Implementation schema concept:

```text
LossinessRecord
  id: ObjectId
  import_session_id
  source_feature
  affected_source_objects[]
  affected_datum_object_ids[]
  lossiness_kind: Unsupported | Approximated | Inferred | Dropped | SemanticsUnknown
  severity
  explanation
  proposed_followup?
```

Lossiness is reportable and checkable. Severe lossiness may block commit if it
would create misleading native truth; lesser lossiness can commit with visible
findings and repair proposals.

### InteropArtifact

Generated or imported exchange artifact.

Examples:
- KiCad/Eagle files
- Gerber/Excellon
- IPC-2581
- ODB++
- STEP/IDF
- IPC-D-356A
- BOM/PnP

Artifacts must record model revision, projection, settings, validator output,
and provenance.

Exported artifacts are snapshots from projections. If an exported artifact is
re-imported later, it starts a new import session with provenance pointing to
that artifact; it still does not become a second source authority.

### ImportAuditFinding

Check finding produced by import review.

Examples:
- unsupported source feature
- geometry preserved but semantically ambiguous
- missing pin/pad map
- unknown footprint basis
- pad/mask/paste policy mismatch
- inferred schematic mismatch
- library binding unresolved

Import audit findings must use the same check primitives as other Datum
findings.

Implementation schema concept:

```text
ImportAuditFinding
  check_finding_ref
  import_session_id
  source_object_key?
  import_provenance_ref
  lossiness_record_ref?
```

Import findings are regular `CheckFinding`s with import context. They can be
waived, documented as deviations, or resolved by accepted repair proposals, but
the source provenance remains inspectable.

### MigrationProposal

Reviewable edit that converts, repairs, normalizes, or enriches imported data.

Examples:
- assign library part
- create missing package
- attach symbol to board-only component
- correct process aperture
- normalize bus syntax
- accept reverse-engineered connectivity

Implementation schema concept:

```text
MigrationProposal
  proposal_id
  import_session_id
  source_finding_refs[]
  operation_batch
  rationale
  expected_fidelity_effect
  source_values_preserved[]
  risks[]
  provenance
```

Migration proposals use normal `OperationBatch` and `Proposal` mechanics from
001. Accepted proposals commit through the journal and record both original
import provenance and repair provenance. Rejected or deferred proposals leave
the imported object untouched.

### Relationship State (RelationshipKind) for imported board-first data

`RelationshipKind` values for data that starts from physical implementation
rather than complete electrical intent. This is not a separate
reverse-engineering state machine; it reuses the ratified `RelationshipKind`
enum directly:
- `ImplementedBy`
- `ReverseEngineered`
- `BoardOnly`
- `SchematicOnly`
- `Pending`
- `Mismatch`

`ReverseEngineered` is one authored `RelationshipKind` among the ratified set,
not a distinct authored-intent, deviation, or reverse-engineering axis. These
labels are authored `RelationshipKind` state, not object identity, and they are
kept separate from resolver-derived status (`Implemented`,
`PendingImplementation`, `UnresolvedMismatch`), which is recomputed on load and
never persisted here. A board-only pad, track, component placement, or inferred
net can later be bound to electrical intent through an accepted relationship
operation. That relationship resolves on the surviving `ComponentInstance`
surrogate identity, not on the reference designator, `source_object_key`, or
`source_path` the importer carries; those source keys remain provenance only.
Until bound, the object remains visible as physical source truth with incomplete
or inferred electrical meaning. Any extra explanation about source basis,
confidence, lossy conversion, repair history, or intentional divergence is
stored as import/provenance metadata or as an accepted-deviation record, not as
a second `ReverseEngineered` axis.

## Native Conversion Mechanics

Import conversion follows one path:

1. Parse source files into a source-format AST/IR with file hashes and parser
   version recorded.
2. Allocate or reuse stable Datum `ObjectId`s through the `ImportMap`.
3. Convert recognized entities into Datum primitives: parts, packages,
   symbols, footprints, padstacks, pin-pad maps, component instances, sheets,
   boards, zones, tracks, vias, rules, standards basis, manufacturing plans,
   and artifacts as applicable.
4. Attach `ImportProvenance`, unknown-basis markers, and `LossinessRecord`s to
   affected objects.
5. Preserve source geometry by default, including pad, mask, paste, drill,
   zone-boundary, and source-tool zone-fill evidence.
6. Run import audit checks and emit `CheckFinding`s with import context.
7. Produce migration/repair proposals for mechanically actionable fixes.
8. Commit the accepted conversion through `commit()`, which stages shards,
   appends the transaction record to the journal as the commit point, and
   advances object/model revisions.

On subsequent open, the committed import shards (including the Import Map and
`ImportProvenance`) are reassembled by `ProjectResolver` into the resolved
`DesignModel`; the resolver validates Import Map `import_key` references and
provenance, and an incoherent or split imported project opens in
recovery/diagnostic mode rather than as accepted truth. Reload, reimport, and
resolved-model context for an imported project are always served from this
`ProjectResolver`-assembled model at a known `model_revision`, never from the
source shards read directly.

Imported electrical-to-physical relationships bind through the
`ComponentInstance` surrogate identity shared by `PlacedSymbol` and
`PlacedPackage`, not through `source_object_key`, `source_path`, reference
designators, or any other source key; those source keys remain provenance only.

Source-tool architecture must end at step 3. After commit, users edit a native
Datum project. Source names, paths, and feature encodings remain provenance,
not a parallel authority.

Zone fill conversion must be honest. Imported fabricator-accurate fill islands
may be stored as derived `ZoneFill { state: Filled }` with import provenance
and source hashes; the authored source zone remains the boundary. Unsupported
or unfilled native zones must not export or check as filled copper.

## Standards And Compliance Impact

Import must preserve the difference between source fidelity and standards
validation.

Datum may successfully import a design that does not pass Datum checks. The
source tool accepting a file does not mean the design satisfies the selected
Datum rules, IPC/process assumptions, or project standards posture.

Required behavior:
- preserve imported geometry and metadata by default
- identify unknown standards basis explicitly
- compare imported footprints, padstacks, mask, paste, courtyard, clearance,
  and naming observables against declared or user-selected basis where possible
- emit import audit findings rather than mutating source-derived geometry
- require proposals for import repair or normalization
- record accepted repair provenance separately from original import provenance
- avoid false claims that an imported project is certified or compliant because
  it was converted

Pad/mask/paste import is specifically provenance-sensitive. If a source tool
omits explicit paste or mask apertures, inherits them from copper, or stores
source-tool defaults, Datum records that basis. Standards-aware import audits
may flag the observable delta, but they offer `SetPadProcessAperture`,
`ApplyFootprintProcessPolicy`, or deviation proposals rather than mutating the
imported geometry during conversion.

## Non-Goals

This decision does not:
- make import the product center
- require native binary parsers for every commercial EDA format
- require Datum to mirror source-format architecture internally
- guarantee lossless import for every feature of every supported tool
- treat imported schematic intent, board geometry, or library metadata as
  automatically trustworthy
- silently rewrite imported data into Datum-preferred style
- make export artifacts authoritative source files

## First Proof Slice

The first proof slice should demonstrate import as reviewable migration:

1. Import a small board or library sample into native Datum primitives.
2. Store object-level import provenance and source hashes.
3. Produce an import report with unsupported/lossy/unknown-basis findings.
4. Run one standards-aware import audit for pad/mask/paste policy.
5. Offer a migration proposal that repairs one process-geometry issue while
   preserving the original imported values in provenance.
6. Export one artifact from the Datum model with model revision and validation
   metadata.

## Owner Questions

- Which import use case is the first product slice: KiCad migration, Eagle
  migration, imported-board audit, library import, or reverse engineering?
- How visible should import provenance remain after a project becomes native
  Datum-authored?
- What source-format lossiness is acceptable before import should fail instead
  of warn?
- Should imported geometry default to stricter unknown-basis warnings than
  native Datum geometry?
- Which export formats are product-critical versus future interop targets?
- How should reverse-engineered schematic reconstruction be labeled so users do
  not confuse inferred intent with authored electrical design?
