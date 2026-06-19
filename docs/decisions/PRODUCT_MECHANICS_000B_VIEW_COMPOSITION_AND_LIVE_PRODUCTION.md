# Product Mechanics Decision 000B: View Composition And Live Production

Status: draft hypothesis + how/mechanism woven 2026-06-18; identity,
ComponentInstance, journal, and M7 sequencing ratified; remaining forks open.
Date: 2026-06-17

Driven by:
- `docs/decisions/PRODUCT_MECHANICS_000_UNIFIED_DESIGN_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_000C_UNIFIED_MODEL_FEASIBILITY.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_RESEARCH_SYNTHESIS.md`
- `docs/DATUM_PRODUCT_MECHANICS.md`
- project-owner product discussion on tabs, PiP/tiled views, live CAM
  visibility, and panelization as production-stage work

## Decision Scope

Define how Datum composes visual/tool surfaces over the unified
`DesignModel`, and how fabrication/assembly outputs behave during design.

This decision covers:
- tabs as view containers
- tiled, floating, and picture-in-picture layouts
- live manufacturing projections
- Gerber and NC drill viewing beside layout
- panelization as manufacturing projection
- output artifacts as versioned snapshots

## Product Intent

Datum should not force users into conventional fixed schematic/PCB/workbench
layouts. Users should be able to compose their workspace around the task.

Datum should also not treat fabrication and assembly outputs as end-of-process
export artifacts that the user sees only after layout is finished.

Required product rule:

> Tabs are composable view/tool containers over the design model. Manufacturing
> outputs are live projections of the design model, not after-the-fact export
> artifacts.

## Decision

Datum shall support a composable tab/workbench model.

Any projection or tool session may be opened as a tab. Tabs can be arranged as:
- focused main view
- vertical split
- horizontal split
- grid/tiled layout
- floating window
- picture-in-picture
- pinned sidecar
- overlay
- saved workbench layout

Tabs are not data authorities. Closing a tab closes a view/tool session, not a
schematic file, PCB file, manufacturing file, or design authority.

Manufacturing outputs shall be represented as live production projections.
Gerber, soldermask, paste, NC drill, assembly, BOM/PnP, panel, and output-job
views should be inspectable during design and layout, including beside the
editable layout view. All such projections are derived state over the single
authority — the in-memory `DesignModel` assembled by the engine-owned
`ProjectResolver` from segmented source shards. Shards are persistence
partitions, never authorities.

The identity, resolver, and journaled-commit substrate this decision relies on
is built additively and concurrently with M7; it does not close M7. The
authority flip and the from-shards KiCad emitter are deferred until M7 closes
and the emitter passes a byte-identical-when-unmodified fidelity gate on the
`datum-test` fixture, run as a dual-write deprecation window. Until that flip,
live CAM is proven against imported boards through the projection/equivalence
path, not by re-emitting the source.

## View Composition Primitives

### ViewTab

A live visual or tool session over the design model.

Examples:
- electrical projection tab
- physical PCB projection tab
- symbol tab
- footprint tab
- library tab
- rules/checks tab
- 3D tab
- simulation tab
- waveform tab
- field-solver tab
- stackup/impedance tab
- Gerber preview tab
- NC drill preview tab
- soldermask/paste preview tab
- BOM/PnP tab
- panel tab
- assembly drawing tab
- output job tab
- artifact preview tab
- terminal tab
- assistant tab
- documentation/notes tab
- log tab

### Pane

A container that holds one or more tabs.

### LayoutGraph

The current arrangement of panes and tabs.

Supports:
- focused
- split vertical
- split horizontal
- tiled/grid
- floating
- picture-in-picture
- pinned sidecar
- overlay

### WorkbenchProfile

A saved arrangement of tabs, panes, toolbars, inspectors, and overlays.

Examples:
- layout workbench
- schematic/electrical workbench
- manufacturing review workbench
- simulation workbench
- AI/debug workbench
- library creation workbench

### PinnedContext

A secondary tab or overlay linked to active selection.

Examples:
- schematic PiP pinned while routing physical layout
- live Gerber preview pinned beside layout
- DRC findings sidecar
- 3D preview floating over layout
- waveform tab pinned while editing simulation source

### Viewport

A saved visual framing of a projection for output, review, or documentation.

Viewports may be opened in tabs, but they are not the same as tabs. A tab is a
live session; a viewport is a saved framing/presentation definition.

## Live Production Model

Datum should treat production data as first-class model projections.

### ManufacturingProjection

A generated, inspectable, checkable projection of manufacturing intent.

Subtypes:
- `GerberLayerProjection`
- `NcDrillProjection`
- `SolderMaskProjection`
- `PasteProjection`
- `FabricationDrawingProjection`
- `AssemblyDrawingProjection`
- `BomProjection`
- `PickAndPlaceProjection`
- `PanelProjection`
- `OutputJobProjection`

Manufacturing projections are live views over the resolved `DesignModel`,
never a second authority. They are derived state, and the mechanism that makes
them both honest and live is a single generation core fronted by three tiers:

- T0 — cold full-regen. The one authoritative generator path. It is *both* the
  exporter and the equivalence oracle: an artifact and the cache must agree
  with what T0 produces from a cold model. Nothing ships a projection that T0
  cannot reproduce.
- T1 — the live tier. A revision/hash-keyed memoized cache over T0 output,
  keyed by `model_revision` (a sha256 over the canonical sorted per-object
  `object_revision` values plus the accepted-transaction tip). A transaction
  changes `model_revision`; the dependent projection key misses; the tier
  re-derives. This is how "invalidated by transactions" actually works — there
  is no ad-hoc dirty flag, only a content key that stops matching.
- T2 — sub-region incremental regeneration. Deferred behind the T0 oracle;
  not built for the first proof.

Because every projection key derives from `model_revision`, a projection can
always be revalidated against a cold T0 run and against an exported artifact
snapshot, which is exactly what the live-CAM equivalence gate checks.

### Artifact

A generated snapshot from a projection at a specific model revision.

Examples:
- Gerber file
- NC drill file
- IPC-2581 package
- ODB++ package
- BOM
- PnP
- PDF drawing
- STEP file
- assembly package
- panel fabrication package

Required artifact metadata:
- source `model_revision` (the sha256 content hash described above, so the
  snapshot is provably tied to an exact resolved model state)
- source board/panel/variant/output job
- generation settings, including the T0 generator version (a `GeneratorVersion`
  difference is one of the classified equivalence-divergence causes, so it must
  be recorded on the artifact)
- timestamp
- tool/version
- validation/check status

## Gerber And NC Drill As Live Views

Gerber and NC drill should be viewable in real time beside editable physical
layout.

Reasons:
- many manufacturing errors are invisible in normal layout views
- soldermask, paste, aperture polarity, drill hits, slots, board outline, and
  layer polarity have production semantics different from editable source
  objects
- users should see the fabrication truth before final export

Required behavior:
- layout edits invalidate affected CAM projections
- live Gerber/NC drill tabs show generated production geometry
- source objects and generated CAM geometry cross-highlight
- DRC/process findings link to both source object and manufacturing projection
- exported artifacts are snapshots of the projection at a model revision

### Fill State And The Honesty Rule

A live copper projection is only as honest as the geometry it draws. Today
Datum carries a zone as its authored boundary outline; for imported boards the
loader additionally carries the fabricator-accurate fill the source tool
already computed. A native zone that has never been filled has no such
geometry, and drawing its boundary as solid copper would present a rough
approximation as production truth — precisely the failure this decision exists
to prevent. Datum therefore separates a zone's authored intent from its
derived fill. `Zone.polygon` reverts to the authored boundary only; a new
`Zone` provenance field — the load-bearing input missing today — records where
that boundary came from, and every zone resolves to a derived
`ZoneFill{state, islands}` carrying the fill projection. The state is explicit:
filled, when real island geometry exists (an imported fill today, a natively
computed fill once the fill engine is authorized); unfilled, when only a
boundary exists; stale, when the inputs that produced a fill have changed; or
unsupported, when the source could not be accounted for.

Only a filled state contributes solid copper to a projection or an export. An
unfilled zone appears in the live view as a clearly badged placeholder over its
boundary and contributes no copper to the artifact, so a designer always sees
the difference between intended pour and fabricated pour. The unfilled-zone
export policy is to emit nothing plus a hard finding — not the
solid-copper-boundary fallback that is today's accidental behaviour. This keeps
the first live-CAM proof honest on the boards where fill genuinely exists —
imported designs — while reserving a clean seam for a native fill engine to
populate the same filled state later without reshaping the projection or the
exporter. Native copper-pour fill is therefore explicitly out of scope for the
first live-CAM proof; if and when it is authorized, the polygon-boolean
substrate is `i_overlay` (pure-Rust, MIT) so no FFI creep enters the engine.

Because of this separation, live CAM is honest on imported boards from the
first cut but is *not* uniformly available for native designs today: a native
board with unfilled zones produces a faithful projection (boundaries badged,
no phantom copper), not a complete fabrication artifact. That is the intended
posture, not a gap to paper over.

Examples of issues this should expose early:
- missing solder-mask expansion
- paste apertures inherited from copper
- incorrect plated/non-plated drill distinction
- missing or malformed slots
- silkscreen over exposed pads
- bad board outline
- wrong layer polarity or output job setting
- BOM/PnP mismatch

## Panelization As Manufacturing Projection

Panelization belongs to the manufacturing/production phase, not the source
board layout.

Required rule:

> Panelization must not mutate source board geometry unless the user explicitly
> requests a board-level change.

### Board

The editable product PCB.

Owns:
- outline
- stackup
- component placements
- pads
- tracks
- vias
- zones
- board-level text/dimensions
- board-level rules

### ManufacturingPlan

Production intent for a board/revision/variant.

May include:
- fab/CM target
- output jobs
- panel strategy
- assembly variant (a sparse authored variant overlay shard keyed by stable
  object identity, with zero base writes; population is three-valued —
  `Fitted`/`Unfitted` authored, `NotApplicableForVariant` derived and never
  stored — and cross-domain population resolution relies on the
  `ComponentInstance` surrogate that joins schematic symbol to board package)
- stencil requirements
- fabrication notes
- validation requirements

### PanelProjection

A manufacturing-stage arrangement of board instances and panel-specific
features.

Owns:
- board instance references and transforms
- rails
- breakaway tabs
- mouse bites
- V-score lines
- global fiducials
- local fiducials if needed
- tooling holes
- coupons
- route paths
- panel labels
- panel-specific manufacturing notes

The isolation rule is enforced by construction, not convention: a
`PanelProjection` is derived state over the resolved `DesignModel`, and it owns
only *references* to board instances plus their transforms. It carries no
authored board geometry of its own, so it has nothing it could write back into
the source board's shards. The single `commit()` path is the only way any
shard byte changes, and a panel operation produces no board-geometry
`OperationBatch`; this is what the shard-diff-isolation and
panelization-isolation proof gates assert.

### PanelRule

Process constraints for panel production.

Examples:
- board spacing
- rail width
- tab spacing
- mouse-bite drill size/count/spacing
- V-score clearance
- fiducial size/clearance
- tooling hole size/clearance
- coupon requirements
- stencil panel constraints

## Designer-Controlled And CM-Controlled Panelization

Datum should support several production models:

- board-only export
- designer-controlled panelization
- CM-template panelization
- fab-house preset panelization
- assembly panel export
- stencil panel export
- multi-board panel
- repeated-board array panel

If a CM modifies panelization downstream, Datum should be able to capture that
as a manufacturing-plan revision or imported production artifact rather than
silently changing the source board. When such a downstream change is folded
back in, it enters as an authored `Relationship` (`ReverseEngineered`) over a
stable surrogate identity, not as a mutation of the original authored geometry
— so provenance stays explicit and the source board remains the authority.

## UI Consequences

The tab model should make live production review natural:

- layout tab focused, Gerber PiP
- layout tab left, NC drill tab right
- layout tab top, DRC/checks sidecar bottom
- panel tab focused, board source PiP
- manufacturing workbench with output job, Gerber, drill, BOM/PnP, and panel
  tabs tiled
- 3D tab floating while editing component placement
- simulation waveform tab pinned while editing electrical projection

This is not cosmetic. It is a quality-control mechanism.

## AI And Tooling Consequences

Agents should be able to:
- query live manufacturing projections
- compare source layout to Gerber/NC drill output
- explain why a CAM projection differs from source intent
- propose output-job changes
- propose panelization settings
- inspect fabrication/assembly risk
- generate manufacturing review reports

Agents must not:
- silently change board geometry to satisfy panelization
- silently accept CM panel changes as board truth
- treat exported artifacts as source authority unless explicitly imported as
  reverse-engineering or production evidence

## Standards And Compliance Impact

Live production projections should make standards and process checks visible
early.

Examples:
- IPC/process aperture checks visible in soldermask/paste projections
- drill and slot checks visible in NC drill projection
- panel rules visible in panel projection
- assembly fiducial and tooling requirements visible in manufacturing
  projection
- output job completeness visible before artifact generation

Standards deviations and waivers must remain explicit model objects, not
hidden export settings.

## Explicit Non-Goals

This decision does not require:
- implementing all tab kinds immediately
- making every projection real-time from the first release
- replacing external CAM viewers entirely
- preventing users from exporting board-only artifacts
- forcing every project to use panelization
- assuming Datum knows every CM's process rules by default
- making tabs the authority model

## Proof Gates

The consolidated 10-gate proof plan is owned by decision 000D; this doc
cross-references the gates it is accountable for and does not redefine them.

The gates load-bearing for view composition and live production are:

- PG-LIVE-CAM-EQUIVALENCE (gate 6) — the live T1 projection must equal a cold
  T0 regeneration (cache completeness) and the exported artifact must match the
  projection (fidelity), with any divergence classified by the full
  eight-cause set (including `GeneratorVersion`). The first equivalence subset
  is Gerber copper plus Excellon drill on the `datum-test` fixture. The honesty
  rule above is part of this gate: an unfilled native zone must project a badged
  placeholder and export nothing plus a hard finding, never solid copper.
- PG-SHARD-DIFF-ISOLATION (gate 4) — a view/projection operation produces no
  source-shard byte changes.
- PG-PANELIZATION-ISOLATION (gate 7, deduped) — a panel operation references
  board instances and mutates no source board geometry.
- PG-VARIANT-RESOLUTION (gate 8, population-only) — assembly-variant overlays
  resolve population three-valued without base writes.

These run via PG-HARNESS-WIRING (gate 10) into `run_drift_gates.sh`.

## First Proof Slice

The first proof slice should show:
- physical layout tab and live Gerber projection tab from the same
  `DesignModel`
- live NC drill or soldermask/paste projection beside layout
- cross-selection between source object and generated manufacturing geometry
- output artifact snapshot tied to `model_revision` and output job
- panel projection referencing a board instance without mutating board geometry
- an imported board (`datum-test`, which has 32 routed segments, 11 footprints,
  and a schematic) as the live-CAM subject, since real fill exists there;
  native unfilled zones are shown as badged placeholders, not solid copper

## Open Owner Questions

The following were resolved by the 2026-06-18 reconciliation and are recorded
as decided, not open:

- The first live-CAM equivalence subset is Gerber copper plus Excellon drill on
  the `datum-test` fixture (was: "which CAM projections in the first proof").
- Native copper-pour fill is out of scope for the first proof; native unfilled
  zones render a badged placeholder and emit no copper plus a hard finding. The
  reserved polygon-boolean substrate, if later authorized, is `i_overlay`.
- Mandatory first-snapshot artifact metadata is the `model_revision` content
  hash plus the T0 `GeneratorVersion`, alongside source job/variant and
  check status (was: "what artifact metadata is mandatory").

Genuinely open forks:

1. Should panelization be available before full board editing, or only after
   board editing is stable?
2. Which tab layout modes are required first: split, PiP, floating, pinned
   sidecar, or saved workbench?
3. Should Datum ship with fab/CM panelization presets, or start with generic
   rules only?
4. Beyond the Gerber-copper + Excellon-drill subset, in what order should
   paste, soldermask, and assembly projections join the equivalence oracle?
