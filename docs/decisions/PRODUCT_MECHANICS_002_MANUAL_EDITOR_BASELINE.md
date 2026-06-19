# Product Mechanics Decision 002: Manual Editor Baseline

Status: draft hypothesis + how/mechanism woven 2026-06-18; aligned with
000-001 commit/journal model.
Date: 2026-06-18

Driven by:
- `docs/DATUM_PRODUCT_MECHANICS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_REVIEW_AGENDA.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_DOCUMENTATION_GOALS.md`
- `docs/decisions/PRODUCT_MECHANICS_000_UNIFIED_DESIGN_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_000B_VIEW_COMPOSITION_AND_LIVE_PRODUCTION.md`
- `docs/decisions/PRODUCT_MECHANICS_001_CANONICAL_EDIT_MODEL.md`

## Decision Scope

Define the minimum manual editor capability Datum needs before it can honestly
be treated as a professional EDA editor.

This decision covers manual electrical, physical, library, rules/checks, and
manufacturing editor behavior. It does not define final implementation order or
complete feature parity with mature EDA suites.

## Product Intent

Datum must be a manual-first professional EDA application with optional AI
collaboration.

The user must be able to create, inspect, edit, validate, and prepare a board
for manufacture without using AI. AI may accelerate or review work, but manual
capability is the baseline product truth.

Required product rule:

> If a user cannot complete a core EDA workflow manually, the workflow is not
> complete, even if an AI agent can perform or fake it.

## Decision

Datum shall define manual editor readiness across five baseline domains:
- electrical editor
- physical PCB editor
- library editor
- rules and checks editor
- manufacturing/output editor

Each domain must expose direct user tools, inspector/property editing,
selection, snapping/grid behavior, undo/redo through transactions, and
deterministic checks where applicable.

Manual editor tools mutate the canonical `DesignModel` through typed
operations, proposals where required, and the single journaled `commit()`
primitive defined in Decision 001. They are not separate file editors and do
not bypass provenance.

All manual-editor surfaces operate on the resolved `DesignModel` assembled by
the engine-owned `ProjectResolver` from source shards. Project open, check
runs, projections, artifact generation, and variant switching read the
resolver-produced model at a known `model_revision`, never on-disk shards
directly. A split or incoherent project opens in resolver recovery or
diagnostic mode, not as accepted truth.

Concretely: GUI canvas tools, inspector/property editors, library editors,
rules editors, and manufacturing setup panels must construct an
`OperationBatch` and either call `commit()` directly or create a `Proposal`
that later calls the same `commit()` path when accepted. They must not write
schematic, board, library, rules, manufacturing, artifact metadata, or
workspace source shards directly. Any helper that mutates a backing file
without an operation, transaction, durable undo data, and provenance is a
migration defect, not an allowed editor shortcut.

## User-Visible Behavior

Users should experience Datum as a normal professional EDA editor:
- create or open a project
- draw schematics manually
- assign parts and footprints manually
- place and route a PCB manually
- edit symbols, footprints, and part metadata manually
- define project rules manually
- run ERC, DRC, and manufacturing checks manually
- inspect and fix findings manually
- preview manufacturing outputs manually
- generate fabrication, assembly, BOM, and pick-and-place artifacts manually

AI surfaces, terminal-launched agents, CLI tools, and scripts may be present,
but no core editor command may require them.

## Manual Workflow Requirements

### Electrical Editor

Minimum electrical projection capability:
- create sheets and sheet metadata
- place symbols
- move, rotate, mirror, copy, paste, and delete symbols
- draw wires, buses, labels, ports, power symbols, no-connects, and junctions
- edit reference designators, values, fields, units, pin visibility, and
  symbol properties
- select by object, net, sheet, component, class, and finding
- inspect connectivity and unresolved electrical states
- annotate or renumber components
- assign or change parts and footprint bindings
- run ERC and navigate findings
- preserve undo/redo and transaction provenance for every edit

First operation families:
- `SheetStructureOps`: `CreateSheet`, `RenameSheet`, `MoveElectricalObject`,
  `DeleteSheetObject`
- `SymbolPlacementOps`: `PlaceSymbol`, `MoveSymbol`, `RotateSymbol`,
  `MirrorSymbol`, `DeleteSymbol`
- `ConnectivityOps`: `DrawWire`, `DeleteWire`, `PlaceJunction`, `PlaceLabel`,
  `PlacePort`, `PlaceNoConnect`, `RenameNet`
- `AnnotationOps`: `SetReferenceDesignator`, `SetValue`, `SetField`,
  `AnnotateComponents`
- `BindingOps`: `AssignPart`, `AssignFootprint`, `SetPinPadMap`

The electrical editor must support normal schematic capture. It may start with
a limited symbol set, but the user must not need imports, scripts, or AI to
author a small schematic.

### Physical PCB Editor

Minimum physical projection capability:
- create and edit board outline
- define stackup basics and layer visibility
- place footprints and board-level mechanical objects
- move, rotate, align, lock, copy, paste, and delete placements
- route tracks and arcs where supported
- place vias, pads, zones, keepouts, copper pours, text, dimensions, and
  fiducials
- edit net assignment, width, clearance, layer, via, pad, mask, paste, and
  plating properties
- show ratsnest/airwires and route-completion state
- support grid, snap, angle, shove/avoidance mode where available, and
  selection filters
- run DRC and navigate findings
- preserve undo/redo and transaction provenance for every edit

First operation families:
- `BoardStructureOps`: `CreateBoard`, `SetBoardOutline`, `SetStackupBasics`,
  `SetLayerVisibility`
- `PlacementOps`: `PlaceFootprint`, `MovePlacement`, `RotatePlacement`,
  `LockPlacement`, `DeletePlacement`
- `RoutingOps`: `RouteTrack`, `EditTrack`, `DeleteTrack`, `PlaceVia`,
  `SetRouteLayer`
- `CopperAndProcessOps`: `PlaceZoneBoundary`, `SetZonePolicy`,
  `PlaceKeepout`, `SetPadAperture`, `SetMaskPastePolicy`
- `MechanicalOps`: `PlaceBoardText`, `PlaceDimension`, `PlaceFiducial`,
  `PlaceMechanicalShape`
- `RelationshipOps`: `BindPhysicalToElectrical`, `MarkBoardOnly`,
  `MarkReverseEngineered`, `AcceptLayoutDeviation`

The physical editor must let a user place and route a simple PCB manually even
if advanced push-and-shove, autorouting, length tuning, and 3D collision
features are not complete yet.

### Library Editor

Minimum library capability:
- create and edit symbols
- create and edit footprints
- define pins, pads, padstacks, outlines, courtyards, assembly graphics, 3D
  model references, and process geometry
- create reusable parts with manufacturer, supplier, value, tolerance,
  lifecycle, package, symbol, footprint, and pin-pad mapping data
- validate symbol pin data against footprint pad data
- validate package/process assumptions where rules exist
- manage project-local library objects before global library workflows are
  complete
- apply library changes to placed design objects through explicit operations
  or proposals

First operation families:
- `SymbolLibraryOps`: `CreateSymbol`, `EditSymbolPin`, `EditSymbolGraphic`,
  `SetSymbolField`
- `FootprintLibraryOps`: `CreateFootprint`, `EditPad`, `EditPadstack`,
  `EditCourtyard`, `EditAssemblyGraphic`, `SetModelReference`
- `PartLibraryOps`: `CreatePart`, `SetPartMetadata`, `BindSymbol`,
  `BindFootprint`, `SetManufacturerData`
- `LibraryApplyOps`: `ApplyLibraryRevisionToInstance`,
  `CreateProjectLocalOverride`

Library editing is a core manual workflow, not a hidden import dependency.

### Rules And Checks Editor

Minimum rules/checks capability:
- create and edit net classes, clearance rules, width rules, via rules, layer
  rules, differential/length constraints where supported, keepouts, zones, and
  manufacturing process constraints
- run ERC, DRC, and manufacturing checks on demand
- show findings with severity, location, affected objects, explanation, and
  status
- mark accepted deviations or waivers explicitly
- navigate from a finding to the relevant electrical, physical, library, or
  manufacturing projection
- prevent check findings and waivers from becoming hidden export settings

First operation families:
- `RuleAuthoringOps`: `CreateRule`, `EditRule`, `DeleteRule`, `SetRuleScope`
- `ConstraintClassOps`: `CreateNetClass`, `AssignNetClass`, `SetWidthRule`,
  `SetClearanceRule`, `SetViaRule`
- `CheckRunOps`: `RunErc`, `RunDrc`, `RunManufacturingCheck`,
  `RecordCheckResult`
- `FindingDispositionOps`: `AcceptDeviation`, `CreateWaiver`,
  `ResolveFinding`, `ReopenFinding`

Rules must be editable by users. A checker that only emits errors without a
manual rule-editing path is not enough.

### Manufacturing Editor

Minimum manufacturing/output capability:
- define output jobs
- preview Gerber, NC drill, soldermask, paste, fabrication drawing, assembly
  drawing, BOM, and pick-and-place projections
- inspect generated production geometry beside source layout using tabs,
  panes, PiP, tile, or floating views
- edit manufacturing notes, output settings, variant/output-job selection, and
  basic panel/manufacturing-plan metadata
- generate versioned artifacts tied to model revision and settings
- run manufacturing completeness and process checks

The authored `Zone.polygon` boundary is not exportable copper. Derived
`ZoneFill{Filled|Unfilled|Stale|Unsupported}` is separate from the polygon:
only `Filled` contributes copper to a projection or export. Gerber/copper
previews must render unfilled, stale, or unsupported zones as such and emit a
hard finding with no copper, never treating a boundary polygon as poured
copper.

First operation families:
- `OutputJobOps`: `CreateOutputJob`, `EditOutputJob`, `SelectVariant`,
  `SetLayerMapping`, `SetUnitsAndOrigin`
- `ManufacturingPlanOps`: `CreateManufacturingPlan`, `EditAssemblySide`,
  `SetProcessAssumption`, `AddManufacturingNote`
- `PanelOps`: `CreatePanel`, `PlaceBoardInstance`, `AddRail`, `AddTab`,
  `AddMouseBites`, `AddVScore`, `AddToolingHole`, `AddCoupon`
- `ArtifactOps`: `GenerateArtifactSnapshot`, `RecordArtifactMetadata`,
  `CompareArtifactToProjection`

`SelectVariant` composes a sparse authored overlay keyed by stable object
identity, storing only `Fitted`/`Unfitted` and sparse per-variant relationship
overrides. Switching the active variant produces zero base writes;
`NotApplicableForVariant` is derived during resolution and is never stored.

Manufacturing outputs are live production projections and artifact snapshots,
not opaque files produced after the design disappears from view.

## Optional AI And Tooling Behavior

AI, CLI, MCP, scripts, routers, and checkers may:
- inspect the same design model the manual tools edit
- propose operations or operation batches
- explain design state, checks, and manufacturing risk
- generate candidate fixes
- run ERC, DRC, and manufacturing checks
- prepare artifact-generation proposals

They may not:
- be required to draw schematics or boards
- mutate project files outside transactions
- create private geometry or hidden rules
- silently accept deviations or waivers
- make Datum appear more capable than its manual editor actually is

The same rule applies to built-in GUI tools. A GUI shortcut may skip proposal
review only when Decision 001 allows direct commit, but it may not skip typed
operations, validation, journal append, atomic shard promotion, undo data, or
provenance.

## Core Primitives

Affected primitives:
- `DesignModel`
- `Projection`
- `EditorContext`
- `Operation`
- `OperationBatch`
- `Transaction`
- `Proposal`
- `ElectricalIntent`
- `PhysicalImplementation`
- `LibraryPart`
- `Symbol`
- `Footprint`
- `Rule`
- `Check`
- `ManufacturingProjection`
- `Artifact`
- `WorkbenchProfile`

## Standards And Compliance Impact

Manual editor capability must expose standards and process assumptions where
they affect authored geometry or outputs.

Examples:
- mask and paste apertures must be inspectable and editable
- drill plating and slot semantics must be visible
- courtyard, clearance, creepage, and assembly constraints must be editable as
  rules when supported
- ERC/DRC/manufacturing findings must attach to affected objects
- deviations and waivers must be explicit model objects with provenance

Datum must not imply standards compliance merely because a board can be drawn.
Compliance requires visible rules, checks, findings, waivers, and output
validation.

## Explicit Non-Goals

This decision does not require:
- full Altium, KiCad, or Cadence feature parity in the first release
- autorouting as a baseline requirement
- every advanced high-speed, RF, rigid-flex, or simulation workflow
- global enterprise library management before project-local libraries work
- AI-assisted editing before manual editing works
- live real-time recomputation for every projection on every keystroke
- replacing external CAM or fabrication review tools immediately

## First Proof Slice

The first proof slice should demonstrate:
- electrical proof: `PlaceSymbol`, `DrawWire`, and `SetField` commit as
  transactions against one electrical scope and produce durable undo entries
- binding proof: `CreatePart` or `AssignPart` plus `AssignFootprint` creates a
  project-local part/footprint binding through the same commit path
- physical proof: `SetBoardOutline`, `PlaceFootprint`, `MovePlacement`, and
  `RouteTrack` commit against one board without touching unrelated electrical
  shards
- relationship proof: at least one `ComponentInstance` or relationship record
  links placed electrical and physical objects, and either one authored
  `RelationshipKind::BoardOnly`/`RelationshipKind::Mismatch` binding or the
  derived `UnresolvedMismatch` status is surfaced (not stored) rather than
  hidden
- rules/check proof: `CreateRule` or `EditRule` commits, ERC/DRC runs against
  the resolved model, and findings navigate to affected objects
- manufacturing proof: `CreateOutputJob` plus one live Gerber-like projection
  and one drill or mask/paste projection are generated from the committed
  model revision
- artifact proof: `GenerateArtifactSnapshot` records source `model_revision`,
  output-job revision, generator version, and validation/check state
- GUI bypass proof: the demonstrated GUI tools call `OperationBatch ->
  commit()` or proposal acceptance; no target shard is modified by a direct
  file write
- recovery proof: close/reopen after the slice restores source state,
  transaction journal, undo cursor, and visible provenance

## Open Owner Questions

1. What is the smallest board that should define the first manual editor proof:
   two components, a connector board, or a known reference circuit?
2. Which physical editing tools are mandatory before push-and-shove routing
   exists?
3. Should project-local libraries be the first library model, with global
   libraries deferred?
4. Which manufacturing projections are mandatory in the first proof: Gerber,
   NC drill, soldermask, paste, BOM/PnP, or assembly drawing?
5. What level of ERC/DRC completeness is enough to call the first manual
   workflow honest rather than cosmetic?
6. Which direct GUI edits must become proposals instead of direct commits:
   relationship-state changes, destructive deletes, imported-geometry repair,
   standards deviations, or batch edits?
7. Which existing GUI/editor helpers are accepted as temporary migration
   exceptions while operation families are moved behind `commit()`?
