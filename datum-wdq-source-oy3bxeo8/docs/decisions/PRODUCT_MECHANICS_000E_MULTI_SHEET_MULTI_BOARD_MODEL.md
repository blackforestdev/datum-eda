# Product Mechanics Decision 000E: Multi-Sheet And Multi-Board Model

Status: draft hypothesis + how/mechanism woven 2026-06-18; identity,
ComponentInstance, journal, and M7 sequencing ratified; remaining forks open.
Date: 2026-06-19

Driven by:
- `docs/DATUM_PRODUCT_MECHANICS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_DOCUMENTATION_GOALS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_REVIEW_AGENDA.md`
- `docs/decisions/PRODUCT_MECHANICS_000_UNIFIED_DESIGN_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_000D_STORAGE_AND_VERSIONING_MODEL.md`
- project-owner need to model multi-sheet schematics, multiple boards,
  board-first flows, modules, daughtercards, variants, and panelization without
  turning storage shards into separate authorities

## Purpose

Define how Datum should model multiple schematic sheets, hierarchical sheets,
multiple boards, board-only or reverse-engineered flows, modules,
daughtercards, variants, and panelization boundaries.

This document does not finalize the unified `DesignModel` architecture. The
unified model remains a hypothesis until feasibility proof gates pass. This
record defines the product mechanics that the hypothesis must support.

Required distinction:

> A sheet is not the electrical design. A board is not the physical design.

Sheets and boards are authoring, presentation, implementation, and production
scopes inside a project model. They must not become hidden project-level
authorities.

## Product Intent

Datum should support professional designs that do not fit a one schematic file
to one PCB file pattern.

Common product cases include:
- one electrical design spread across many schematic sheets
- hierarchical sheets used to organize repeated or nested electrical scopes
- one electrical design implemented by more than one board
- multiple electrical scopes implemented on one physical board
- a project that starts from imported or reverse-engineered board geometry
- modules, plug-in cards, mezzanines, and daughtercards with explicit
  interconnect relationships
- variants that change fitted parts, options, rules, manufacturing outputs, or
  implementation expectations
- panelization that arranges board instances for manufacturing without
  mutating source board geometry

The user should experience one coherent project even when the project contains
many sheets, boards, variants, and production outputs.

## Product Posture

Treat the unified `DesignModel` as the leading hypothesis, not as final
doctrine.

The model is acceptable only if it can represent multi-sheet and multi-board
work without:
- forcing every project into one schematic-to-one-board workflow
- making sheets or boards independent source authorities
- hiding electrical/physical mismatches
- silently redefining electrical intent from physical edits
- silently destroying physical work after electrical edits
- making git diffs and merges impractical
- collapsing panelization into source board geometry

If proof gates show that this model is too complex or fragile, Datum should
narrow or reject the unified-model hypothesis before implementation depends on
it.

## Core Distinctions

### Electrical Design

The electrical design is authored electrical intent.

It includes:
- components and logical units
- pins, ports, wires, buses, labels, hierarchy, and connectivity intent
- electrical constraints and rules
- variant-aware fitted or unfitted electrical options
- relationships to library parts, symbols, models, and physical
  implementations

An electrical design may be shown across one or many schematic sheets. It may
also exist partially, as in board-first recovery or early architecture work.

### Sheet

A sheet is an electrical authoring and presentation scope.

It includes:
- sheet identity and metadata
- placed symbols and annotation presentation
- wires, buses, labels, ports, junctions, no-connects, and graphics
- hierarchical sheet instances or references
- local presentation state

A sheet helps users author and understand electrical intent. It is not the
electrical design itself.

Moving an object between sheets should not change the object's stable identity
unless the user semantically replaces the object.

### Physical Design

The physical design is authored physical implementation and manufacturing
geometry across one or more physical scopes.

It includes:
- board outlines and stackups
- component placements
- footprint and pad instances
- tracks, vias, zones, copper, keepouts, drawings, text, and mechanical
  geometry
- physical rules and manufacturing constraints
- physical relationships to electrical intent, modules, variants, panels, and
  artifacts

A physical design may contain one board, many boards, or imported board
geometry before complete electrical intent exists.

### Board

A board is a physical implementation scope.

It includes:
- board identity and metadata
- board outline and stackup reference
- component placements and footprint instances
- copper, vias, zones, keepouts, text, dimensions, and process geometry
- board-level rules, overrides, and manufacturing assumptions where allowed

A board is not the whole physical design. A project may contain several
boards, and a board may implement all, part, or none of the currently authored
electrical design.

### Panel

A panel is a manufacturing production scope.

It references board instances and adds production-only features such as rails,
tabs, mouse bites, V-scores, fiducials, tooling holes, coupons, route paths,
labels, and panel notes.

Panelization must not mutate source board geometry unless the user explicitly
requests a board-level change.

## User-Visible Behavior

Datum should present multi-sheet and multi-board projects as normal project
structure, not as an advanced escape hatch.

Expected behavior:
- users can create, rename, duplicate, move, and delete sheets
- users can create hierarchy using sheet symbols, ports, labels, and repeated
  sheet instances where supported
- users can create, rename, duplicate, move, and delete boards
- users can open electrical, physical, library, rules, manufacturing, BOM, and
  analysis projections from the same project tree
- users can navigate from a schematic object to related physical objects and
  back
- users can navigate from a board object to related electrical intent, part,
  footprint, rule, check, variant, and manufacturing context
- users can see whether a component, net, connector, board feature, or module
  is implemented, pending, intentionally deviated, mismatched, board-only, or
  reverse-engineered
- users can work board-first without being forced to invent a fake schematic
  immediately
- users can generate board-level and panel-level outputs with explicit target
  board, panel, variant, manufacturing plan, and output job context

The UI may use tabs, panes, inspectors, overlays, workbench profiles,
sidecars, or saved viewports. The UI arrangement does not define design
authority.

## Manual Workflow Requirements

Datum must remain usable without AI.

Minimum manual multi-sheet workflow:
- create a project with one or more sheets
- place and edit symbols, wires, labels, buses, ports, junctions, no-connects,
  and graphical annotations on selected sheets
- create hierarchical sheet references or instances once hierarchy is in
  scope
- inspect resolved connectivity across sheet boundaries
- run ERC across the resolved electrical design, not only the active sheet
- move electrical objects between sheets while preserving identity when the
  electrical object is the same
- identify sheet-local presentation changes separately from electrical intent
  changes

Minimum manual multi-board workflow:
- create a project with one or more boards
- select which electrical scope, module, variant, or relationship set a board
  is intended to implement
- place, route, edit, and check objects on a selected board
- mark physical objects as implementing electrical intent, board-only,
  reverse-engineered, intentionally deviated, or unresolved
- run DRC for one board and cross-domain checks for the relevant project scope
- generate board-level outputs for a selected board, variant, and output job
- create panel-level production plans that reference board instances without
  editing board source geometry

Minimum manual board-first workflow:
- import or create physical board geometry without complete electrical intent
- assign board-only and reverse-engineered states to physical objects
- progressively infer components, nets, connectors, and electrical intent
- accept, reject, or defer inferred relationships
- preserve import provenance and original geometry basis
- avoid silently converting inferred electrical data into final authored
  electrical intent

## Optional AI And Tooling Behavior

AI, scripts, importers, and external tools may assist with multi-sheet and
multi-board work, but they must use the same primitives as manual tools.

Allowed optional behavior:
- propose sheet decomposition or hierarchy changes
- propose moving objects between sheets for readability
- infer likely electrical intent from board-only or imported geometry
- propose component-to-footprint, pin-to-pad, and net-to-copper relationships
- compare multiple boards against a shared electrical scope
- identify unresolved implementation gaps between electrical and physical
  domains
- propose variant-specific fitted or unfitted component changes
- propose module, daughtercard, connector, or inter-board relationship records
- propose panelization features and output job settings
- run ERC, DRC, relationship checks, and manufacturing checks before applying
  proposals

Required behavior:
- tool and AI edits must be proposals or transactions with provenance
- inferred relationships must be visible and reviewable
- AI must not silently resolve electrical/physical mismatches
- AI must not invent fake electrical intent to satisfy board geometry
- AI must not mutate source board geometry as part of panelization unless the
  user explicitly approves a board-level change

## Core Primitives

This record extends the existing primitives rather than replacing them.

### DesignModel

The resolved project authority, if the unified-model hypothesis passes.

Contains electrical design state, physical design state, sheets, boards,
relationships, variants, manufacturing plans, output jobs, proposals,
transactions, artifacts, and provenance.

### ElectricalScope

A named authored electrical scope.

Examples:
- whole product electrical design
- hierarchical block
- repeated channel
- module interface
- connector-defined subsystem
- recovered or partial board-first scope

An `ElectricalScope` may be presented on one or many sheets and may map to one
or many boards.

### Sheet

Electrical authoring and presentation scope.

Stores visual placement and sheet-local authored presentation while referencing
stable electrical object IDs.

### HierarchyInstance

An instance of an electrical scope within another electrical scope.

Defines:
- referenced electrical scope or sheet group
- instance identity
- port/interface bindings
- repetition/indexing where supported
- variant and parameter bindings where supported

Hierarchy must resolve to explicit electrical connectivity for checks,
relationships, and outputs.

### PhysicalScope

A named physical implementation scope.

Examples:
- one PCB
- one flex section
- one module layout
- imported board recovery scope
- mechanical/process region that participates in board production

For the first implementation, a physical scope may map directly to a `Board`.
The distinction remains important because a future physical design may contain
multiple boards or non-board physical scopes.

### Board

Physical implementation scope for a PCB or board-like object.

Stores board outline, stackup reference, placements, routing, copper, zones,
keepouts, drawings, and manufacturing geometry.

### Module

A reusable or project-local electrical/physical design unit.

A module may include:
- electrical scopes
- physical scopes or boards
- library bindings
- interface definitions
- constraints
- manufacturing assumptions
- variant options

Modules can support daughtercards, mezzanines, repeated channels, validated
subcircuits, and project-local reusable blocks. Module reuse must preserve
identity and relationship semantics rather than copying anonymous geometry by
default.

### Interconnect

An explicit relationship between electrical or physical scopes.

Examples:
- connector-to-connector relationship between a main board and daughtercard
- board edge contact interface
- cable or harness relationship
- mezzanine connector stack
- module boundary port mapping
- panel or fixture connection that is manufacturing-only

Interconnects should be checkable and variant-aware where relevant.

### Variant

A named configuration of fitted parts, options, rules, implementation
expectations, manufacturing targets, and outputs. It is configuration state,
not an informal comment, and it affects checks and production outputs.

How a variant is stored without becoming a second authority: a variant is a
sparse authored **overlay** over the canonical `DesignModel`, not a separate
copy of it. The overlay is keyed by stable object identity (`ObjectId = Uuid`,
persist-on-create) and records only what the variant authors: population
decisions, selected option-link choices, and sparse per-variant authored
relationship overrides. Switching the active variant swaps an overlay handle
and produces zero base writes, so adding variants does not expand the
relationship shard beyond the few objects that genuinely differ. Derived
variant views and graph materializations are invalidated by
`model_revision` plus `variant_revision`; they are caches, not source
authorities.

Population is three-valued at resolution time — `Fitted`, `Unfitted`, and
`NotApplicableForVariant` — but only the first two are authored. `Fitted` and
`Unfitted` express a deliberate build decision and are stored in the overlay;
`NotApplicableForVariant` is **derived**, never stored (see Relationship
Vocabulary below). Population-only variants resolve as a no-clone late-bound
view over the base model. Variants that change net topology (option links or
net-merges) require a bounded **derived materialization** of the resolved
graph — a cache, not a second authority, acceptable under the single-authority
model.

Honest scope correction: cross-domain population coherence (deriving a
variant-correct BOM from schematic intent and reflecting it onto the board)
requires the `ComponentInstance` surrogate ratified in 000D / 000 — the
per-instance stable join that both `PlacedSymbol` and `PlacedPackage`
reference. Until `ComponentInstance` is linked at import and forward
annotation, variants are **board-local**, keyed on the package Uuid only.

Variants may affect:
- BOM and PnP
- ERC/DRC expectations
- component population (three-valued, as above)
- net relationships where option links are present
- board implementation requirements
- assembly drawings
- output jobs and artifact identity

### Relationship

An explicit cross-domain binding or state record.

Examples:
- electrical component implements physical footprint instance
- electrical pin maps to physical pad
- electrical net maps to copper connectivity
- electrical scope maps to board implementation scope
- board-only mounting hole intentionally has no schematic source
- reverse-engineered trace is linked to inferred net
- daughtercard connector maps to main-board connector
- board instance is used by a panel

Relationships prevent sheets and boards from becoming separate authorities.

## Relationship Vocabulary

Multi-sheet and multi-board modeling uses a three-axis relationship model at
object, scope, board, module, interconnect, and variant boundaries. Older flat
"relationship state" terms may still appear as user-visible condition labels,
but the persisted model must keep these axes separate.

Authored `RelationshipKind` bindings define the intended cross-domain link:
component-to-package, pin-to-pad, net-to-copper, scope-to-board,
module-interface, interconnect, panel/fixture usage, or similar bindings.

Derived resolver status is computed from the active model and variant:
- `Implemented`: the relevant authored binding or scope is satisfied.
- `PendingImplementation`: electrical intent exists but physical
  implementation is incomplete for the selected board or scope.
- `UnresolvedMismatch`: electrical and physical domains disagree, and no
  accepted decision explains the difference.

Authored intent and deviation records explain intentional divergence:
- `LayoutDeviation`: physical implementation intentionally differs from
  electrical intent and the accepted rationale is recorded.
- `ReverseEngineered`: physical implementation exists first; electrical intent
  is inferred, partial, or not yet accepted as authored truth.
- `BoardOnlyObject`: physical object intentionally has no schematic source.
- `SchematicOnlyObject`: electrical object intentionally has no physical
  implementation.

`NotApplicableForVariant` is resolved, not open. It is a first-class
**derived** value, never a stored relationship fact. During resolution it is
produced when an option link or scope the active variant does not select
removes an object from consideration entirely. Checks and outputs may read and
stamp it, but it is never persisted, because storing it would create a second
source of truth alongside the option-link and rule state that actually
determines it. This aligns with the reconciled split: AUTHORED relationship
facts are `RelationshipKind` bindings plus intent/deviation records, DERIVED
resolver status is `{Implemented, PendingImplementation,
UnresolvedMismatch}`, and `NotApplicableForVariant` is a derived population
value.

Additional relationship questions for owner review:
- whether `ModuleInterfaceOnly` is needed for connector/interface intent that
  is checked but not implemented as normal board geometry
- whether `ManufacturingOnlyObject` should be distinct from `BoardOnlyObject`
  for fiducials, coupons, panel rails, tooling holes, and assembly notes

## Modeling Patterns

### Multi-Sheet Schematic

One electrical design may span many sheets.

Rules:
- connectivity is resolved across sheets using explicit labels, ports,
  hierarchy, and project rules
- sheet-local visual placement is not electrical identity
- sheet movement is a presentation edit unless it changes connectivity or
  hierarchy
- ERC runs against the resolved electrical scope, with findings anchored back
  to sheet locations where possible
- generated documentation may use saved sheet viewports, but viewports are not
  design authority

### Hierarchical Sheets

Hierarchy organizes electrical scopes and repeated structures.

Rules:
- a hierarchical sheet or block references an electrical scope
- each hierarchy instance has stable instance identity
- port and interface bindings are explicit
- repeated instances must resolve to distinct instance-level objects and nets
  where required
- variants and parameters may affect hierarchy resolution only through
  explicit model state
- relationship checks must know whether a board implements the parent scope,
  child scope, one instance, or several repeated instances

### Multiple Boards

A project may contain more than one board.

Supported patterns:
- one electrical design split across main board and daughtercard
- one electrical design implemented by alternate board layouts
- one board implementing multiple electrical scopes
- one product project containing several cooperating boards
- imported boards being reverse-engineered into one or more electrical scopes
- a module with its own board used inside a larger product project

Rules:
- each board has stable identity independent of filename and display name
- board-level DRC runs against a selected board
- cross-domain checks run against the board's intended electrical and variant
  scope
- moving a component from one board to another is a physical implementation
  edit and may change user-visible relationship condition labels, but it must
  not silently change electrical intent
- deleting a board must not delete electrical design objects unless the user
  explicitly accepts that separate operation

### Board-Only And Reverse-Engineered Flows

Datum must support physical-first work.

Rules:
- imported or manually created physical objects may exist without electrical
  sources
- such objects require explicit relationship classification such as
  `RelationshipKind::BoardOnly` or `RelationshipKind::ReverseEngineered`, not
  fake schematic objects
- inferred components, nets, and pin mappings are proposals until accepted
- accepted inferred electrical intent records source geometry and provenance
- physical cleanup or manufacturing correction does not automatically rewrite
  inferred electrical intent
- board-first projects can remain board-only if that is the user's intent

First proof seed (Slice 0) is a board-first, all-`BoardOnly` or
`ReverseEngineered` seed of the datum-test fixture: every recovered package
starts with an explicit `RelationshipKind` classification and no schematic
source. Now that
`ComponentInstance` is approved as the electrical<->physical join, the
immediate follow-on seed is an `ImplementedBy` seed — and because datum-test
**does** have a schematic available, that `ImplementedBy` seed is a viable
alternative to the board-first `BoardOnly` seed for exercising cross-domain
relationship condition labels earlier.

### Modules And Daughtercards

Modules and daughtercards require explicit boundaries.

Rules:
- a module may contain electrical scopes, physical scopes, boards, library
  bindings, rules, and interface definitions
- a daughtercard is a board or module with explicit interconnect relationships
  to another board or module
- connector, cable, board-edge, and mezzanine relationships should be model
  objects that checks can inspect
- module reuse must preserve object identity and provenance expectations
- copying a module as a new independent design unit must create new identities
  where semantic independence is intended

### Variants

A variant is configuration state, not a comment. To keep one resolved
authority while still letting a variant change what is built, a variant is
stored as a sparse authored overlay over the canonical model, not as a
separate copy of it. The overlay records, keyed by stable object identity,
only the objects whose population the variant changes, the option-link choices
the variant selects, and any authored relationship override that must differ
for that variant. Population is three-valued at resolution time — `Fitted`,
`Unfitted`, and `NotApplicableForVariant` — but only the first two are
authored. `Fitted` and `Unfitted` express a genuine build decision (a part is
deliberately stuffed or deliberately omitted) and are therefore stored in the
variant overlay. `NotApplicableForVariant` is not authored: it is derived
during resolution when an option link or scope the variant does not select
removes an object from consideration entirely. It is a first-class resolved
value that checks and outputs may read and stamp, but it is never stored,
because storing it would create a second source of truth alongside the
option-link and rule state that actually determines it. Because the overlay is
keyed by stable object identity, a part renamed or moved between sheets carries
its variant intent with it untouched; and because relationship divergence is
stored as a sparse per-variant authored override rather than a full
relationship record per variant, adding variants does not expand the
relationship shard beyond the few objects that genuinely differ. Resolved
variant views are invalidated by `model_revision` plus `variant_revision`;
they are derived caches, not a second source of truth.

Resolution cost depends on what the variant changes. Population-only variants
are no-clone late-bound views: switching the active variant swaps an overlay
handle with zero base writes. Variants that change net topology (option links
or net-merges, where the survivor rule is lowest `NetId` wins) require a
bounded derived materialization of the resolved graph — a cache, not a second
authority, and acceptable under the single-authority model.

Rules:
- variants can change fitted parts, assembly options, output targets, and
  implementation expectations
- a physical board may support multiple assembly variants
- an electrical scope may have variant-dependent fitted components or options
- authored relationship overrides may differ by variant, stored sparsely by
  stable object identity
- BOM, PnP, ERC, DRC, assembly drawings, panel outputs, and artifact metadata
  must identify variant context
- unfitted parts should not look like accidental missing implementation
  when the active variant says they are intentionally absent
- cross-domain (BOM-from-schematic) population coherence requires the
  `ComponentInstance` join; until it lands, variants are board-local keyed on
  the package Uuid

### Panelization Boundary

Panelization belongs to manufacturing production intent.

Rules:
- panels reference board instances with transforms
- panels may add rails, tabs, mouse bites, V-scores, fiducials, tooling holes,
  coupons, route paths, labels, and manufacturing-only notes
- panel features are manufacturing-plan or panel-projection state by default
- panel outputs identify board, panel, variant, manufacturing plan, output job,
  generator version, and source model revision
- panel edits must not mutate source board geometry unless the user explicitly
  applies a board-level edit

## Standards And Compliance Impact

Multi-sheet and multi-board support expands the scope of checks and generated
evidence.

Required impacts:
- ERC must resolve connectivity across sheets and hierarchy
- DRC must identify the selected board, variant, rules, and manufacturing
  context
- cross-domain checks must report user-visible electrical/physical condition
  labels at object and scope level, derived from bindings, resolver status,
  and authored intent/deviation records
- inter-board and module-interface checks should be possible when connectors,
  cables, edge contacts, or mezzanine relationships are modeled
- manufacturing checks must distinguish board source geometry from panel
  production features
- artifact metadata must identify model revision, board or panel target,
  variant, manufacturing plan, output job, and generator version
- waivers and deviations must record the affected scope, board, variant, and
  authored intent/deviation record where applicable

Compliance evidence must not depend on a UI arrangement such as which sheets
or boards are open in tabs.

## Storage And Versioning Impact

The storage model must support many sheets and many boards while preserving
one resolved authority.

Expected source shards:
- project manifest
- electrical sheet shards
- hierarchy or electrical-scope shards where needed
- resolved electrical graph cache, if persisted
- physical board shards
- relationship shards
- module or interconnect shards where needed
- variant overlay shards (sparse authored overlays keyed by stable identity,
  zero base writes; see the Variant primitive)
- the Import Map shard (import seeds keyed by `import_key`, for imported
  objects whose v5 id is demoted to a seed)
- rules and constraints shards
- manufacturing plan shards
- output job shards
- proposal shards and the transaction journal (the persisted, replayable,
  auditable history; undo/redo are cursors into it)
- artifact metadata shards

Required behavior:
- adding a sheet should not rewrite unrelated boards
- moving an electrical object between sheets should preserve stable object ID
  when the object is semantically unchanged
- adding a board should not rewrite unrelated sheets
- editing board placement should not rewrite electrical sheets unless
  electrical intent changes
- accepting inferred board-first electrical intent should touch electrical,
  relationship, provenance, and transaction state explicitly
- changing a variant should touch only that variant's sparse overlay shard,
  never look like editing every object in every sheet or board
- panel edits should touch manufacturing plan or panel state, not source board
  geometry

Files are persistence partitions (shards), never authorities. The resolved
`DesignModel` assembled by the engine-owned `ProjectResolver` is the single
authority; the variant overlay, the resolved electrical graph, zone fill, and
manufacturing projections are all derived state, revision/hash-keyed off
`object_revision` / `model_revision`, never a second source of truth.

## Non-Goals

This decision does not require:
- implementing every hierarchy feature in the first slice
- implementing reusable module packaging before basic multi-board support
- supporting every vendor-specific multi-board or harness format immediately
- treating every connector as a full cable/harness design
- forcing all projects to define hierarchy, modules, variants, or panels
- making panelization part of source board geometry by default
- choosing the final file serialization format
- declaring the unified `DesignModel` hypothesis proven

## Proof Gates

The multi-sheet and multi-board model is viable only if these gates can be
demonstrated.

### Proof Gate 1: Sheet Is Not Electrical Design

Create one electrical scope across two sheets and move one component or net
label presentation between sheets.

Pass criteria:
- stable electrical object identity is preserved where semantics are unchanged
- only expected sheet, relationship/cache, and transaction state changes
- ERC runs against resolved electrical intent, not only the active sheet
- related physical implementation state remains explicit

### Proof Gate 2: Board Is Not Physical Design

Create one project with two boards and one shared or related electrical scope.

Pass criteria:
- each board has stable identity
- board-level edits touch only the expected board, relationship, and
  transaction state
- checks can run for one selected board and relevant electrical/variant scope
- deleting or duplicating one board does not silently delete or duplicate the
  electrical design

### Proof Gate 3: Board-First Recovery

Import or author board geometry without a complete schematic, then infer one
component and one net relationship.

Pass criteria:
- board-only and reverse-engineered objects are explicit
- inferred electrical intent is proposed before becoming accepted source state
- original import provenance remains visible
- no fake schematic authority is required to edit the board

### Proof Gate 4: Module Or Daughtercard Relationship

Model a main board and daughtercard connected through explicit connector or
interface relationships.

Pass criteria:
- each board remains an independent physical scope
- interconnect relationships are checkable model objects
- unresolved connector or pin mapping gaps are visible
- output artifacts identify the selected board, variant, and output context

### Proof Gate 5: Variant-Aware Relationship

Create one variant where a component is fitted and one where it is unfitted.

Pass criteria:
- user-visible relationship condition labels and checks reflect the active
  variant
- BOM, PnP, and assembly output context identify the variant
- unfitted-by-variant components do not appear as accidental missing physical
  implementation
- variant edits produce reviewable diffs

### Proof Gate 6: Panelization Isolation

This is the consolidated `PG-PANELIZATION-ISOLATION` gate (gate 7 of the
10-gate proof plan whose authoritative definition lives in 000D). It was
deduplicated across the decision family — 000F states the same gate; do not
maintain a second copy. Likewise, the variant pass criteria below are the
`PG-VARIANT-RESOLUTION` (population-only) gate (gate 8 in 000D).

Create a panel from one or more board instances and add production-only panel
features.

Pass criteria:
- source board geometry remains unchanged
- panel features belong to manufacturing plan or panel state
- panel output identifies board instance transforms, variant, manufacturing
  plan, output job, generator version, and source model revision
- board-level and panel-level artifacts are distinguishable

## Open Owner Questions

1. What is the minimum hierarchy model for the first proof: flat multi-sheet
   only, simple sheet symbols and ports, or repeated parameterized instances?
2. Should `ElectricalScope` and `PhysicalScope` become explicit persisted
   primitives immediately, or should the first implementation derive them from
   sheets and boards?
3. What is the first supported multi-board case: main board plus daughtercard,
   alternate board implementations, or multiple boards in one product project?
4. Should `ManufacturingOnlyObject` be a distinct authored intent label from
   `BoardOnlyObject` for panel features, fiducials, coupons, and tooling holes?
5. (Resolved 2026-06-18.) `NotApplicableForVariant` is a first-class derived
   population value, never a stored relationship fact — see Relationship
   Vocabulary and the Variants pattern.
6. How much module reuse should Datum support before first-class library
   packaging exists?
7. What import or reverse-engineering workflow is the first board-first proof
   based on?
8. Should inter-board cables and harnesses be modeled in this decision family,
   or deferred as a separate product-mechanics track?
9. What check must block manufacturing output when a board has unresolved
   electrical/physical condition labels?
10. What owner-approved criteria would reject or narrow the unified
    `DesignModel` hypothesis for multi-sheet or multi-board work?
