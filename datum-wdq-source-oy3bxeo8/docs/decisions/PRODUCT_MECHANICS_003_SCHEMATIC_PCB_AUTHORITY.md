# Product Mechanics Decision 003: Schematic And PCB Authority

Status: draft for owner review.
Date: 2026-06-19

Driven by:
- `docs/DATUM_PRODUCT_MECHANICS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_REVIEW_AGENDA.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_RESEARCH_SYNTHESIS.md`
- `docs/decisions/PRODUCT_MECHANICS_000_UNIFIED_DESIGN_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_000D_STORAGE_AND_VERSIONING_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_000E_MULTI_SHEET_MULTI_BOARD_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_001_CANONICAL_EDIT_MODEL.md`
- project-owner clarification that normal design flow is schematic to PCB, but
  Datum must support different engineering workflows through primitives rather
  than one rigid sequence

## Decision Scope

Define how schematic/electrical intent and PCB/physical implementation relate
inside Datum.

This decision does not finalize the storage architecture. It defines authority
and relationship rules that must work whether Datum ultimately accepts the
unified `DesignModel` hypothesis or narrows it during feasibility review.

## Product Intent

For most new designs, the schematic is the authored electrical source of truth
and the PCB implements that intent.

Datum must preserve that common flow without hard-coding it as the only valid
workflow. Analog, RF, power, digital, reverse-engineering, fixture, and
board-only work can move through electrical and physical domains differently.

Required rule:

> The schematic/electrical domain owns electrical intent. The PCB/physical
> domain owns physical implementation. Neither domain may silently redefine the
> other.

The product goal is not to weaken schematic authority. The goal is to make
authority explicit, inspectable, and flexible enough for real engineering
workflows.

## Decision

Datum shall model schematic and PCB data as related domains inside the project
model:

- electrical intent is authored through schematic/electrical projections
- physical implementation is authored through PCB/layout projections
- connectivity used by layout is derived from electrical intent unless a
  board-first or reverse-engineering state explicitly says otherwise
- mismatches are represented as authored relationship bindings plus derived
  resolver status, not hidden drift
- changes that cross authority boundaries become proposals unless the user
  explicitly commits an authored relationship or deviation change

The authority mechanism is the same as Decisions 000, 000D, 000E, and 001:
the engine-owned `ProjectResolver` assembles one in-memory `DesignModel` from
segmented source shards. Schematic sheets, boards, library bindings,
relationship records, variants, manufacturing plans, and journal segments are
persistence partitions, not competing authorities. Schematic and PCB
projections read and mutate the same resolved model through typed operations.

Normal forward flow:

1. User authors electrical intent.
2. Datum resolves connectivity, parts, and implementation requirements.
3. User implements that intent physically on one or more boards.
4. DRC/ERC/relationship checks report incomplete, divergent, or invalid
   implementation.
5. Intentional deviations are recorded explicitly.

Reverse or board-first flow:

1. User authors or imports physical implementation first.
2. Datum records authored `RelationshipKind` bindings such as `BoardOnly` or
   `ReverseEngineered` for the affected stable object IDs.
3. Electrical intent can be reconstructed, linked through `ComponentInstance`
   and `NetId` identity, or left incomplete.
4. Any inferred schematic data is provenance-marked and reviewable.

This makes schematic-to-PCB the normal flow without making it the only flow.
Board-first and reverse-engineering workflows use the same relationship,
identity, proposal, and transaction primitives rather than a side channel.

## User-Visible Behavior

Users should be able to:

- draw and edit a schematic without opening PCB layout
- create a PCB from schematic intent
- place components, draw routes, vias, zones, keepouts, and board geometry
- see whether physical implementation satisfies electrical intent
- select a schematic object and navigate to related physical objects
- select a PCB object and navigate to related schematic, part, rule, and check
  context
- intentionally mark a physical object as board-only or manufacturing-only
- intentionally accept a layout deviation with rationale and provenance
- reconstruct or attach schematic intent to imported board geometry
- run checks that distinguish missing implementation, unresolved mismatch,
  intentional deviation, and invalid connectivity

Datum should not force users into permanent schematic/PCB split views. Related
context may appear through inspector panes, overlays, navigation, sidecars,
tabs, tiled views, or saved workbench profiles.

## Manual Workflow Requirements

Manual users must be able to perform authority-sensitive work without AI:

- create schematic sheets and electrical connectivity
- assign parts, footprints, and pin/pad maps
- create a board or physical scope from selected electrical scope
- place components from electrical intent
- route nets and inspect airwires/connection intent
- create board-only mechanical, drawing, mounting, tooling, or process objects
- accept, reject, or resolve schematic/PCB mismatches
- back-annotate only through explicit reviewable actions
- view object relationship state and provenance in the inspector
- run ERC, DRC, and relationship checks manually

Manual tools may commit local visible edits directly through transactions.
Cross-domain changes such as electrical intent changes inferred from physical
layout, bulk ECO application, import repair, and standards deviations should
produce proposals by default.

## Authority And Resolver Mechanism

Schematic/electrical and PCB/physical authority are domain authorities inside
the resolved `DesignModel`, not file authorities.

Electrical authority owns:
- authored components, logical units, pins, wires, labels, ports, hierarchy,
  buses, no-connects, and electrical constraints
- `ComponentInstance` creation or binding when electrical intent is authored
  before placement
- stable `NetId` identity through `NetAnchor` hints and the resolved
  connectivity graph
- ERC-relevant rules, waivers, and accepted electrical-deviation metadata

Physical authority owns:
- board outlines, stackups, placement, footprints, pads, tracks, vias, zones,
  keepouts, copper, drawings, text, dimensions, and process geometry
- physical implementation of `ComponentInstance` and `NetId` relationships
- DRC/manufacturing geometry, process constraints, and physical deviations

The `ProjectResolver` derives cross-domain facts from authored source:
- resolved electrical connectivity from sheets, hierarchy, labels, ports, and
  `NetAnchor` state
- physical connectivity from board copper, pads, vias, zones, and placement
- relationship status from authored `Relationship` bindings plus the resolved
  electrical and physical graphs
- variant population and applicability from variant overlays and option/scope
  resolution

Derived resolver state is cacheable and revision-keyed. It may be persisted as
a cache or report, but it is never source authority. If derived status
conflicts with authored intent, the resolver emits a diagnostic or
relationship finding; it does not silently rewrite the authored binding.

## Identity Mechanism

Relationship decisions must target stable identities, not filenames,
reference-designator strings, names, or canvas positions.

Required identity rules:
- every authored domain object has a stable persisted `ObjectId = Uuid`
- moving an object between sheets or boards is not an identity change unless
  the user semantically replaces the object
- a `ComponentInstance` is the canonical electrical-to-physical join; both a
  schematic `PlacedSymbol` and a board `PlacedPackage` reference it
- reference designators are display/annotation fields, not relationship
  identity
- a `NetId` is stable across rename, split, and merge according to the
  `NetAnchor` rules in 000/000D
- pin-to-pad and net-to-copper relationships target object IDs, pin IDs,
  pad IDs, `ComponentInstance`, and `NetId`, not ad hoc text labels

Imported objects carry the same persisted `ObjectId = Uuid`; their deterministic
v5 import UUIDs are demoted to import seeds persisted in an Import Map shard
keyed by `import_key`, not `source_hash`. Board-first and reverse-engineering
relationships still target the persisted `ObjectId`, `ComponentInstance`, and
`NetId`, never the import seed. See Decision 011 for the Import Map mapping.

This identity layer is what allows normal forward annotation, board-first
recovery, ECO, and variant checks to share one mechanism.

## Optional AI And Tooling Behavior

AI, CLI, MCP, importers, checkers, and scripts may help reconcile schematic and
PCB state, but they must use the canonical operation/proposal/transaction
model.

Allowed behavior:
- query electrical/physical relationships
- explain why an object is unimplemented, mismatched, board-only, or deviated
- propose ECO application
- propose schematic reconstruction from board geometry
- propose layout changes to satisfy electrical intent
- propose authored relationship, deviation, or waiver changes with rationale
- run checks before and after proposal application

Disallowed behavior:
- silently redefine electrical intent from PCB geometry
- silently delete or reroute physical implementation after schematic edits
- silently mark mismatches as acceptable
- mutate source shards outside Datum transactions
- treat imported KiCad or other EDA data as inherently trusted Datum intent

## Relationship Mechanism

Datum uses a two-class relationship model. Authored bindings are source state.
Resolver status is derived state.

### Authored Relationship Bindings

A `Relationship` is an authored source record with its own stable surrogate
ID. It binds stable object identities across domains and carries a
`RelationshipKind`.

Initial `RelationshipKind` vocabulary:
- `ImplementedBy`: an electrical object, scope, component, pin, or net is
  intended to be implemented by a physical object, scope, package, pad, or
  copper connectivity group
- `BoardOnly`: a physical object intentionally has no schematic source
- `SchematicOnly`: an electrical object intentionally has no physical
  implementation
- `ReverseEngineered`: physical data existed first and any electrical intent
  is inferred, partial, or under review
- `Pending`: implementation is expected but not yet bound completely
- `Mismatch`: a user or import workflow has recorded a known unresolved
  disagreement requiring arbitration

Authored intent records include human decisions that the resolver may not
overwrite:
- `LayoutDeviation`: physical implementation intentionally differs from
  electrical intent with rationale and provenance
- accepted electrical/physical divergence metadata used during ECO or
  board-first work when the project owner has not chosen a separate named
  relationship primitive
- accepted deviation or waiver records attached to affected object IDs,
  check basis, variant/output context, actor, date, and rationale

`BoardOnly`, `SchematicOnly`, and `ReverseEngineered` are authored
relationship classifications expressed only through `RelationshipKind`.
They are persisted source model data, not comments, viewer annotations, or
importer leftovers. `ReverseEngineered` is not also a deviation or
authored-intent axis; board-first/import explanation belongs in
ImportProvenance, import-map metadata, and explicit accepted-deviation records
when the physical source intentionally diverges from electrical intent.

### Derived Resolver Status

The resolver computes relationship status from authored bindings, resolved
electrical connectivity, physical connectivity, library bindings, and active
variant overlays.

Derived status vocabulary:
- `Implemented`: the authored binding is satisfied for the relevant active
  scope, board, and variant
- `PendingImplementation`: electrical intent exists and implementation is
  expected, but the relevant physical binding or geometry is incomplete
- `UnresolvedMismatch`: electrical and physical domains disagree and no
  authored intent, accepted deviation, or waiver explains the disagreement

These statuses are resolver outputs. They may be cached or stamped in reports,
but they are recomputed on load and never persisted as relationship authority.
Derived transitions happen when the resolver recomputes after transactions:
for example, `PendingImplementation` becomes `Implemented` after placement and
routing satisfy the authored binding, and `Implemented` becomes
`UnresolvedMismatch` if later edits create unexplained divergence.

### Variant Applicability

`NotApplicableForVariant` is a derived population/applicability value, never a
stored relationship state. It is produced when the active variant's sparse
overlay, option links, or scope selection removes an object or relationship
from consideration entirely.

Switching the active variant composes a sparse overlay that stores only
`Fitted`/`Unfitted`, option-link selections, and sparse per-variant relationship
overrides, and produces zero base-object writes. `NotApplicableForVariant` is
derived during resolution and is never persisted.

Checks, outputs, BOM/PnP, assembly drawings, and UI projections may display or
stamp `NotApplicableForVariant`, but no operation may persist it as an
authored relationship status. Persisting it would create a second source of
truth beside the variant overlay and option/scope rules that determine it.

### Manufacturing-Only Scope

Production objects are scoped by manufacturing/panel-scope membership, not by a
schematic/PCB relationship state. A production object such as a panel rail, tab,
mouse bite, V-score, tooling hole, coupon, assembly note, or panel fiducial
belongs to manufacturing or panel production scope unless the user explicitly
promotes it to board-level physical geometry through an accepted board-level
operation.

Such objects may have relationships to boards, board instances, panels, output
jobs, or manufacturing plans, but membership in manufacturing/panel scope is not
a schematic/PCB relationship. They must not be misclassified as `BoardOnly`
unless the object is actually authored physical board implementation that should
participate in board-level DRC/outputs. Panelization isolation, keeping panel
features in manufacturing/panel scope and board-only artifacts free of
panel-only geometry, is a proof gate for this distinction. Whether a named
relationship-adjacent state is needed instead of pure scope membership is an
open owner question below, not first-slice doctrine.

## Operation, Proposal, And Transaction Mechanism

Every authority-sensitive edit uses the canonical edit model from Decision
001.

Direct transaction examples:
- `DrawWire` authors electrical intent
- `RouteTrack` authors physical implementation
- `PlaceComponentFromIntent` creates or binds a physical package to a
  `ComponentInstance`
- `MarkBoardOnlyObject` records a `BoardOnly` relationship for a physical
  board object

Proposal-required examples:
- infer electrical intent from imported board geometry
- back-annotate an electrical change from physical layout
- accept a `LayoutDeviation`, waiver, or other owner-approved divergence
  record
- apply a bulk ECO that changes bindings across many objects
- repair imported data or replace imported geometry basis
- create standards waivers or accepted deviations

Whether direct or proposal-driven, the committed mutation is an
`OperationBatch -> commit()` transaction. The engine applies the batch to the
resolved `DesignModel`, stages shard bytes, appends the `TransactionRecord` to
the journal as the commit point, then atomically renames staged shard files.
No GUI, CLI, MCP, AI, importer, checker, or script may update schematic, PCB,
relationship, variant, or manufacturing shards through a private writer.

Transactions are the sole producer of `object_revision` and `model_revision`.
Relationship status, variant views, resolved connectivity, and manufacturing
projections key their staleness from those revisions.

## Core Primitives Affected

- `DesignModel`
- `ProjectResolver`
- `ElectricalDesign`
- `ElectricalScope`
- `SchematicSheet`
- `PhysicalDesign`
- `Board`
- `PhysicalScope`
- `Net`
- `NetId`
- `NetAnchor`
- `ConnectivityGraph`
- `ComponentInstance`
- `PartAssignment`
- `FootprintBinding`
- `PinPadMap`
- `Relationship`
- `RelationshipKind`
- derived relationship status
- variant population/applicability value
- `ECO`
- `Operation`
- `OperationBatch`
- `Proposal`
- `Transaction`
- `TransactionRecord`
- `CheckFinding`
- `Waiver`
- `Deviation`
- `ImportProvenance`
- `ManufacturingProjection`

## Standards And Compliance Impact

Authority rules affect compliance because checks must know whether they are
checking intended design data, incomplete implementation, imported unknowns,
or accepted deviations.

Required standards behavior:
- ERC findings must report electrical-intent issues in schematic/electrical
  context.
- DRC findings must report physical-implementation issues in PCB/layout
  context.
- relationship findings must report mismatches between intent and
  implementation using derived resolver status and authored relationship
  intent separately.
- standards-aware physical checks, including pad/mask/paste aperture policy,
  must reference the governing library, rule, process, and board state.
- accepted deviations and waivers must record actor, rationale, date,
  affected objects, check basis, and variant/output context where applicable.

Datum may support standards-aware workflows, but it must not claim product or
organizational certification unless a future explicit compliance workflow
supports that claim.

## Non-Goals

- Do not require every board object to have a schematic source.
- Do not allow physical edits to silently rewrite electrical intent.
- Do not require permanent schematic/PCB split-screen workflow.
- Do not define final storage file format in this record.
- Do not define full ECO UI.
- Do not treat importer fidelity as the center of product identity.
- Do not make AI required for relationship management.

## First Proof Slice

The first proof should demonstrate:

1. Create a schematic with two sheets and at least one cross-sheet net.
2. Resolve electrical connectivity and part/footprint bindings.
3. Create stable `ComponentInstance` and `NetId` identities for the resolved
   intent.
4. Create one board implementing that electrical scope through authored
   `ImplementedBy` relationships.
5. Place components and route at least one net manually through `commit()`.
6. Show derived `PendingImplementation` for unrouted or unplaced intent.
7. Create one authored `BoardOnly` object and verify it does not require
   schematic source.
8. Create one intentional `LayoutDeviation` through a proposal accepted as a
   journaled transaction.
9. Run checks that distinguish ERC, DRC, authored relationship intent,
   derived relationship status, and accepted deviations.
10. Navigate from schematic object to PCB object and back through stable
    object IDs, `ComponentInstance`, and `NetId`.
11. Export or preview a manufacturing projection without changing authority
    state.
12. Show a variant in which one object resolves to
    `NotApplicableForVariant` without storing that value as relationship
    source state.

## Proof Gates

Before this model is treated as implementation doctrine:

- schematic edits must not rewrite unrelated board state
- PCB edits must not rewrite electrical intent without explicit transaction
- authored relationships and deviations must survive save/load
- derived relationship status must recompute deterministically on load without
  overwriting authored intent
- object IDs must remain stable across projection navigation
- `ComponentInstance` must join schematic symbol instances to board package
  instances without relying on reference designator strings
- `NetId` must remain stable across rename and deterministic across split/merge
- relationship checks must be deterministic
- imported board objects must retain provenance and unknown-basis state
- accepted deviations must be visible in checks and audit history
- variant-specific absences must not look like accidental mismatches
- `NotApplicableForVariant` must be derived from variant/option/scope
  resolution and must not appear as stored relationship source state
- manufacturing/panel-scope objects must not mutate source board geometry or be
  misclassified as schematic/PCB relationships
- every mutation in the proof must pass through `OperationBatch -> commit()`
  and produce a journaled `TransactionRecord`

## Open Owner Questions

- Should `ElectricalScope` and `PhysicalScope` be persisted primitives in v1 or
  derived resolver concepts?
- What user action should create a `LayoutDeviation` versus a waiver?
- Should `ElectricalDeviation` remain a named authored deviation type in the
  first schema, or be modeled as accepted deviation metadata on a relationship?
- Should manufacturing-only objects be a named relationship-adjacent state, or
  modeled purely as panel/manufacturing-scope membership? First-slice behavior
  should treat them as manufacturing/panel-scope membership unless the owner
  ratifies a named state.
- Should board-level mechanical/process `BoardOnly` objects and
  manufacturing/panel-only objects share a UI affordance while remaining
  distinct schema concepts?
- What relationship findings should block manufacturing output by default?
- How much reverse-engineering workflow belongs in the first proof?
- Should back-annotation ever be a direct command, or always a proposal?
- What visual language should distinguish source intent, implementation,
  mismatch, and accepted deviation?
