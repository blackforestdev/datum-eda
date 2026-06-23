# Product Mechanics Decision 000D: Storage And Versioning Model

Status: draft hypothesis + how/mechanism woven 2026-06-18; identity,
ComponentInstance, journal, and M7 sequencing ratified; remaining forks open.
Date: 2026-06-18

Driven by:
- `docs/decisions/PRODUCT_MECHANICS_000_UNIFIED_DESIGN_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_000B_VIEW_COMPOSITION_AND_LIVE_PRODUCTION.md`
- `docs/decisions/PRODUCT_MECHANICS_000C_UNIFIED_MODEL_FEASIBILITY.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_RESEARCH_SYNTHESIS.md`
- project-owner concern that the unified model must remain practical for
  versioning, git diffs, collaboration, generated artifacts, and migration

## Purpose

Define how Datum has one canonical `DesignModel` authority — the single
authority, assembled in memory by an engine-owned `ProjectResolver` — while
persisting project state as deterministic source shards. Shards are
persistence partitions, never authorities.

This document carries the storage/versioning hypothesis as
pending-owner-review, with four mechanism choices ratified 2026-06-18 and
stated here as decided: (1) object identity is a persisted surrogate with a
`Uuid` wire type (`ObjectId = Uuid`, persist-on-create); (2)
`ComponentInstance` is introduced now as the canonical electrical-to-physical
join; (3) the transaction journal is required source state from the first
cut, with durable undo/redo as cursors into it; (4) the identity, resolver,
and commit modules are built concurrently with M7 — they do not close M7. The
native source shards are the only authority; import is a one-time converter
that writes those shards directly (see
`docs/DATUM_PRODUCT_MECHANICS.md` "Interop Boundary And Import Posture"). An
optional from-shards KiCad emitter is a separate, deferred export path, not a
migration step. The single authority over segmented shards remains the leading
hypothesis pending the proof gates below; the four mechanism choices above are
no longer open.

## Product Intent

Datum should solve the workflow pain caused by segregated schematic and PCB
worlds without creating one giant unmergeable project blob.

The product goal is:

> One design authority, many deterministic storage shards.

The user should experience one coherent project model. The filesystem should
remain practical for review, version control, object-level diffs,
collaboration, and generated artifact traceability.

## Storage Approaches Compared

### A. Monolithic File Or Database

All project state is persisted in one file or database.

Benefits:
- strongest single-authority semantics
- simple loader boundary
- simple global revision identity
- easier to enforce cross-domain invariants

Risks:
- poor git diffs
- high merge-conflict risk
- small edits can rewrite large blobs
- harder manual inspection
- harder partial recovery
- generated artifacts and workspace state can accidentally churn source state

Use only if Datum intentionally chooses a database-backed project model and
accepts the version-control tradeoffs.

### B. Conventional Separate Schematic/PCB Files

Schematic, PCB, library, manufacturing, and output data are separate documents
with synchronization between them.

Benefits:
- familiar EDA model
- easy import/export mapping
- simple conceptual ownership per file
- aligns with existing segmented tools

Risks:
- repeats split-authority EDA doctrine
- requires ECO/sync/back-annotation discipline
- encourages schematic and PCB drift
- makes AI/tooling reason across documents rather than one model
- live manufacturing projections become downstream viewers instead of model
  projections

Use only if feasibility work shows unified authority creates more risk than it
removes.

### C. Unified Authority With Segmented Storage Shards

The project loader resolves many deterministic files into one canonical
`DesignModel`.

Benefits:
- preserves one design authority
- keeps diffs smaller and more reviewable
- supports multiple schematic sheets, boards, manufacturing plans, and output
  jobs
- lets generated artifacts remain snapshots instead of source truth
- supports AI/tooling through one resolved model

Risks:
- requires a strong loader/resolver
- requires stable object IDs and schema discipline
- shard boundaries can drift into hidden authority boundaries
- cross-shard transactions need precise provenance and validation

Preliminary posture:
- Treat C as the leading candidate.
- Do not consider it accepted until the proof gates in this document pass.

## Proposed Storage Shard Classes

### Project Manifest

Owns project-level identity and references.

Contains:
- project ID
- project name and metadata
- schema version
- storage layout version
- enabled standards/process basis
- shard index
- root model revision
- active variants and output contexts

The manifest points to shards. It must not duplicate their source data.

### Electrical Sheets

Own visual and authored electrical sheet data.

Contains:
- sheet identity
- placed symbols and graphical positions
- wires, buses, labels, ports, junctions, no-connects, and annotations
- sheet-local presentation state
- references to library objects and stable design object IDs

Electrical sheets are authoring/presentation shards. They are not independent
electrical authorities.

### Resolved Electrical Graph Or Cache

Owns derived electrical connectivity when persisted.

Contains:
- resolved nets
- component/pin connectivity
- hierarchy resolution
- bus expansion
- variant-aware connectivity
- graph revision and source input revisions

This shard is a cache unless Datum explicitly promotes part of it to authored
source state. If persisted, the loader must be able to validate whether it is
current or stale.

The graph shard is a cache; net identity is not. To reconcile these — a
derived graph that must nonetheless carry stable identity across rename,
split, and merge — Datum separates the resolved connectivity (derived,
recomputable, cacheable) from the net's identity anchor (authored, stable).
A net has a stable `NetId` allocated like any other object. A `NetAnchor`
binds a resolved connectivity group to that `NetId` via an authored hint: a
specific label object, a specific wire endpoint, or, for anonymous nets, the
lowest-`ObjectId` pin in the group. Recomputing the graph re-derives which
pins, labels, and wires belong together, then re-binds each group to its
`NetId` through the surviving anchor. The anchor is a resolver-refreshable
derivation except where the user placed it explicitly. Renaming a net changes
a display name; the anchor is unchanged, so the `NetId` is unchanged.
Splitting a net by deleting a wire keeps the `NetId` on the sub-group that
still contains the anchor and allocates a single new `NetId` for the orphaned
group. Merging two nets selects a deterministic survivor — lowest `NetId`
wins — and records the merged-away `NetId` in the committing transaction. If
an anchor object is deleted, the resolver re-derives a fresh anchor for the
surviving group and records the transition. The cache records its graph
revision and the object revisions of every input it consumed; on load, the
resolver marks the cache stale and recomputes whenever any input object
revision exceeds the recorded value. This replaces the prior behavior in
which net identity was the label name recomputed from scratch on every query,
which made identity change on rename and made split and merge identity
unpredictable.

### Physical Boards

Own physical board implementation data.

Contains:
- board identity
- outline
- stackup reference
- component placements
- footprints or footprint instance references
- pads, tracks, vias, zones, keepouts, copper, text, dimensions
- board-level rules and overrides where allowed

A project may contain more than one board. A board is a physical
implementation shard, not the whole project authority.

A zone's fill is derived state, not board authority. `Zone.polygon` is the
authored boundary; the fill is carried separately as a derived `ZoneFill`
{state: Filled | Unfilled | Stale | Unsupported, islands} alongside a new
`Zone` provenance field that records where the fill came from. A fill
projection records the revision of its source zone and a hash of the inputs
that produced it, so the loader and the equivalence harness can decide whether
a cached fill is current, stale, or must be recomputed — exactly as the
resolved-net graph is treated. For a board brought in through the import
converter, the fabricator-accurate islands the source tool computed are
carried as a `Filled` projection with import provenance attached; they are
trusted geometry but remain derived-with-provenance, not re-authored board
truth. Native un-filled zones
project as a badged `Unfilled` placeholder and emit no copper; the first
live-CAM proof does not author native pour fill (if later authorized, the
polygon-boolean substrate is `i_overlay`, pure-Rust MIT, with no FFI creep).
The unfilled-zone export policy is to emit nothing plus a hard finding — not
the solid-copper-boundary that is today's accidental behavior.

### Library Bindings

Own project use of library objects.

Contains:
- part/entity/package references
- symbol references
- footprint references
- padstack references
- pin-pad maps
- model attachments
- pinned library revisions
- project-local overrides

Library bindings must identify the exact revision used by the project.

### Relationships

Own cross-domain relationship state.

Contains:
- electrical-to-physical implementation bindings carried on a
  `ComponentInstance` (a per-instance stable surrogate `Uuid` that both the
  schematic `PlacedSymbol` and the board `PlacedPackage` reference; it is the
  canonical electrical-to-physical join, distinct from the library `part`
  field, and replaces the reference-designator string that informally serves
  that role today)
- symbol-to-part bindings
- component-to-footprint bindings
- pin-to-pad mappings
- `Relationship` records (authored bindings with a surrogate id) of
  `RelationshipKind` {ImplementedBy, BoardOnly, SchematicOnly,
  ReverseEngineered, Pending, Mismatch}
- relationship status, split into a DERIVED axis {Implemented,
  PendingImplementation, UnresolvedMismatch} and an AUTHORED-intent axis
  {LayoutDeviation, BoardOnlyObject, SchematicOnlyObject, accepted-deviation}
- affected object IDs

`ReverseEngineered` is a `RelationshipKind`, not a second authored-intent or
deviation axis. Board-first and import workflows that need to explain source
basis, confidence, inference, or repair history use import/provenance metadata
and accepted-deviation records attached to affected object IDs.

This shard is critical. It prevents electrical and physical shards from
becoming separate authorities.

The relationships layer is critical precisely because so many edits reach it
— move-component, draw-track, edit-wire, change-footprint, and accepted DRC
corrections all record relationship state. Persisting that state as one
document would make it a merge-conflict magnet and defeat the goal that
unrelated edits touch unrelated shards. Datum therefore does not store
relationships as one file (000D defers final layout, so this is a design
choice rather than a fixed serialization rule). Each relationship record is
co-located with the object that owns it and appended to a per-owner,
per-domain-pair log keyed on the owning object's stable `ObjectId`. Moving
`U12` appends only to `U12`'s relationship log; routing a net appends only to
that net's. Two agents editing different objects append to different files and
merge cleanly; a genuine conflict can only occur when two agents change the
same relationship of the same object, and that collision is detected at record
granularity and surfaced as an `UnresolvedMismatch` for explicit arbitration
rather than silently merged. (The first cut is single-writer-per-project; the
per-owner partitioning is the mechanism that keeps the layer safe once
multi-writer is built — reserved by design, not yet implemented.)

### Rules And Constraints

Own executable and declarative design requirements.

Contains:
- electrical rules
- physical rules
- manufacturing/process rules
- standards basis
- rule scopes
- constraint classes
- waivers and deviations
- sign-off requirements

Rules must be available to editing tools, checks, proposals, agents, and live
manufacturing projections.

### Manufacturing Plans

Own production intent.

Contains:
- board or panel production targets
- assembly variants
- CM/fab process assumptions
- panel strategies
- rails, tabs, mouse bites, V-scores, fiducials, tooling holes, coupons, and
  manufacturing-only notes
- production checks and sign-off expectations

Panelization belongs here. It must not mutate source board geometry unless the
user explicitly requests a board-level change.

### Output Jobs

Own artifact generation recipes.

Contains:
- Gerber/NC drill settings
- IPC-2581/ODB++ settings
- BOM/PnP settings
- drawing/export settings
- board, panel, variant, and manufacturing-plan targets
- validation requirements

Output jobs define how artifacts are generated. They are source state.

### Proposals

Own uncommitted candidate changes.

Contains:
- proposed operation batch
- affected object IDs
- rationale
- checks run
- preview/diff metadata
- risk notes
- proposed-by provenance
- apply/reject/defer state

Proposals are not committed design state until accepted.

### Transactions

Own committed change history. The journal is required source state from the
first cut, not an optional add-on.

Contains:
- transaction ID and the parent-txn DAG field (present so multi-writer is
  reserved by design; the field is recorded but the first cut is
  single-writer-per-project)
- operation batch (the `OperationBatch` that produced the change)
- affected object IDs and shard IDs with before/after hashes
- user/tool/AI/import provenance
- validation state
- the inverse data needed for undo (the tagged `InverseBatch`)
- the caches the transaction invalidates
- the resulting model revision

The Transactions shard is not merely a passive audit record; it is the commit
primitive that makes segmented storage safe. Because nearly every edit touches
several shards at once — the Common Edit Shard Impact examples show physical,
relationship, transaction, and cache shards moving together — a save that
writes those shards one at a time is a crash hazard: an interruption can leave
one shard new and another stale, producing exactly the split truth this model
forbids. Datum therefore commits across shards atomically through one
`commit()` primitive that every mutation surface (GUI/CLI/MCP/AI/importer/
checker) funnels through. A transaction applies its `OperationBatch` in
memory, then stages the new bytes for every touched shard into a transient
staging area and flushes them durably (fsync), then appends a single
`TransactionRecord` to an append-only journal and flushes it — that journal
append is THE COMMIT POINT. Only afterward are the staged shard files promoted
into place by atomic rename (with a directory fsync). On open, the loader
replays the journal tail: if the last record's recorded shard hashes match
what is on disk, the transaction is complete; if staged bytes remain for a
journaled transaction, the loader rolls forward and finishes the promotion; if
the journal has no complete final record, the staged bytes are rolled back and
discarded. A project can thus only ever resolve to a state that some committed
transaction produced, and a project that cannot be resolved opens in recovery
mode rather than silently accepting partial truth. Durable undo and redo are
cursors into this journal — undo is itself recorded as a compensating
transaction (a tagged `InverseBatch`), keeping history append-only and
auditable — so the same structure serves atomic commit, durable undo/redo, and
cross-shard audit at once.

### Artifacts Metadata

Own metadata about generated outputs.

Contains:
- artifact ID
- artifact type
- source model revision
- source board/panel/variant/output job
- generator version
- generated file paths or content hashes
- timestamp
- validation/check status

Artifact metadata may be source state. Generated artifact files are snapshots,
not design authority.

### Workspace And Layout State

Own user/workbench presentation state.

Contains:
- open tabs
- panes and layout graph
- PiP/floating/sidecar state
- camera positions
- selected workbench profile
- recent navigation stack

Workspace state must be isolated from design authority so UI changes do not
churn core design diffs.

## Stable Object Identity Mechanism

Datum needs stable object identity across projections, shards, transactions,
and artifacts.

Required rules:
- Every source domain object has a stable object ID.
- Object IDs are generated by Datum and do not depend on display name,
  reference designator, file path, or array order.
- Renaming an object does not change its ID.
- Moving an object between sheets, boards, panes, or projections does not
  change its ID unless the object is semantically replaced.
- Imported objects retain source provenance and receive native Datum IDs.
- Generated projection objects reference source object IDs and generator
  context.
- Relationship objects use stable IDs for both sides of the relationship.
- Transactions reference object IDs and shard revisions, not only file paths.

Names, reference designators, net labels, and paths are user-facing
properties. They are not identity.

The rules above state what must be true; this section states how Datum makes
them true. Object identity is a persisted surrogate, `ObjectId`, persisted on
create and never derived from display name, reference designator, net label,
file path, or array order. For the first cut the wire type is `Uuid` (the
allocated integer `ObjectId(u64)` is a deferred diff-size optimization, behind
a global-vs-project-local owner call that is still open). Native objects keep
the v4 UUID they generate once at creation and PERSIST it as a field, so it is
stable across reload; because name and `sheet_ref` are non-identity fields,
rename and move become identity no-ops that mutate one field and leave
identity untouched. References between objects (a pin to its net, a
relationship to both its sides) store `ObjectId`s, so a rename or move
re-points nothing and yields a one-field diff.

Imported objects keep the deterministic v5 UUID, but it is DEMOTED from
identity to an import SEED. On first import the seed is persisted into a
net-new Import Map shard keyed by `import_key` — NOT by `source_hash`.
Re-importing the same source reuses those identities by `import_key`, so
identity is deterministic across imports even when the source file is edited.
This deliberately fixes the real lost-on-edit defect, which is NOT the dead
`ids_sidecar` module: the running rules, part, package, and net-class overlays
gate their reload on a whole-file `source_hash` and then silently drop the
overlay on mismatch via a no-op `Ok(_) => {}` else branch — see
`crates/engine/src/api/project_surface.rs:36`, `:55`, `:75`, and `:95`, where
`sidecar.source_hash == source_hash` is the load guard and any edit to the
source file invalidates the hash and discards the user's authored overlay.
Keying on `import_key` instead survives source edits. (Implementers: do not
send identity work to `ids_sidecar`; the live defect is in the overlay reload
path in `project_surface.rs`.)

A `ComponentInstance` surrogate (a per-instance stable `Uuid`, distinct from
the library `part` field) is created and linked at import and at forward
annotation, and both the schematic `PlacedSymbol` and the board
`PlacedPackage` reference it. It is the canonical electrical-to-physical join,
replacing the reference-designator string that informally serves that role
today.

The surrogate-key approach is the same decoupling used by KiCad net-codes, the
Horizon EDA pool, and conventional database design, and it is the only
construction that is simultaneously rename-stable, deterministic on import, and
small in diff.

## Revision Classes

Transactions are the SOLE producer of revisions. No surface bumps a revision
directly; a revision exists only because a committed transaction produced it.
This is what lets every cache, projection, and artifact key its staleness off
revisions without a second bookkeeping path. Until revisions land in the
engine, downstream caches may fall back to hashing their input content
directly; the revision scheme is the durable replacement for that fallback.

### Model Revision

Revision of the resolved `DesignModel`.

A model revision identifies the full project state after resolving all source
shards, schema versions, relationships, and accepted transactions. It is
computed as a sha256 over the canonical, sorted set of every object's
`object_revision` plus the accepted-transaction tip, so the model revision is
a deterministic function of object state and journal position — not a
separately maintained counter.

### Object Revision

Revision of a single source domain object — a monotonic `u64` bumped per
object by the transaction that changes it.

Object revisions allow checks, projections, and artifacts to know whether a
specific source object changed, and they are the inputs the model revision
hashes over.

### Transaction Revision

Revision created by a committed operation batch.

A transaction revision records the operation, affected objects, affected
shards, provenance, validation state, and resulting model revision.

### Artifact Revision

Revision of a generated snapshot.

An artifact revision ties generated output to:
- model revision
- output job revision
- board or panel revision
- variant revision
- generator version
- validation state

### Library Revision

Revision of a library object used by the project.

Project library bindings must pin library revisions or explicitly record local
overrides.

### Variant Revision

Revision of a single variant overlay shard.

A variant is a sparse authored overlay shard keyed by stable object identity:
a population map (object to `Fitted` or `Unfitted`), the option-link
selections the variant makes, and any variant-specific relationship-state
overrides. Population is three-valued — `Fitted` and `Unfitted` are authored;
`NotApplicableForVariant` is derived from option-link/scope resolution and is
never stored. Because the overlay holds only deviations from the canonical
model, changing the active variant changes which overlay is composed over the
base authority; it does not rewrite the base model and it touches no object the
variant leaves at its default (zero base writes). This is why switching a
variant does not look like editing every object in every sheet or board: the
only storage that moves is the variant shard, and a variant-to-variant
comparison is a diff of two small overlay maps that surfaces exactly the
objects whose population, option, or relationship state differs.

Editing a variant bumps that variant's revision and invalidates only that
variant's derived check and projection caches; editing the base model bumps
the model revision and invalidates every variant's derived state, since all
variants are projections of the one authority. Generated artifacts therefore
identify both the model revision and the variant revision, and that pair is
the same key under which resolved variant state is cached, so artifact
identity and cache identity coincide rather than being maintained separately.
Variant revisions affect BOM, PnP, assembly drawings, simulation setup,
physical implementation expectations, and output artifacts.

## Common Edit Shard Impact

The following examples define expected storage behavior. They are not final
serialization requirements.

### Move Component In Board Layout

Expected shard touches:
- physical board shard
- relationships shard if implementation state or deviation state changes
- transactions shard
- derived check/projection cache invalidation

Should not touch:
- electrical sheet shard unless electrical intent changed
- library binding shard unless the part/footprint binding changed
- generated artifact files unless export is explicitly run

### Draw Track

Expected shard touches:
- physical board shard
- relationships shard if route completion state changes
- transactions shard
- affected DRC/manufacturing projection invalidation

Should not touch:
- electrical sheet shard unless the user explicitly changes electrical intent

### Edit Wire

Expected shard touches:
- electrical sheet shard
- resolved electrical graph cache invalidation or update
- relationships shard if implementation state becomes pending or mismatched
- transactions shard
- ERC/DRC invalidation as needed

Should not touch:
- physical board shard unless an explicit physical update proposal is accepted

### Rename Net

Expected shard touches:
- electrical sheet shard or electrical intent shard
- resolved graph cache
- relationships shard for net identity/display-name mapping if needed
- transactions shard

Important rule:
- If the net object remains the same, the stable net object ID should not
  change. The display name changes.

### Change Footprint Or Padstack

Expected shard touches:
- library binding shard or project-local library override shard
- physical board shard if instances are updated in-place
- relationships shard for affected component/footprint/pin-pad mappings
- rules/checks invalidation
- transactions shard

Preferred behavior:
- Library updates should produce proposals when existing physical designs or
  manufacturing outputs are affected.

### Modify Panelization

Expected shard touches:
- manufacturing plan shard
- output job shard only if output recipe changes
- transactions shard
- panel/manufacturing projection invalidation

Should not touch:
- source board geometry unless the user explicitly applies a board-level
  change.

### Change Output Job

Expected shard touches:
- output job shard
- artifact metadata if a new artifact is generated
- transactions shard
- live output projection invalidation

Should not touch:
- electrical sheet shard
- physical board shard
- library binding shard

### Change Active Variant

Expected shard touches:
- variant overlay shard (the sparse authored overlay only, when its
  population/option/relationship-override state is edited)
- transactions shard
- that variant's derived check/projection cache invalidation

Should not touch:
- the base model — switching the active variant composes a different overlay
  over the base authority with zero base writes
- any object the variant leaves at its default state

### Accept DRC Correction

Expected shard touches depend on correction type.

For pad mask/paste correction:
- physical board shard or footprint/project-local library shard
- rules/checks shard if waiver/deviation/sign-off state changes
- relationships shard if implementation or standards state changes
- transactions shard
- affected manufacturing projections invalidated

Required behavior:
- The original imported geometry provenance remains available.
- The accepted correction records user/tool rationale and basis.

### Attach Model

Expected shard touches:
- library binding shard or model attachment shard
- simulation/analysis setup shard if applicable
- relationships shard if model coverage state changes
- transactions shard

Should not touch:
- electrical or physical geometry unless the attachment also includes an
  accepted geometry update.

## Generated Artifact Policy

Generated artifacts are snapshots, not source authority.

Required rules:
- Gerber, NC drill, IPC-2581, ODB++, BOM, PnP, PDF, STEP, reports, and
  assembly packages are generated from model state and output jobs.
- Artifacts must record the source model revision.
- Artifacts must record the output job revision and generator version.
- Artifacts must identify board, panel, variant, and manufacturing-plan
  context.
- Live production projections and exported artifacts must share the same
  generation core or a provably equivalent one.
- Generated artifact files should not be required for opening or editing the
  project.
- Artifact metadata may be versioned for traceability.
- Artifact files may be ignored, stored, or released depending on project
  policy, but they must not become source truth.

## The Resolver

Segmented storage is only acceptable if the loader/resolver enforces one
model authority. The `ProjectResolver` is the single engine-owned component
that discharges that obligation.

Required loader/resolver checks:
- all shard schema versions are compatible
- shard IDs match the manifest index
- referenced object IDs exist or are explicitly external/imported
- relationship states are valid for referenced objects
- physical implementation bindings resolve to electrical intent or explicit
  board-only/reverse-engineered states
- library bindings resolve to pinned revisions or project-local overrides
- rules and constraints resolve to valid scopes
- output jobs resolve to valid board/panel/variant/manufacturing contexts
- proposal and transaction references resolve to known object and shard
  revisions
- generated caches are current or marked stale
- artifact metadata references known model and output job revisions

Required product rule:

> Files are persistence partitions. The resolved `DesignModel` is the
> authority.

If a shard cannot be resolved into a coherent model, Datum should open in a
recovery or diagnostic mode rather than silently accepting split truth.

The checks above describe obligations; this is the resolver that discharges
them. The `ProjectResolver` loads the Project Manifest, then loads source
shards in dependency order — manifest, library bindings, electrical sheets,
physical boards, relationships, rules, manufacturing plans, output jobs, and
finally transactions and derived caches. It materializes domain objects into a
deterministic `ObjectId`-keyed index and runs the eleven checks above in
sequence: schema compatibility, shard-index agreement, reference existence,
relationship validity, physical-to-electrical binding resolution,
library-revision pinning, rule-scope validity, output-context validity,
proposal/transaction reference resolution, cache staleness, and
artifact-metadata resolution.

Resolution is the moment the doctrine "files are persistence partitions; the
resolved `DesignModel` is the authority" becomes operational. Import is a
one-time converter: it reads a KiCad file once, materializes a native model,
records `ImportProvenance` on every object, and writes the native source
shards directly — after which the result is simply a native project and the
original imported file is retained only as immutable provenance, never again
consulted to resolve the project. There is no "imported board" state and no
authority distinct from the native shards; origin is provenance only (see
`docs/DATUM_PRODUCT_MECHANICS.md` "Interop Boundary And Import Posture"). If any
check fails fatally,
the resolver does not assemble a normal authoritative model; it opens a
recovery/diagnostic model that loads the inspectable shards read-only, reports
each failing check with the offending object and shard references, and offers
repair operations that flow through the normal transaction model. No shard is
permitted to silently win.

## Git Diff And Merge Expectations

Datum should optimize for reviewable source changes.

Expected properties:
- deterministic key ordering
- stable object ordering
- stable IDs
- minimal churn for common edits
- generated caches either deterministic and clearly marked or excluded from
  source review
- workspace/layout state separable from design source state
- object-level conflicts detectable where practical
- post-merge resolver/check run identifies cross-domain mismatches

Example expectations:
- moving `U12` changes the physical board shard and transaction metadata, not
  unrelated electrical sheets
- editing a wire changes the relevant sheet and resolved graph state, not
  unrelated board geometry
- modifying panel rails changes manufacturing plan state, not board outline
- changing Gerber settings changes output job state, not copper geometry
- regenerating artifacts does not rewrite source design shards

Merge support does not need to be perfect in the first implementation, but the
format must not make safe merges impossible by design.

## Proof Gates

The proof gates across this document and its companions (000E, 000F)
previously overlapped heavily — panelization isolation alone appeared as a
separate gate in three documents — and each document independently asked which
gate to run first, with pass criteria stated as "reviewable diff" or
"acceptable diffs" and no thresholds. For a project that already gates itself
with machine-checked spec parity, drift gates, import determinism, and a 0%
false-positive/false-negative check bar, that was the missing rigor. The gates
are therefore consolidated into one ordered, deduplicated, threshold-bearing
set. 000D is the canonical home of the full set; the companion documents
cross-reference it.

The canonical metric for a "small, reviewable diff" is shard isolation by
content hash: a qualifying edit must leave every unrelated shard byte-identical
(equal sha256) before and after, and the changed shard set must equal the
document's predicted set exactly.

The ten consolidated gates:

1. PG-IDENTITY-SUBSTRATE — surrogate `ObjectId` is persisted on create,
   survives rename/move/reload as identity no-ops, and survives source-file
   edit via the `import_key`-keyed Import Map (proving the
   `project_surface.rs` source_hash defect is closed).
2. PG-RESOLVER-RECOVERY — the resolver assembles one `ObjectId`-keyed
   authority from shards, and on a fatal check opens a read-only diagnostic
   model rather than a partial authority.
3. PG-COMMIT-ATOMIC+DURABLE-UNDO — commit stages+fsyncs, journal-appends as
   the commit point, then atomic-renames; an interrupted commit rolls
   forward/back deterministically; durable undo/redo replays journal cursors
   with undo as a compensating transaction.
4. PG-SHARD-DIFF-ISOLATION — the byte-identical isolation metric above:
   moving one component on the datum-test board changes only the board and
   transaction shards; a wire edit changes only the sheet, resolved-graph
   cache, relationship, and transaction shards while board geometry stays
   byte-identical.
5. PG-PROPOSAL-PARITY — a proposed `OperationBatch` previews exactly the
   shard/object set it commits.
6. PG-LIVE-CAM-EQUIVALENCE — first equivalence subset: Gerber copper +
   Excellon drill on datum-test; live (T1 cached) output equals cold full
   regen (T0), and export matches the projection.
7. PG-PANELIZATION-ISOLATION (deduped, was four copies) — adding panel
   rails, tabs, and fiducials leaves the board outline, copper, mask, paste,
   drill, and placement shards byte-identical while the panel state lives
   entirely in the manufacturing-plan shard; the panel artifact identifies
   board instance transforms and model revision.
8. PG-VARIANT-RESOLUTION (population-only) — switching the active variant
   composes the overlay with zero base writes; a variant-to-variant diff is a
   diff of two small overlay maps.
9. PG-ARTIFACT-TRACEABILITY — a changed Gerber setting increments the
   output-job revision; artifact metadata records model revision, output-job
   revision, generator version, and validation state; three-run regeneration
   is byte-identical. (The optional from-shards KiCad emitter is a separate,
   deferred export path; if and when it is built, a
   byte-identical-when-unmodified fidelity gate on datum-test applies to it
   as export validation — not as a migration or authority step.)
10. PG-HARNESS-WIRING — all gates run from a new
    `scripts/run_migration_proof_gates.sh`, invoked from
    `scripts/run_drift_gates.sh`.

Cross-shard validation (an injected electrical/physical mismatch) is covered
under PG-RESOLVER-RECOVERY: the resolver opens the project in diagnostic mode,
the mismatch is recorded in the relationships shard as an `UnresolvedMismatch`,
a check reports it, and no shard overwrites another. Library-revision pinning
(an updated footprint or model attachment leaves the project pinned to its
prior revision until an accepted proposal/transaction updates it) is covered
under PG-ARTIFACT-TRACEABILITY's traceability obligations.

These gates are wired into a runnable harness
(`scripts/run_migration_proof_gates.sh`, invoked from
`scripts/run_drift_gates.sh`) that constructs the datum-test fixture project,
performs each edit, and asserts the numeric thresholds, exiting non-zero on any
miss in the same convention as the existing drift gates. The sequencing
recommendation is to prove PG-SHARD-DIFF-ISOLATION (small physical diff) and
the board-first cross-shard gates first, because they exercise the resolver
and the relationships shard — the two keystone surfaces with no current
implementation — before any live-CAM or library-pinning work depends on them.
First proof seed (Slice 0) is the board-first, all-`BoardOnly` seed on
datum-test, with reverse-engineering context carried in provenance; the
`ImplementedBy` seed is the immediate follow-on now that `ComponentInstance` is
approved.

## Open Owner Questions

Several questions that were open in the prior draft are now answered by the
ratified mechanism and have moved into the body: transaction history IS
required source state from the first cut (the journal is the commit primitive,
not optional); resolved electrical graphs ARE persisted as revision/hash-keyed
caches that the resolver marks stale and recomputes; manual file edits that
break invariants cause the resolver to open in recovery/diagnostic mode rather
than accept split truth. The genuinely open forks remain:

- Should `ObjectId` integers be globally unique across all projects or only
  project-local? (The integer `ObjectId(u64)` is itself a deferred diff-size
  optimization; the wire type is `Uuid` for the first cut. This is the
  global-vs-project-local owner call that gates the integer variant.)
- Should Datum source shards use human-readable files first, or should some
  shards be database-backed from the beginning?
- Should workspace/layout state live inside the project tree, user config, or
  both?
- Should generated artifact metadata be committed by default?
- How strict should Datum be when opening a project with stale caches or
  missing artifact files — auto-recompute silently, warn, or block?
- What merge/conflict behavior is required before multi-writer workflows are
  considered safe? (Multi-writer is reserved by design — the parent-txn DAG
  field is present — but the first cut is single-writer-per-project and
  multi-writer is not built.)
- Should the relationships and journal partitions be per-owner / per-session
  segments exactly as proposed, or is a coarser partitioning acceptable for
  the single-writer first cut? (000D defers final layout.)

## Non-Goals

This document does not define:
- final file format syntax
- exact directory layout
- database engine selection
- complete schema for every shard
- complete merge algorithm
- artifact release policy
- implementation milestone order
- acceptance of the unified model as final architecture
