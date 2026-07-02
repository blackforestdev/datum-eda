# Product Mechanics Decision 000C: Unified Model Feasibility

Status: draft hypothesis + how/mechanism woven 2026-06-18; identity,
ComponentInstance, journal, and M7 sequencing ratified; remaining forks open.
Date: 2026-06-17 (mechanism woven 2026-06-18)

Driven by:
- `docs/decisions/PRODUCT_MECHANICS_000_UNIFIED_DESIGN_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_000B_VIEW_COMPOSITION_AND_LIVE_PRODUCTION.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_RESEARCH_SYNTHESIS.md`
- project-owner concern that innovation must be validated against logistics,
  existing EDA practice, and implementation cost

## Purpose

Validate whether Datum should adopt the unified `DesignModel` and live
projection architecture as a product foundation.

The unified model is promising, but it must not become doctrine just because it
is novel. This feasibility review compares candidate architectures and defines
what must be proven before implementation work depends on the model.

## Candidate Architectures

### A. Conventional Segmented Authority

Schematic, PCB, library, rules, manufacturing, and outputs are separate
documents or subsystems with synchronization between them.

Characteristics:
- familiar to EDA users
- simpler mental model for file ownership
- easier to map onto existing code and formats
- natural fit for schematic-file and board-file import/export
- relies on sync/ECO/back-annotation discipline

Primary risk:
- Datum repeats the old EDA split-world workflow and loses the product
  differentiation that motivated the project.

### B. Unified Authority, Monolithic Storage

One canonical `DesignModel` stored as one large project file or database.

Characteristics:
- strongest single-authority semantics
- simplest conceptual source of truth
- easy to make cross-domain relationships explicit

Primary risk:
- poor git diffs, hard merges, large-file churn, and high blast radius for
  small changes.

### C. Unified Authority, Segmented Storage

One canonical `DesignModel`, persisted as storage shards.

Possible shards:
- electrical sheets and resolved electrical graph
- physical boards
- library bindings
- rules and constraints
- relationship states
- manufacturing plans
- output jobs
- proposals
- transactions
- artifacts metadata

Characteristics:
- preserves unified authority while allowing practical file organization
- supports git diffs better than monolithic storage
- lets electrical, physical, library, and manufacturing domains evolve without
  becoming separate authorities

Primary risk:
- requires a strong project loader/resolver so shards never become competing
  truths.

## Preliminary Recommendation

Treat C, unified authority with segmented storage, as the leading candidate,
and treat the resolved spine below as the hypothesis under proof.

The reconciled spine is: there is exactly one authority, an in-memory
`DesignModel` assembled by a single engine-owned `ProjectResolver` from
segmented source shards on disk. Shards are persistence partitions, never
authorities; the resolved model is the authority. Everything else —
connectivity graph, zone fill, manufacturing projections, variant views,
relationship status — is DERIVED state: cacheable, revision/hash-keyed, never a
second authority. Four mechanism choices inside this spine are no longer
hypothetical; they are owner-ratified (2026-06-18) and stated as DECIDED below:
object identity is a persisted surrogate (`ObjectId = Uuid`, persist-on-create);
a `ComponentInstance` surrogate is the canonical electrical-to-physical join;
every mutation flows through one journaled `commit()` primitive; and the
identity/resolver/commit modules are built CONCURRENTLY with M7 without closing
it. The remaining structural forks stay open for owner review.

Do not fully accept the rest of the model until the proof gates in this document
(and the consolidated 10-gate plan in 000D) are met.

Do not choose B unless Datum intentionally moves toward a database-backed
project format and accepts the version-control cost.

Do not default to A unless the feasibility review shows the unified model adds
more complexity than it removes.

## Feasibility Areas

### Multiple Schematic Sheets

Question:
- Can many schematic sheets be represented as projections/documents inside one
  electrical domain without turning each sheet into a separate authority?

Required answer:
- `SchematicSheet` is a visual/authoring document.
- `ElectricalDesign` or equivalent resolved electrical graph is the logical
  intent across sheets, hierarchy, labels, ports, power symbols, buses, and
  variants.
- Sheet files may be storage shards, but the design model owns the resolved
  electrical truth.

Proof needed:
- two sheets contribute to one resolved electrical graph
- selecting a physical object can identify its electrical source across sheets
- editing a sheet invalidates/recomputes affected relationship/check state

### Multiple Boards

Question:
- Can one project contain multiple physical implementations without collapsing
  everything into one board?

Required answer (mechanism):
- `PhysicalDesign` or `Board` instances must be first-class.
- A project may contain one board, multiple boards, daughtercards, modules,
  alternate physical implementations, or reverse-engineered board-only
  designs.
- Each board binds to an electrical design scope through explicit `Relationship`
  state — an authored binding with a surrogate id and a `RelationshipKind`
  {ImplementedBy, BoardOnly, SchematicOnly, ReverseEngineered, Pending,
  Mismatch}. Relationship STATUS splits into DERIVED {Implemented,
  PendingImplementation, UnresolvedMismatch} versus AUTHORED intent
  {LayoutDeviation, BoardOnlyObject, SchematicOnlyObject, ReverseEngineered,
  accepted-deviation}.
- The per-instance electrical-to-physical join is a `ComponentInstance`
  (RATIFIED, introduced NOW): a per-instance stable surrogate (Uuid) that BOTH
  `PlacedSymbol` and `PlacedPackage` reference, distinct from the library `part`
  field, created/linked at import and forward-annotation. It replaces the
  reference-designator string that informally serves as that join today.

Proof needed:
- one electrical domain can bind to more than one board
- one board can implement a subset of electrical intent
- board-only objects and schematic-only objects can be represented explicitly
  as `BoardOnly`/`SchematicOnly` relationships
- first proof seed (Slice 0) is board-first: an all-`BoardOnly`
  reverse-engineered seed, with the `ImplementedBy` seed as the immediate
  follow-on now that `ComponentInstance` is approved

### Panelization

Question:
- Can panelization live in manufacturing without mutating source board
  geometry?

Required answer:
- `PanelProjection` references board instances with transforms.
- Panel rails, tabs, mouse bites, V-scores, fiducials, tooling holes, coupons,
  and panel labels are manufacturing-domain objects.
- Board Gerbers and panel Gerbers are different output contexts.

Proof needed:
- panel output references a board without duplicating editable board truth
- changing panel tabs does not modify the board outline/copper
- output artifacts identify whether they came from board or panel context

### Versioning And Git Diffs

Question:
- Can unified authority avoid becoming one unmergeable blob?

Required answer (mechanism):
- Storage shards must be deterministic and stable, and object identity is a
  persisted SURROGATE, never derived from name, path, or position. The wire type
  is `ObjectId = Uuid`, persist-on-create (RATIFIED): native objects keep the v4
  uuid generated once at creation and PERSIST it as a field, so it is stable
  across reload and rename/move become identity no-ops (name and `sheet_ref` are
  non-identity fields). Imported objects keep v5 but DEMOTED to an import SEED,
  persisted on first import into a net-new Import Map shard keyed by
  `import_key` (NOT `source_hash`). Allocated integer `ObjectId(u64)` is a
  DEFERRED diff-size optimization behind a global-vs-project-local owner call.
- Because identity is persisted and rename/move are identity no-ops, unrelated
  edits touch unrelated shards by construction.
- Generated artifacts carry `model_revision` (sha256 over canonical sorted
  object_revisions + accepted-txn tip) so they can never be mistaken for source
  authority.

Proof needed:
- moving a physical component produces a small deterministic diff (rename/move =
  identity no-op)
- changing an electrical net/sheet produces a small deterministic diff
- changing panelization does not rewrite board source files
- generated artifact metadata points to exact `model_revision`

### Merge And Collaboration

Question:
- Can two users or agents safely edit different domains or projections?

Required answer (mechanism):
- SINGLE-WRITER-PER-PROJECT is the explicit first-cut precondition (RATIFIED).
  Multi-writer is reserved by design — a parent-txn DAG field is present in the
  `TransactionRecord` — but is NOT built in the first cut.
- Object-level identity (persisted `ObjectId = Uuid`) and deterministic shard
  serialization make object/operation-level reconciliation tractable when
  multi-writer is later built.
- The journal is the conflict substrate: because every mutation appends a
  replayable `TransactionRecord`, post-hoc reconciliation has an auditable
  history to merge against rather than opaque file states.
- Relationship/check invalidation (via `model_revision`) catches cross-domain
  mismatches at resolve time.

Proof needed (deferred with multi-writer; reserved, not first-cut):
- concurrent electrical and physical edits can be detected and reconciled
- conflicting edits to the same object are detected
- post-merge check run identifies unresolved relationship mismatches

### Performance And Incremental Projection Regeneration

Question:
- Can projections remain interactive on real projects?

Required answer (mechanism):
- The interactivity question resolves not by promising sub-region recomputation
  everywhere, but by choosing the right granularity for each projection. One
  generation core serves three tiers: T0 cold full-regen (the exporter AND the
  equivalence oracle), T1 a revision/hash-keyed memoized live cache (the live
  tier), and T2 sub-region incremental (deferred behind the T0 oracle).
- Projections that are global by nature — copper pours, solder-mask polarity,
  aperture and drill-tool tables — are regenerated whole within the live tier.
  Attempting to recompute them region-by-region would risk presenting partially-
  updated production geometry, which the determinism gate forbids.
- Projections that are embarrassingly per-object — board outline, silkscreen,
  paste — are the only honest candidates for future sub-region incrementality
  (T2), and even then only behind a full-regeneration equivalence oracle.
- Staleness keys come for free from the commit substrate: `object_revision`
  (monotonic u64 per object) and `model_revision` (sha256 over canonical sorted
  object_revisions + accepted-txn tip) drive cache invalidation; no projection
  is its own authority.

Proof needed:
- A physical edit invalidates and regenerates exactly the affected PROJECTIONS
  (the copper and mask of the touched layers; the drill table only when a via
  changed) regenerated WHOLE — not the whole output set, and never on a
  transient interaction but on the COMMITTED transaction. The practical unit for
  the first proof is "the affected projection regenerated whole, on commit not
  mouse-move." Affected-region recomputation is admitted later only behind the
  T0 full-regeneration equivalence oracle.
- An electrical edit invalidates affected nets, relationships, and checks via
  the same revision keys.
- Opening live CAM beside layout serves cached projections and pays only for
  what the last commit actually changed; it does not require full export on
  every interaction for large boards.

### Undo/Redo Transaction Scope

Question:
- Can cross-domain transactions be understandable and reversible?

Required answer (mechanism):
- A TRANSACTION JOURNAL is required source state from the first cut (RATIFIED).
  Every mutation from every surface (GUI/CLI/MCP/AI/importer/checker) produces
  an `OperationBatch` and flows through ONE `commit()` primitive: apply in
  memory → stage shard bytes + fsync → append a `TransactionRecord` to the
  journal + fsync (THE COMMIT POINT) → atomic rename + dir fsync. The journal is
  the persisted, replayable, auditable history.
- Durable undo/redo are CURSORS into the journal; undo is recorded as a
  COMPENSATING transaction (append-only history, nothing is erased), which is
  what preserves relationship/check invalidation behavior across undo.
- Transactions are the SOLE producer of revisions (`object_revision`,
  `model_revision`), so undo/redo move the staleness keys and projections
  re-derive correctly.
- Simple manual edits remain direct; cross-domain changes and repairs are
  proposals by default.

Proof needed (PG-COMMIT-ATOMIC+DURABLE-UNDO, gate 3 of the 000D plan):
- one electrical edit undo/redo
- one physical edit undo/redo
- one accepted proposal undo/redo
- one cross-domain relationship-state change undo/redo

### Manufacturing Output Determinism

Question:
- Are live manufacturing projections trustworthy enough to show beside layout?

Required answer (mechanism):
- Production projections use the SAME generation core as artifact export: T0
  (cold full-regen) is both the exporter AND the equivalence oracle; T1 is the
  live memoized cache. Visible CAM geometry is never a rough approximation
  presented as truth.
- Zone fill is de-overloaded so native zones cannot accidentally export as solid
  copper. `Zone.polygon` is the authored boundary; a derived
  `ZoneFill{state: Filled|Unfilled|Stale|Unsupported, islands}` carries fill
  geometry, fed by a NEW `Zone` provenance field. Native copper-pour fill is OUT
  of scope for the first live-CAM proof (tactical default): native unfilled
  zones render a badged `Unfilled` placeholder and emit NO copper. Unfilled-zone
  export policy is to emit nothing plus a hard finding — NOT the
  solid-copper-boundary that is today's accidental behaviour. If pour fill is
  later authorized, the polygon-boolean substrate is `i_overlay` (pure-Rust,
  MIT) — no FFI creep.
- Artifacts record source `model_revision` and output job settings.

Proof needed (PG-LIVE-CAM-EQUIVALENCE, gate 6 of the 000D plan):
- the first equivalence subset is Gerber copper + Excellon drill on datum-test
- live Gerber projection matches exported Gerber semantics for that subset
- live NC drill projection matches exported drill semantics for that subset
- output job changes visibly affect the live projection and artifact metadata

### Import And Export Boundaries

Question:
- Can imported KiCad/IPC/Gerber/etc. data enter the unified model without
  losing source truth?

Required answer:
- imported data keeps provenance
- imported board-only flows can remain board-only or reverse-engineered until
  electrical intent exists
- import repair must be proposal-based when changing source-derived geometry

Proof needed:
- imported board can exist as physical implementation with incomplete
  electrical intent
- import provenance remains attached
- standards/process repair proposal can change native geometry without hiding
  original source facts

### Library Reuse And Versioning

Question:
- Can library objects be shared without making every project unstable when the
  library changes?

Required answer:
- library bindings need explicit version/revision references
- project-local overrides must be possible
- part/package/symbol/footprint changes must invalidate affected projections
  and checks

Proof needed:
- project uses a pinned library part/package revision
- library update creates a proposal or explicit update transaction
- footprint update identifies affected boards/panels/manufacturing outputs

### UI Complexity

Question:
- Does composable tab/PiP/tiled layout solve workflow pain without creating
  SolidWorks-style window juggling?

Required answer:
- one active editing context remains primary
- related context should be inspector/overlay/navigation first, split views
  optional
- saved workbench profiles must reduce setup overhead

Proof needed:
- user can route in physical projection with electrical related context visible
  without opening a required split-screen schematic editor
- user can open live Gerber/NC drill beside layout when desired
- closing a tab does not imply closing or unloading source authority

### AI And Agent Tooling

Question:
- Does unified authority materially help AI workflows?

Required answer:
- agents should query the design model, not scrape views
- projections should expose structured context and findings
- proposals must target model operations, not view-local hacks

Proof needed:
- agent can inspect selected physical object and retrieve electrical, library,
  rules, manufacturing, and check context
- agent can propose a bounded edit and explain affected projections
- agent cannot bypass transaction/proposal model

### Existing Codebase Migration

Question:
- Can current Datum move toward this without a rewrite cliff?

Required answer (mechanism):
- The migration risk is not that Datum lacks a native format. A native project
  format already exists, implemented in the CLI layer: `project new` writes the
  segmented layout this document anticipates (`project.json`, `schematic/`,
  `board/`, `rules/`), `project validate` already runs schema-version,
  required-file, and dangling-cross-reference checks, and manufacturing export
  already generates Gerber, drill, and BOM output from native projects.
- The true migration problem is that the native format is a CLI-only parallel
  universe: its shard types are duplicated serde structs that do not reuse the
  engine's `Board` and `Schematic` types (the board shard even stores packages
  as untyped JSON), and the engine cannot load them. Native is the only intended
  authority. The current engine storing imported boards as KiCad source text
  plus sidecars is a TRANSITIONAL DEFECT, not an accepted second authority and
  not a design choice; see `docs/DATUM_PRODUCT_MECHANICS.md` "Interop Boundary
  And Import Posture". Import is a ONE-TIME converter: a KiCad file becomes
  native data and is thereafter just a native board, with origin retained only
  as provenance.
- Migration proceeds without a rewrite cliff by lifting the already-shipped
  native shard layout, loader, and validator into the engine rather than
  rebuilding. They become the seed for an engine-owned `ProjectResolver` that
  resolves native shards into one `DesignModel`; the CLI `validate` command
  becomes a thin caller into it. The defect fix is that import writes native
  shards directly — not a phased authority flip from a retained KiCad-text
  authority.
- The first slice resolves the smallest viable shard set — manifest, board, and
  a new relationships shard — seeded from the canonical datum-test fixture (32
  routed segments, 11 footprints, and a schematic), where every recovered
  footprint is recorded as a `BoardOnly` `Relationship` and the original KiCad
  text is retained as a provenance shard (provenance only, never authority).
  Each subsequent slice resolves one further native domain — persisted
  transactions, electrical sheets, manufacturing plans, artifact metadata — so
  no single step replaces all persistence at once. An optional future
  from-shards KiCad EXPORTER (native → KiCad) is separate and DEFERRED; if built
  it would carry its own byte-identical-when-unmodified fidelity gate on
  datum-test, and it does not gate native maturity in any way.

Domain-to-module map (the concrete answer this section asks for):
- electrical → native schematic shard + engine connectivity solver
- physical → engine `Board` type + native board shard + current KiCad write-back
- rules and checks → engine rule/ERC/DRC modules + native rules shard
- manufacturing → existing Gerber, drill, and manufacturing CLI surfaces
- projection rendering → the read-only GUI consumers of engine types

Net-new keystone gaps (no code today; the migration must create them):
- the relationships shard (with `ComponentInstance` as its per-instance
  electrical-to-physical surrogate) that carries native identity (persisted v4
  `ObjectId = Uuid`) alongside the import SEED (the demoted v5 value persisted
  in the Import Map shard as provenance) without conflating the two
- the persisted transactions shard (the journal) that turns today's in-memory
  undo into durable, replayable provenance
- (optional, deferred) a from-shards KiCad EXPORTER (native → KiCad) for users
  who want to round-trip back to imported formats; this is separate from import,
  optional, and does not gate native maturity

Proof needed:
- the domain-to-module map above is exercised end-to-end on a resolved
  datum-test model
- the first migration slice (manifest + board + relationships, seeded
  board-first as all-`BoardOnly` reverse-engineered) loads through the
  `ProjectResolver` as native shards, with the original KiCad text retained as a
  provenance shard only

## Decision Gates

The feasibility gates below are the product-level questions; their
threshold-bearing, machine-checked form is the consolidated 10-gate proof plan,
whose full set lives in 000D (this doc cross-references it). The 10 gates are:
1 PG-IDENTITY-SUBSTRATE; 2 PG-RESOLVER-RECOVERY; 3 PG-COMMIT-ATOMIC+
DURABLE-UNDO; 4 PG-SHARD-DIFF-ISOLATION; 5 PG-PROPOSAL-PARITY; 6 PG-LIVE-CAM-
EQUIVALENCE; 7 PG-PANELIZATION-ISOLATION (deduped, was 4x); 8 PG-VARIANT-
RESOLUTION (population-only); 9 PG-ARTIFACT-TRACEABILITY; 10
PG-HARNESS-WIRING (into `run_drift_gates.sh`; retired 2026-07-02 —
self-referential wiring check superseded by CI invoking the runner directly).

Before committing to the unified model as governing architecture, Datum should
prove:

1. Multiple sheets can resolve into one electrical design graph
   (PG-RESOLVER-RECOVERY).
2. At least one board can bind to that electrical graph through explicit
   `Relationship` state.
3. A physical edit and electrical edit can commit through the same journaled
   `commit()` primitive (PG-COMMIT-ATOMIC+DURABLE-UNDO).
4. A live manufacturing projection can render from the same model source as
   export for the Gerber-copper + Excellon-drill subset on datum-test
   (PG-LIVE-CAM-EQUIVALENCE).
5. Panelization can reference a board instance without mutating board source
   geometry (PG-PANELIZATION-ISOLATION).
6. Storage shards produce acceptable diffs for common edits
   (PG-SHARD-DIFF-ISOLATION; rename/move = identity no-op).
7. Import provenance survives conversion into the design model
   (PG-IDENTITY-SUBSTRATE / Import Map shard).
8. The UI can show related context without forcing permanent split-screen
   workflow.

## Failure Modes

Adopt the conventional segmented model if:
- unified authority requires too much project loader/resolver complexity
- storage diffs become unmanageable
- projection invalidation cannot be made reliable
- cross-domain transactions are too confusing for users
- live production projections cannot be made trustworthy
- migration from current code requires a rewrite cliff — note the lift path
  above is designed to avoid exactly this (the shipped native shard layout,
  loader, and validator are lifted into an engine-owned `ProjectResolver`, and
  import is fixed to write native shards directly per the posture), so this
  failure mode fires only if that domain-by-domain lift proves impossible in
  practice

Adopt unified authority only for selected domains if:
- electrical/physical can share authority but library/manufacturing need to
  remain separate for versioning
- live production projections are useful but too expensive to keep fully live
  at first
- panelization needs separate project/package semantics

## Recommended Near-Term Posture

Treat unified authority with segmented storage as a hypothesis, with one
exception: the four ratified mechanisms (persisted `ObjectId = Uuid`,
`ComponentInstance`, the journaled `commit()` primitive, and concurrent-with-M7
sequencing) are DECIDED and may be built now. The identity, resolver, and commit
modules are additive engine modules built CONCURRENTLY with M7; this work does
NOT close M7. Native is the only authority; the defect fix is that import writes
native shards directly. The optional from-shards KiCad EXPORTER (native → KiCad)
is separate and DEFERRED, would carry its own byte-identical-when-unmodified
fidelity gate on datum-test if built, and does not gate native maturity. See
`docs/DATUM_PRODUCT_MECHANICS.md` "Interop Boundary And Import Posture".

Use the docs as a product direction, but do not mark the remaining structural
forks as finalized until the decision gates are tested against small
implementation slices and owner review.

Near-term wording should be:

> Datum is exploring a unified `DesignModel` with domain projections and
> segmented storage. The model must prove that it improves workflow coherence
> without making persistence, versioning, checks, manufacturing output, or
> implementation complexity worse than conventional EDA separation.

## Open Owner Questions

Several questions from the prior draft are now ANSWERED by ratified decisions and
have been folded into the mechanism above, not left open:
- Which proof to run first is decided: Slice 0 is the board-first,
  all-`BoardOnly` reverse-engineered seed on datum-test, with the
  `ImplementedBy` seed as the immediate follow-on; the first equivalence subset
  is Gerber copper + Excellon drill.
- Whether panelization is an early proof is decided: it is NOT a first proof;
  PG-PANELIZATION-ISOLATION (deduped) is one gate of ten, not a keystone.

Genuinely-open forks remaining:

1. Is segmented storage acceptable if the user experience feels unified?
2. What level of git-diff quality is required before accepting the model — and
   does that threshold justify the DEFERRED allocated-integer `ObjectId(u64)`
   diff-size optimization, which itself rides on a global-vs-project-local owner
   call?
3. Where is the line between useful innovation and unnecessary architecture
   risk for Datum's first serious editor milestone?
