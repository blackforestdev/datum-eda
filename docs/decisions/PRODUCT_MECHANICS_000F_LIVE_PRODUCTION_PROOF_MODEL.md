# Product Mechanics Decision 000F: Live Production Proof Model

Status: draft hypothesis + how/mechanism woven 2026-06-18; identity,
ComponentInstance, journal, and M7 sequencing ratified; remaining forks open.
Date: 2026-06-19

Driven by:
- `docs/DATUM_PRODUCT_MECHANICS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_DOCUMENTATION_GOALS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_REVIEW_AGENDA.md`
- `docs/decisions/PRODUCT_MECHANICS_000B_VIEW_COMPOSITION_AND_LIVE_PRODUCTION.md`
- `docs/decisions/PRODUCT_MECHANICS_000D_STORAGE_AND_VERSIONING_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_000E_MULTI_SHEET_MULTI_BOARD_MODEL.md`
- project-owner need to prove live production projections and panelization
  before implementation depends on them

## Purpose

Define the first concrete proof path for Datum's live production model.

This document does not specify the final manufacturing subsystem. It defines
the minimum behavior needed to validate that Gerber, NC drill, soldermask,
paste, BOM, pick-and-place, assembly, and panelization can behave as live
projections over one `DesignModel` without becoming separate source
authorities.

## Product Intent

Datum should let users inspect production truth while designing, not only
after a final export step.

The product goal is:

> Manufacturing outputs are live, checkable projections. Exported files are
> revisioned snapshots of those projections.

Panelization must also stay in the manufacturing domain. A panel is production
intent for building, separating, inspecting, assembling, or documenting one or
more board instances. It is not the editable source board unless the user
explicitly applies a board-level design change.

## Decision

Datum shall treat the first production proof as a bounded manufacturing
projection problem.

The minimum proof path shall include live projections for:
- Gerber layer output
- NC drill output
- soldermask output
- paste output
- BOM output
- pick-and-place output
- assembly drawing or assembly projection
- panel output

Each live projection must be derived from resolved `DesignModel` state,
manufacturing-plan state, output-job settings, and variant state as applicable.
Each exported artifact must be a snapshot of the same projection context at a
known revision.

Required rule:

> A production projection may expose manufacturing geometry that does not exist
> in the source board, but it must identify that geometry as manufacturing
> state and must not silently write it back into the board.

## User-Visible Behavior

Users should be able to open production views beside editable source views.

Required first behaviors:
- open a physical board layout and live Gerber projection from the same model
- open NC drill beside layout and cross-highlight drill hits, slots, and source
  pads or holes
- open soldermask and paste projections and inspect aperture geometry separate
  from copper geometry
- open BOM and PnP projections for a selected board, variant, and assembly
  side
- open an assembly projection that shows component placements, designators,
  orientation, side, variant inclusion, fiducials, and assembly notes
- open a panel projection that references board instances and shows
  panel-specific features
- export board artifacts and panel artifacts from explicit output jobs
- compare the current live projection to the last exported artifact snapshot

The user should see whether a projection is current, stale, invalid, or not yet
generated. Staleness is computed, not guessed: the served live tier is keyed by
the resolved `model_revision` and output-job revision, so a projection is stale
exactly when its key no longer matches the resolved model. If a projection
differs from an exported artifact, Datum reports which of the eight equivalence
causes applies — `ProjectionGeneration`, `ExportGeneration`, `StaleData`,
`ChangedModelRevision`, `ChangedOutputJob`, `ChangedManufacturingPlan`,
`GeneratorVersion`, or `Unsupported` (see Projection And Export Equivalence
Checks).

## Manual Workflow Requirements

AI must not be required for production review or panel setup.

The first manual workflow must allow a user to:
- choose a board, manufacturing plan, output job, and assembly variant
- open and refresh live Gerber, drill, mask, paste, BOM, PnP, assembly, and
  panel projections
- inspect source-to-projection relationships by selection and cross-highlight
- configure basic output-job settings for layer mapping, polarity, drill
  classes, mask and paste behavior, BOM columns, PnP columns, units, origins,
  side filters, and assembly drawing options
- create a simple repeated-board panel from an existing board without editing
  the board outline
- add rails, tabs, mouse bites, V-score lines, fiducials, tooling holes,
  coupons, labels, and panel notes as panel/manufacturing objects
- run production checks and review findings
- export artifacts and inspect artifact metadata

Manual controls may be simple in the first proof. They must still exercise the
real model boundaries.

## Optional AI And Tooling Behavior

Agents, scripts, CLI tools, and optional assistants may help with production
review, but they must use the same primitives as manual workflows.

Allowed AI/tooling behavior:
- query live production projections
- compare board layout to Gerber, drill, mask, and paste projections
- compare BOM/PnP/assembly projections to placed components and variants
- propose output-job settings
- propose panel strategies and panel-rule values
- propose manufacturing-plan edits for rails, tabs, mouse bites, V-scores,
  fiducials, tooling holes, coupons, labels, and notes
- explain projection/export differences
- generate production review reports

Required restrictions:
- agents must not silently change board geometry to make panelization work
- agents must not silently treat panel rails, tabs, coupons, or panel labels as
  board source geometry
- agents must not treat exported Gerber, drill, BOM, PnP, or assembly files as
  source authority unless the user explicitly imports them as production
  evidence or reverse-engineering input
- agent-originated manufacturing changes must be proposals unless the user has
  configured a safe automation policy

## Core Primitives

### Board

The editable product PCB source implementation.

Owns:
- board outline
- stackup
- component placements
- pads
- tracks
- vias
- zones
- keepouts
- board-level text and dimensions
- board-level rules and overrides

A board may be exported directly. A board may also be instantiated into one or
more panels.

### BoardInstance

A manufacturing reference to a board revision placed into a panel.

Owns:
- referenced board ID and board revision
- transform in panel coordinates
- side or flip state where applicable
- instance designator or panel position label
- variant or assembly context if the instance is variant-specific
- source-to-panel mapping metadata

A board instance does not duplicate editable board truth. It is an instance of
a board in a production context.

### Panel

Manufacturing-domain state for producing one or more board instances as a
single production unit.

Owns:
- panel outline or extents
- board instances
- rails
- tabs
- mouse bites
- V-score lines
- fiducials
- tooling holes
- coupons
- labels
- route paths
- panel notes
- panel rules and process assumptions

A panel is source state inside a manufacturing plan. It is not a board, and it
must not be serialized as if it were a second editable board layout.

### Rails

Panel material added for handling, clamping, conveying, assembly support, or
process requirements.

Rails belong to the panel. They may affect panel Gerbers, drills, drawings,
and assembly outputs, but they do not alter the source board outline.

### Tabs

Breakaway connections between board instances, rails, or other panel material.

Tabs belong to the panel. They may include routed geometry, keepout clearance,
mouse-bite drill patterns, or process notes.

### Mouse Bites

Drill patterns that weaken a tab for depanelization.

Mouse bites belong to tabs or panel breakaway features. Their drill hits appear
in panel NC drill output and panel fabrication drawings, not in board-only
drill output unless the board itself explicitly owns those holes.

### V-Scores

Scoring lines used for depanelization.

V-scores belong to the panel and must carry process parameters such as side,
depth, clearance, and allowed crossing rules where applicable. V-score
geometry appears in panel manufacturing outputs and notes.

### Fiducials

Optical registration marks for assembly or fabrication.

Datum should distinguish:
- board fiducials owned by the board
- local fiducials owned by a board or instance context
- global panel fiducials owned by the panel

Panel fiducials must not be written into the board unless explicitly converted
to board-level fiducials by the user.

### Tooling Holes

Mechanical holes used for fabrication, assembly, fixtures, or registration.

Panel tooling holes belong to the panel. Board tooling holes belong to the
board. Output projections must make the ownership clear because the same
physical hole shape can have different production meaning.

### Coupons

Manufacturing test or process-control features added to the panel.

Examples:
- impedance coupons
- solderability coupons
- plating coupons
- registration coupons
- process witness coupons

Coupons belong to manufacturing state and may depend on stackup, impedance
rules, copper classes, or fab requirements. They do not become source board
copper unless explicitly authored as board geometry.

### Labels

Human-readable or machine-readable markings used for manufacturing,
traceability, assembly, or inspection.

Datum should distinguish:
- board labels owned by the board
- panel labels owned by the panel
- artifact/package labels owned by output or release metadata

### Panel Notes

Manufacturing notes that apply to the panel context.

Panel notes may describe depanelization, array count, board orientation,
process assumptions, fab/CM instructions, coupon purpose, rail handling,
fiducial requirements, or artifact package contents.

Panel notes do not replace board fabrication notes unless intentionally linked
or promoted.

### ManufacturingPlan

Production intent for a board, panel, revision, variant, fab, or CM context.

Owns:
- target board or panel
- production process assumptions
- panel strategy
- assembly variant
- stencil assumptions
- production checks
- sign-off expectations

### OutputJob

An artifact generation recipe.

Owns:
- target context: board or panel
- layer and file mapping
- units and precision
- coordinate origin
- Gerber settings
- NC drill settings
- mask and paste settings
- BOM and PnP settings
- assembly drawing settings
- validation requirements

Changing an output job changes the output-job revision, not board geometry.

### LiveProductionProjection

A current or stale generated view of manufacturing intent.

Subtypes required by the proof:
- `GerberLayerProjection`
- `NcDrillProjection`
- `SolderMaskProjection`
- `PasteProjection`
- `BomProjection`
- `PickAndPlaceProjection`
- `AssemblyProjection`
- `PanelProjection`

Live projection objects must carry enough provenance to map generated geometry,
rows, or annotations back to source model objects, manufacturing objects,
variant state, and output-job settings.

### Artifact

A generated snapshot from a live production projection.

Required identity fields:
- artifact ID
- artifact type
- source model revision
- source board or panel ID and revision
- source manufacturing-plan revision
- source output-job revision
- variant revision where applicable
- generator name and version
- generated timestamp
- validation status
- content hash or file path

Generated artifact files are not design authority. Artifact metadata may be
versioned for traceability.

## Board Versus Panel Artifact Identity

Board artifacts and panel artifacts must be distinct even when generated from
the same board revision.

Board artifact examples:
- board Gerbers
- board NC drill files
- board soldermask and paste outputs
- board fabrication drawing
- board BOM
- board PnP
- board assembly drawing

Panel artifact examples:
- panel Gerbers
- panel NC drill files
- panel soldermask and paste outputs
- panel fabrication drawing
- panel BOM or array-aware BOM where needed
- panel PnP or array-aware PnP where needed
- panel assembly drawing
- panel notes package

Required identity rules:
- board artifacts identify the board context
- panel artifacts identify the panel context
- panel artifacts identify all board instance references and transforms
- panel artifacts identify panel-only manufacturing objects
- board-only exports must not include panel rails, tabs, mouse bites,
  V-scores, panel fiducials, panel tooling holes, coupons, or panel labels
- panel exports may include board geometry through board instances plus
  panel-specific manufacturing geometry
- artifact comparisons must not compare board artifacts and panel artifacts as
  equivalent unless the comparison mode explicitly accounts for panel context

## Projection And Export Equivalence Checks

Live production projections are trustworthy only if export uses the same
generation semantics.

Required rule:

> A live production projection and its exported artifact must use the same
> generation core or pass deterministic equivalence checks.

### Generation Tiers And The Equivalence Oracle

Live production projections are trustworthy only because export and projection
are the same computation, not two computations that happen to agree. Datum
therefore defines exactly one generation core and three tiers over it. The cold
tier (T0) is the artifact exporter and the equivalence oracle: a full-board,
deterministically ordered pass that is the definition of correct output. The
live tier (T1) is a memoization of that core keyed by the resolved
`model_revision` and the output-job revision; opening a projection beside
layout serves the cache, and a committed edit recomputes only the projections
affected by the changed objects, leaving the rest served unchanged. Because
each projection is a pure function of its layer-filtered inputs at a model
revision, the served bytes are identical to a cold regeneration by
construction — equivalence is structural, not aspirational. A sub-region
incremental tier (T2) that recomputes only changed islands within a projection
is a deliberate future option; it is admitted only behind the cold oracle,
never as the served truth on its own.

The equivalence harness exists to defend the one place this identity can
break: an input that feeds the core but is absent from the cache key. The
harness therefore makes two assertions per supported projection — that the
live cache equals a cold regeneration (the cache-completeness invariant), and
that the exported artifact semantically equals that regeneration (the
export-fidelity invariant).

The `model_revision` that keys T1 is supplied for free by the storage model
(000D): it is the sha256 over the canonical sorted per-object `object_revision`
values plus the accepted-transaction tip, so any committed mutation that
touches a projection's inputs necessarily moves the key. The first live-CAM
equivalence subset is Gerber copper + Excellon drill on the datum-test
fixture; native un-filled zones project as a badged `Unfilled` placeholder
that emits no copper and raises a hard finding (not the accidental
solid-copper-boundary behaviour of today's path), driven by the new `Zone`
provenance field and the derived `ZoneFill{Filled|Unfilled|Stale|Unsupported}`
defined in 000D. Native copper-pour fill is out of scope for this first proof;
if later authorized, the polygon-boolean substrate is `i_overlay` (pure-Rust,
MIT) so no FFI is introduced.

Minimum equivalence checks:
- Gerber live geometry matches exported Gerber geometry for the same layer,
  polarity, aperture, units, precision, output job, and context
- NC drill live geometry matches exported drill hits, slots, plated state,
  non-plated state, tool table, units, precision, and context
- soldermask live apertures match exported soldermask apertures, expansions,
  clearances, polarity, and context
- paste live apertures match exported paste apertures, reductions, omissions,
  stencil side, and context
- BOM live rows match exported BOM rows for selected variant, side filters,
  DNI/DNP state, quantities, manufacturer data, and configured columns
- PnP live rows match exported PnP rows for selected variant, component side,
  coordinates, rotation convention, origin, units, designators, and package
  references
- assembly projection matches exported assembly drawing or package for
  component inclusion, side, orientation, designators, fiducials, notes, and
  variant state
- panel projection matches exported panel artifacts for board instance
  transforms, panel outline, rails, tabs, mouse bites, V-scores, fiducials,
  tooling holes, coupons, labels, route paths, and panel notes

Equivalence checks produce machine-readable findings. A failed equivalence
check must classify the mismatch as exactly one of the eight causes:

- `ProjectionGeneration` — the live tier (T1) produced wrong geometry
- `ExportGeneration` — the cold tier (T0) exporter produced wrong geometry
- `StaleData` — the served cache is keyed to a superseded `model_revision`
- `ChangedModelRevision` — source objects changed between projection and export
- `ChangedOutputJob` — the output-job revision changed
- `ChangedManufacturingPlan` — manufacturing-plan state changed
- `GeneratorVersion` — the two computations ran under different generator
  versions; this cause is load-bearing because the reconciled authority flip
  (000D) stands up a second from-shards emitter, so the cold oracle and a
  candidate emitter can disagree purely on generator version during the
  dual-write deprecation window
- `Unsupported` — the comparison semantics are not yet defined for this
  projection or artifact type

A `ProjectionGeneration` or `ExportGeneration` finding is a true defect in the
shared core. A `StaleData`, `ChangedModelRevision`, `ChangedOutputJob`, or
`ChangedManufacturingPlan` finding means a key moved as designed and the
artifact is simply out of date. A `GeneratorVersion` finding is expected and
tolerated only inside the dual-write window; outside it, it is a defect.

## Standards And Compliance Impact

Live production projections make standards and process checks visible before
artifact release.

Initial standards/process impacts:
- Gerber and drill generation must preserve explicit process meaning for
  polarity, apertures, plated drills, non-plated drills, slots, and routing
- soldermask and paste projections must expose aperture policy rather than
  hiding it in export settings
- BOM, PnP, and assembly projections must expose variant, side, coordinate,
  rotation, and inclusion assumptions
- panel projections must expose process constraints for rails, tab geometry,
  mouse-bite drill patterns, V-score clearances, fiducials, tooling holes,
  coupons, and depanelization notes
- deviations, waivers, and process assumptions must be model objects or
  manufacturing-plan state, not undocumented export behavior

Datum does not need to implement every IPC, fab, CM, or assembly standard in
the first proof. It must leave explicit places for standards basis, process
assumptions, check results, and waivers.

## Non-Goals

This decision does not require:
- full real-time regeneration for every mouse move
- every Gerber, drill, IPC-2581, ODB++, PDF, STEP, BOM, PnP, or assembly
  package feature in the first proof
- automatic CM-specific panel optimization
- built-in knowledge of every fab or assembler rule set
- replacing external CAM review tools
- forcing every project to use panels
- making panelization a board-editing feature
- importing exported artifacts as normal source truth
- solving release packaging, procurement, quoting, or PLM workflows
- finalizing storage serialization details

## Proof Gates

The consolidated proof plan is a single 10-gate set that lives in 000D; all
other product-mechanics decision docs cross-reference it rather than restating
gates. This document owns the manufacturing/live-CAM detail behind three of
those gates and shares the panelization gate, which was deduplicated from the
four near-identical copies that previously appeared across 000D, 000E, and
this doc.

The gates this document is responsible for are:

- `PG-LIVE-CAM-EQUIVALENCE` (gate 6) — the T0/T1 generation tiers, the two
  harness assertions (live==cold cache-completeness, export≈projection
  fidelity), and the eight-cause failure classification above. First subset:
  Gerber copper + Excellon drill on datum-test, with un-filled native zones
  badged and emitting no copper.
- `PG-PANELIZATION-ISOLATION` (gate 7, deduped) — panel state lives in
  manufacturing-plan/panel state, board instances reference board revisions
  and transforms, source board geometry is unchanged by panel edits, and
  board-only artifacts exclude panel-only geometry. The full pass criteria are
  stated once in 000D; the manufacturing-object inventory (rails, tabs, mouse
  bites, V-scores, fiducials, tooling holes, coupons, labels, panel notes) and
  the board-versus-panel artifact identity rules are defined in this document's
  Core Primitives and Board Versus Panel Artifact Identity sections.
- `PG-VARIANT-RESOLUTION` (gate 8, population-only) — BOM/PnP/assembly
  projections reflect the active variant overlay and the three-valued
  population state {Fitted, Unfitted (authored), NotApplicableForVariant
  (derived)} without mutating physical geometry, and switching the active
  variant swaps an overlay handle with zero base writes. Cross-domain
  population requires the `ComponentInstance` surrogate.
- `PG-AUTHORITY-FLIP+ARTIFACT-TRACEABILITY` (gate 9) — exported artifacts
  carry the full revision provenance (model, board/panel, manufacturing-plan,
  output-job, variant, generator version, validation state) so a stale
  snapshot can be detected and its differing revision named; this gate also
  guards the from-shards emitter's byte-identical-when-unmodified fidelity
  during the dual-write window, where `GeneratorVersion` findings are the
  expected, tolerated mismatch class.

Wiring of these gates into `scripts/run_drift_gates.sh` is gate 10
(`PG-HARNESS-WIRING`), owned in 000D alongside the consolidated plan.

## Resolved (no longer open)

These questions were answered by the reconciled model and the 2026-06-18
tactical defaults; they are recorded here so the prior open list stays
auditable.

- First production projection / first equivalence subset: Gerber copper +
  Excellon drill on the datum-test fixture (Slice 0), seeded board-first as an
  all-`BoardOnly` reverse-engineered seed. IPC-2581 and ODB++ are therefore
  deferred until Gerber/Excellon equivalence is stable; the eight-cause
  classifier marks them `Unsupported` until then. (Was: which projection
  first; should IPC-2581/ODB++ be in the first proof.)
- Unfilled-zone export policy: emit nothing plus a hard finding, driven by the
  new `Zone` provenance field and the derived `ZoneFill` state; the prior
  solid-copper-boundary behaviour was accidental and is not the contract.
- Polygon-boolean substrate (only if native pour fill is later authorized):
  `i_overlay` (pure-Rust, MIT), no FFI.

## Open Owner Questions

1. Should the first panel proof support only repeated-board arrays, or also
   multi-board panels?
2. Are V-scores required in the first panel proof, or can routed tabs and mouse
   bites prove the manufacturing-domain boundary first? (Tactical lean: tabs +
   mouse bites first; record any objection.)
3. Should board-level fiducials and panel-level fiducials share one primitive
   with different ownership, or be separate primitive types?
4. What minimum BOM/PnP column and coordinate conventions are required for the
   first equivalence proof?
5. Should Datum define generic panel rules first, or import named fab/CM rule
   presets during the proof?
6. Should generated artifact files be checked into source control for proof
   projects, or should only artifact metadata be tracked?
7. What level of deterministic comparison is required for PDF assembly
   drawings, where visual rendering may include tolerated metadata or
   formatting differences?
