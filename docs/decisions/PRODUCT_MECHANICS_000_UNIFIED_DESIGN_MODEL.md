# Product Mechanics Decision 000: Unified Design Model

Status: draft hypothesis + how/mechanism woven 2026-06-18; identity,
ComponentInstance, journal, and M7 sequencing ratified; remaining forks open.
Date: 2026-06-17

Driven by:
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_RESEARCH_SYNTHESIS.md`
- `docs/DATUM_PRODUCT_MECHANICS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_REVIEW_AGENDA.md`
- `docs/decisions/PRODUCT_MECHANICS_000C_UNIFIED_MODEL_FEASIBILITY.md`
- project-owner product discussion on avoiding conventional EDA file/workspace
  segregation

## Decision Scope

Define the top-level product model Datum edits and presents.

This decision comes before the Canonical Edit Model because operations must
target the right authority. Datum should not accidentally encode the old EDA
assumption that schematic, PCB, library, and manufacturing are separate
authoritative worlds synchronized after the fact.

## Product Intent

Datum should avoid copying conventional EDA doctrine when the doctrine exists
mainly because older tools evolved as separate schematic capture and PCB layout
programs.

The product should support familiar schematic and PCB workflows, but its
internal model should be stronger:

> One canonical design model, many domain projections, optional saved
> viewports.

The schematic and PCB layout should inform each other through explicit
relationships inside one project model, not through fragile synchronization
between segregated files.

## Decision

Datum shall be designed around a single canonical `DesignModel`, and the
mechanism that produces it is the load-bearing part of this decision. There is
exactly one authority: an in-memory `DesignModel` assembled by a single
engine-owned `ProjectResolver` from segmented source shards on disk. Files are
persistence partitions; the resolved model is the authority. The `DesignModel`
is never serialized as a monolith and never read from disk directly — it is
always the product of resolving shards. This is how "one canonical model" and
"diffable on-disk files" coexist without either one becoming a second truth.

The `DesignModel` contains electrical intent, physical implementation, library
bindings, rules/constraints, standards basis, analysis records, manufacturing
state, artifacts, proposals, transactions, and provenance.

Everything else — the connectivity graph, zone fill, manufacturing
projections, variant views, and relationship status — is DERIVED state:
cacheable, revision/hash-keyed, and never a second authority. Schematic pages,
PCB layouts, symbols, footprints, BOMs, manufacturing drawings,
rules/check reports, and analysis views are domain projections over the
resolved model.

The application may present those projections in separate tabs, panels,
fullscreen contexts, split panes, or saved output sheets, but the projections
are not separate authorities. Shards are persistence partitions, never
authorities; the `ProjectResolver` is the sole assembler of the one
`DesignModel`.

## Core Primitives

### DesignModel

Canonical project truth — the single authority, assembled in memory by the
`ProjectResolver` from source shards. It is never the on-disk form.

Contains:
- electrical intent
- physical implementation
- library bindings
- rules and constraints
- standards/process basis
- derived connectivity state
- analysis records
- manufacturing/output state
- artifacts
- proposals
- transactions
- provenance

### DomainObject

Any authored or derived object in the design model.

Domain object families include:
- electrical objects: symbols, pins, wires, labels, buses, ports, no-connects,
  hierarchy, electrical constraints
- physical objects: component placements, footprints, pads, tracks, vias,
  zones, stackup, keepouts, board outline, mechanical/process geometry
- library objects: units, entities, parts, packages, symbols, footprints,
  padstacks, pin-pad maps, model attachments
- rule/check objects: constraints, rules, check runs, findings, waivers,
  deviations
- artifact objects: BOMs, output jobs, Gerbers, drill files, PnP files,
  assembly drawings, reports

### Object Identity

Object identity is a persisted SURROGATE, never derived from name, path, or
position. The wire type is `ObjectId = Uuid`, persist-on-create (RATIFIED).
Native objects keep the v4 UUID they already generate at creation, made stable
across reload by persisting it as a field rather than regenerating it; because
identity is a stored field, rename and move are identity no-ops (name and
`sheet_ref` are non-identity fields). Imported objects keep their v5 UUID, but
it is DEMOTED to an import SEED: persisted on first import into a net-new
Import Map shard keyed by `import_key`, NOT `source_hash`. Keying on
`import_key` fixes the real lost-on-edit defect, which lives in the running
rules / part / net-class overlays, not in the dead `ids_sidecar` module. An
allocated integer `ObjectId(u64)` remains a DEFERRED diff-size optimization
behind a global-vs-project-local owner call (see Open Owner Questions).

### ComponentInstance

A `ComponentInstance` is a per-instance stable surrogate (`Uuid`) that BOTH the
schematic `PlacedSymbol` and the board `PlacedPackage` reference. It is
introduced now (RATIFIED) and is distinct from the library `part` field. It is
the canonical electrical-to-physical join, created and linked at import and at
forward annotation, and it replaces the reference-designator string that
informally serves as that join today. The implicit assumption that a schematic
symbol and its board package share one identity is incorrect: they are two
domain objects bound through the `ComponentInstance` surrogate.

### Net Identity

Net identity is a stable `NetId` bound to derived connectivity groups through a
`NetAnchor`. The anchor is an authored hint that the resolver may refresh,
except for user-placed anchors. The `NetId` persists across rename, split, and
merge; `AnchorKind` is a refreshable derivation. When nets merge, the
survivor rule is lowest `NetId` wins.

### Commit Primitive And Journal

Every mutation from every surface (GUI, CLI, MCP, AI, importer, checker)
produces an `OperationBatch` and flows through ONE `commit()` primitive:
apply-in-memory → stage shard bytes + fsync → append a `TransactionRecord` to
the journal + fsync (THE COMMIT POINT) → atomic rename + dir fsync. The journal
is required source state from the first cut (RATIFIED): it is the persisted,
replayable, auditable history. Durable undo/redo are cursors into the journal;
undo is recorded as a compensating transaction so history stays append-only.
Single-writer-per-project is the explicit first-cut precondition; multi-writer
is reserved by design (a parent-txn DAG field is present) but NOT built.

Transactions are the SOLE producer of revisions: `object_revision` (monotonic
`u64` per object) and `model_revision` (sha256 over the canonical sorted
`object_revisions` plus the accepted-transaction tip). These revisions give the
derived projection, variant, and staleness layers their cache keys for free.
The Relationships shard and the journal are partitioned (per-owner /
per-session segment) to avoid hot-files.

### Projection

A projection is a task-oriented lens over the design model.

Examples:
- electrical/schematic projection
- physical PCB layout projection
- symbol projection
- footprint projection
- library/part projection
- rules/checks projection
- manufacturing projection
- analysis projection
- BOM projection

Projections may expose different tools and visual language, but they do not
own separate source truth.

### EditorContext

The active editing context for a projection.

Defines:
- available tools
- snapping behavior
- selection rules
- command behavior
- overlays
- inspector sections
- check feedback

Editor contexts are allowed to be domain-specific. The electrical projection
and physical projection have different editing requirements. The unifying rule
is that their edits mutate the same `DesignModel` through the same canonical
edit model.

### RelatedContext

Context surfaced from related projections without requiring the user to manage
a tiled split-screen workflow.

Examples:
- selecting `R12` in physical layout shows its electrical symbol, nets, part,
  footprint, rules, check findings, and deviations in the inspector
- selecting a schematic net shows its routed copper, airwires, length, rule
  class, and DRC/ERC state
- selecting a pad shows its pin, symbol, part, padstack, mask/paste policy,
  and IPC/process findings

### Viewport

A saved visual framing of a projection.

Viewports are for:
- review
- documentation
- output pages
- assembly drawings
- fabrication drawings
- comparison views
- presentation sheets

Viewports should not be the primary authority. They are saved views or output
frames over model/projection state.

### NavigationStack

User movement through projections and related objects.

Datum should let users jump from physical layout to electrical intent, from
symbol to footprint, from DRC finding to affected object, and back without
forcing a SolidWorks-style tiled part/assembly management workflow.

## Relationship States

Because electrical intent and physical implementation can intentionally
diverge, Datum must represent relationship state directly. A `Relationship` is
an authored binding with its own stable surrogate id, classified by a
`RelationshipKind`: `ImplementedBy`, `BoardOnly`, `SchematicOnly`,
`ReverseEngineered`, `Pending`, `Mismatch`.

The decisive mechanism is that relationship state has three parts that must not
be conflated. The relationship binding — which electrical net implements as
which physical net, which symbol instance binds to which placement — is authored
source state with its own stable `ObjectId`, and it is what the Relationships
shard persists. The relationship kind — including `BoardOnly`,
`SchematicOnly`, and `ReverseEngineered` — is the authored classification of
that binding. The relationship status — `Implemented`,
`PendingImplementation`, `UnresolvedMismatch` — is derived by the resolver from
the authored binding and the resolved electrical graph, and is recomputed on
load. Human decisions that are not relationship kinds, such as
`LayoutDeviation` and accepted deviations, are explicit authored-intent records
and the resolver may never overwrite them with a derived status.

The state machine therefore has two classes of transition: derived transitions
driven by resolver recomputation (pending becomes implemented when the physical
net binds all electrical pins; implemented becomes unresolved-mismatch when
recomputation finds divergence and no authored deviation explains it), and
authored transitions driven by explicit operations such as
`AcceptLayoutDeviation`. A derived transition that would contradict an authored
intent state is reported as a diagnostic rather than applied silently.

This split is the canonical vocabulary:

- DERIVED status (resolver-computed, recomputed on load, never persisted as
  authority): `Implemented`, `PendingImplementation`, `UnresolvedMismatch`.
- AUTHORED intent (human decision, persisted, resolver may never overwrite):
  `LayoutDeviation`, `BoardOnlyObject`, `SchematicOnlyObject`,
  accepted-deviation.

`ReverseEngineered` is only a `RelationshipKind`. Board-first or import
workflows that need to explain inference, confidence, lossy conversion, or
repair history attach that explanation as import/provenance metadata or as an
accepted-deviation record, not as a parallel `ReverseEngineered` intent axis.

These states are better than treating every mismatch as accidental drift or
hiding it inside a bulk ECO workflow. Because the binding carries a
`ComponentInstance` (electrical) to placement (physical) join, relationship
status is computed per instance rather than inferred from a
reference-designator string.

## UI Consequence

Datum should not depend on permanent split-screen schematic/PCB panes to feel
coherent.

Preferred interaction model:
- one active editing context at a time
- fast projection switching
- inspector-driven related context
- cross-domain overlays when useful
- optional split panes only when the user asks for comparison
- saved viewports for output/review/documentation
- navigation history across projections and related objects

Split screens remain useful, but they are not the product model.

## Canonical Edit Model Consequence

Operations target the `DesignModel`, not isolated schematic or PCB files, and
they reach disk only through the single `commit()` primitive described under
Core Primitives — apply-in-memory, stage shard bytes, append the journal record
(the commit point), then atomic rename. Every example below is an
`OperationBatch` that commits this way; none writes a file path directly.

Examples:
- `DrawWire` mutates electrical domain objects in the design model.
- `RouteTrack` mutates physical domain objects in the design model.
- `AssignPart` mutates library binding relationships in the design model.
- `SetRule` mutates rule/check domain objects in the design model.
- `SetPadProcessAperture` mutates physical/process geometry in the design
  model.
- `AcceptLayoutDeviation` records an explicit relationship state between
  electrical intent and physical implementation.

## Standards And Compliance Impact

Standards data belongs in the design model, not in a separate report silo.

Examples:
- project standards basis
- IPC/process aperture policy
- rule/check findings
- waivers
- deviations
- manufacturing sign-off artifacts

DRC/ERC/check findings should attach to domain objects and relationship states
in the design model so they are visible from every relevant projection.

## Explicit Non-Goals

This decision does not require:
- one giant visual canvas containing every possible view at once
- eliminating familiar schematic or PCB editing projections
- preventing users from opening split views
- deciding the final file serialization format
- implementing all projections immediately
- collapsing electrical symbols, footprints, and placed components into one
  object type

The point is shared authority, not visual chaos.

## First Proof Slice

Slice 0 is board-first: an imported/reverse-engineering proof seed in which
every relationship is `BoardOnly`, resolved into one `DesignModel` from shards.
Any reverse-engineering context is carried by provenance, while the relationship
kind itself remains `BoardOnly`. The `ImplementedBy` seed is the immediate
follow-on now that `ComponentInstance` is approved, since that join is what an
`ImplementedBy` binding needs.

The first proof slice should show:
- one `DesignModel` assembled by the `ProjectResolver` from source shards,
  containing at least electrical and physical domain objects
- one electrical projection and one physical projection reading that same model
- selecting an object in one projection exposes related context from the other
- one operation flows through `commit()` and mutates electrical objects
- one operation flows through `commit()` and mutates physical objects
- one relationship state or unresolved mismatch is represented explicitly,
  with its authored-binding vs derived-status split visible

This work is additive: the identity, resolver, and commit modules are built
CONCURRENTLY with M7 and do NOT close it. Native shards are the only authority;
import is a one-time converter that writes native shards directly, after which
the board is simply a native board (origin survives only as provenance). The
engine that currently stores imported boards as KiCad-text plus sidecars is a
TRANSITIONAL DEFECT, not a design choice; the fix is "import writes native
shards directly," not a phased authority flip with a dual-write window. The
optional from-shards KiCad EXPORTER (native -> KiCad) is separate and DEFERRED;
it does not gate this slice or native maturity. See
`docs/DATUM_PRODUCT_MECHANICS.md` "Interop Boundary And Import Posture".

## Proof Gates

The consolidated proof plan is the ten-gate set; its full definitions live in
`docs/decisions/PRODUCT_MECHANICS_000D_*` and are cross-referenced here. The
gates most directly load-bearing for this decision are:

1. PG-IDENTITY-SUBSTRATE — `ObjectId = Uuid` persist-on-create survives reload,
   rename, and move as identity no-ops; imported seeds land in the Import Map
   shard keyed by `import_key`.
2. PG-RESOLVER-RECOVERY — the `ProjectResolver` reassembles the one
   `DesignModel` from shards, and derived relationship status recomputes
   without overwriting authored intent.
3. PG-COMMIT-ATOMIC+DURABLE-UNDO — `commit()` is atomic and crash-safe; undo is
   a compensating transaction recorded as a journal cursor.
4. PG-SHARD-DIFF-ISOLATION — an edit touches only its owning shard; partitioned
   Relationships and journal avoid hot-files.

The remaining gates (PG-PROPOSAL-PARITY, PG-LIVE-CAM-EQUIVALENCE,
PG-PANELIZATION-ISOLATION, PG-VARIANT-RESOLUTION,
PG-ARTIFACT-TRACEABILITY, PG-HARNESS-WIRING (retired 2026-07-02)) belong to the projection and
variant decisions and are defined in 000D. The earlier
PG-AUTHORITY-FLIP framing is SUPERSEDED: there is no imported-to-native
authority flip to gate — import writes native shards directly and native is the
only authority (see `docs/DATUM_PRODUCT_MECHANICS.md` "Interop Boundary And
Import Posture"). All gates wire into `scripts/run_drift_gates.sh` via
`run_migration_proof_gates.sh` (PG-HARNESS-WIRING retired 2026-07-02 —
self-referential wiring check superseded by CI invoking the runner directly).

## Resolved By Mechanism

Several earlier open questions were answered by the woven mechanism and are
recorded here as resolved:

- Persist schematic/PCB as separate files? Yes — as shards. Files are
  persistence partitions; the `ProjectResolver` assembles the one `DesignModel`
  from them, so separate on-disk files and one authority coexist by design.
- Minimum relationship-state set for the first slice? The derived status triad
  (`Implemented`, `PendingImplementation`, `UnresolvedMismatch`) plus the
  `BoardOnly` authored intent — that is exactly what board-first Slice 0
  exercises.
- Start from schematic-to-physical or reverse engineering? Reverse engineering
  first: Slice 0 is board-first all-`BoardOnly` with reverse-engineering
  context in provenance, and the `ImplementedBy` seed is the immediate
  follow-on.
- How much existing native layout can be preserved? The shipped native format,
  loader, and validate command are kept as the seed for the engine
  `ProjectResolver`, evolved in place with no rewrite cliff. This is a native
  resolver build-out, not an import-authority migration: native is the only
  authority throughout (see `docs/DATUM_PRODUCT_MECHANICS.md` "Interop Boundary
  And Import Posture").

## Open Owner Questions

1. Should the allocated integer `ObjectId(u64)` diff-size optimization be
   adopted, and if so under a global or project-local owner? It is deferred,
   not decided; the first cut stays on `Uuid`.
2. Should viewports become first-class persisted objects early, or wait until
   manufacturing/documentation work?
3. SUPERSEDED — there is no dual-write deprecation window and no
   imported-KiCad authority flip. Import is a one-time converter that writes
   native shards directly; native is the only authority and the KiCad-text-plus-
   sidecar storage is a transitional defect to remove, not a window to manage.
   See `docs/DATUM_PRODUCT_MECHANICS.md` "Interop Boundary And Import Posture".
4. Should `NetAnchor` refresh policy treat any anchor classes beyond
   user-placed as pinned, or is user-placed the only non-refreshable kind?
