# Datum GUI Product Specification

> **Status**: Governed active planning spec.
> **Authority**: Realization of
> `docs/decisions/PRODUCT_MECHANICS_019_GUI_PRODUCT_MODEL.md`.
> **Depends on**:
> decisions 001, 002, 004, 005, 006, 007, 014, 015, and 016, plus
> `docs/contracts/UI_LAYOUT_SYSTEM_CONTRACT.md`.
> **Scope**: Human-driven Datum desktop GUI product model and recovery plan.

## Purpose

Define the GUI Datum must become before more GUI implementation expands. This
spec replaces ad hoc M7 review-shell momentum with one product model that can be
implemented, tested, and reviewed by the human operator.

This document is intentionally one spec. It does not create separate schematic,
PCB, library, terminal, or manufacturing GUI specs. Narrower contracts can be
created later only when this product model is accepted and a bounded
implementation slice needs one.

## Product Standard

Datum's GUI is complete only when a human can operate the core EDA workflow from
the application itself:

- open or create a project;
- inspect project/library/schematic/PCB/manufacturing state;
- load/import/export through visible application commands;
- select objects and inspect properties;
- invoke tools from menus, toolbar/tool palettes, shortcuts, and command
  palette;
- commit direct manual edits through the engine operation/journal path;
- run checks and inspect findings;
- view manufacturing outputs and validation;
- use a real terminal for shell workflows without the terminal substituting for
  the GUI editor.

Backend capability is not product completion unless the GUI exposes it in a
human-operable way or the spec explicitly marks it as headless-only.

## Reference Inputs

Internal research inputs to mine before any external re-research:

- `docs/ALTIUM_LESSONS.md`
- `docs/HORIZON_ANALYSIS.md`
- `docs/EAGLE_BLUEPRINT.md`
- `docs/POOL_ARCHITECTURE.md`
- `docs/gui/REFERENCE_STUDY.md`
- `docs/gui/FOUNDATION.md`
- `docs/gui/WORKSPACE_MODEL.md`
- `docs/gui/INTERACTION_MODEL.md`
- `docs/gui/VISUAL_LANGUAGE.md`
- `docs/gui/TECHNICAL_PRINCIPLES.md`
- `docs/gui/M7_BOARD_REVIEW_FIDELITY_GAP.md`
- `docs/gui/M7_IMPORTED_BOARD_FIDELITY_PLAN.md` as historical fidelity
  analysis only, not active roadmap authority.
- `docs/gui/DATUM_GUI_VISUAL_REGRESSION_HARNESS.md` and the M7 render/text
  technical notes as subordinate renderer references only, not product-shell or
  roadmap authority.

External reference review should be used to verify current professional UX
expectations, not to restart research from zero.

## Schematic and Library Surface Specification Successor

The `GUI-SURFACE-SPECS` Frontier item turns the product model into buildable
schematic-editor and library-browser surface contracts. It is planning work
only: it does not authorize implementation, dependencies, or a GUI mutation
path.

<!-- REQ:GUI-SURFACE-SPECS:SURFACE-S01 -->
SURFACE-S01 inventories current schematic and library requirements, prototypes,
research, code seams, object authority, and unresolved product questions before
either surface contract is refined.

<!-- REQ:GUI-SURFACE-SPECS:SURFACE-S02 -->
SURFACE-S02 produces a buildable schematic-editor visual and interaction
contract, including document/hierarchy behavior, viewport tooling inheritance,
read-only versus authoring boundaries, accessibility, and honest conformance
dispositions.

<!-- REQ:GUI-SURFACE-SPECS:SURFACE-S03 -->
SURFACE-S03 produces a buildable library-browser and library-object surface
contract across symbols, units, parts, packages, footprints, padstacks,
pin-pad maps, models, metadata, and validation, with honest conformance
dispositions.

<!-- REQ:GUI-SURFACE-SPECS:SURFACE-S04 -->
SURFACE-S04 reconciles the two surface contracts with the shared shell,
selection/inspection tooling, manual-first workflow, engine authority, visual
prototype inventory, specification governance, and roadmap dependencies.

<!-- REQ:GUI-SURFACE-SPECS:SURFACE-S05 -->
<!-- OWNER:GUI-SURFACE-SPECS:SURFACE-S05:SURFACE-S05 -->
SURFACE-S05 records the owner's visual and product disposition, resolves or
explicitly defers every remaining owner question, and selects a bounded
implementation successor without granting execution authority implicitly.

## Shell Contract

The recovered GUI shell must include:

- top application menu;
- primary toolbar or tool strip;
- document tabs or document switcher;
- central editor viewport area;
- project/navigation browser;
- inspector/properties panel;
- layers/appearance/filter panel;
- checks/issues panel;
- manufacturing/output panel;
- bottom or side terminal area with real PTY sessions;
- Application Status Bar candidate (retention and contents deferred under
  `DATUM_APPLICATION_STATUS_BAR_GUIDANCE.md`); selection, coordinates, and active
  tool MUST remain legible near their engaged pane regardless, while model/check/
  background state needs a durable global route whether or not the strip remains.

Minimum top menu:

- File: New Project, Open Project, Import, Save, Save As, Export, Close.
- Edit: Undo, Redo, Cut, Copy, Paste, Delete; Preferences submenu with Global
  Preferences and Project Preferences. Project Preferences is disabled when no
  Project is open.
- View: Fit, Zoom, Pan, Layer Visibility, Panels, Reset Workspace.
- Place: symbol/component/text/label/wire/track/via where enabled by the active
  editor.
- Route/PCB Tools: track, via, zone, move, align/distribute, measure.
- Project: validate, resolve/debug. Settings use the Edit > Preferences family;
  they never appear in the Project Navigator.
- Checks: run ERC/DRC/check profiles, findings, waivers/deviations.
- Manufacturing: output jobs, artifacts, generate, validate, compare, export.
- Window: documents, terminal sessions, workspace layout.
- Help: about, diagnostics, command reference.

Menu commands may be disabled before their editor exists, but their placement
defines the product information architecture. Each menu command's real backing
mechanism (verb id / native-write builder / CLI), its live/blocked/not-built
status, and the write-path plumbing it depends on are catalogued in
`docs/gui/DATUM_GUI_MENU_BINDINGS.md`.

## Document Model

The GUI hosts documents and views.

Document types:

- Project overview.
- Library browser.
- Library object editor/preview.
- Schematic Design.
- PCB board.
- Rules/check report.
- Manufacturing outputs/artifacts.
- Terminal session.

Views are presentations of documents, not separate state authorities.

Examples:

- A PCB board document may have layout, layer-focused, review, and future 3D
  views.
- A schematic document may have source-definition and hierarchy/occurrence views.
- A library part may show symbol, package, footprint, padstack, pin-pad map,
  metadata, and validation panes.

The application must always make the active project, active document, active
view, selection, active tool, and dirty/check status legible.

## Editor Authority

GUI editors must commit through the canonical engine mutation model.

Direct manual GUI edits are allowed only when they are:

- local and visible;
- directly caused by a human gesture or property edit;
- validated by engine write builders;
- represented as typed operations;
- journaled as one undoable transaction;
- refreshed through resolver state after commit.

GUI edits must become proposals when they are:

- AI/tool generated;
- destructive or broad batch changes;
- cross-domain ECOs;
- standards deviations or check repairs;
- import repair/migration edits;
- hidden changes not fully visible at the point of action.

Forbidden:

- GUI tools that mutate by writing source shards directly.
- GUI tools whose primary implementation is "type this CLI command into the
  terminal."
- GUI-only undo stacks for project state.
- terminal or agent paths with private mutation authority.

## Terminal Model

The terminal is a real PTY-backed shell session surface.

Required behavior:

- launch the user's shell in project root;
- support multiple terminal tabs/sessions;
- pass keyboard input to the PTY when focused;
- support copy/paste, resize, scrollback, lifecycle status, restart/close;
- inject Datum context through environment variables and context files;
- allow users to run CLI tools, scripts, git, and terminal agents normally.

The terminal is not:

- a command echo log;
- an activity log dressed as a terminal;
- the internal implementation of GUI editor actions;
- a separate design authority.

Activity summaries and provenance logs may exist, but they should be labeled as
logs/status, not confused with shell input/output.

## Agent Model

Agents are terminal-launched by default. The GUI may offer a terminal launcher
for installed agents, but no separate AGENTS dock is part of the product model.

Allowed first-slice behavior:

- a menu/toolbar command that opens a new terminal session prefilled or ready to
  run a configured agent command;
- context files/env vars for agents;
- agent output visible in the terminal.

Not allowed without a future decision:

- separate assistant/editor surface with its own state;
- assistant mutation controls that bypass proposal/operation review;
- an AGENTS pane that looks like a first-class editor lane.

## Board Rendering Fidelity

Board rendering is the first GUI recovery acceptance gate.

The GUI must render the `datum-test` board fixture with recognizable EDA object
classes:

- board outline and substrate/field;
- footprints/packages as grouped physical objects;
- pads with shape, size, layer, drill/plating where supported;
- tracks as copper, not generic linework;
- vias as vias, not generic dots;
- zones/pours with distinct filled/unfilled/stale states;
- silkscreen and reference/value text where supported;
- layer identity and visibility;
- selection/highlight/dim states;
- unrouted/ratsnest state distinct from copper;
- proposal/diagnostic overlays distinct from authored geometry.

Acceptance requires a human-reviewed `datum-test` visual fixture and
screenshot-golden regression. A textual unit test cannot prove board
readability.

Unsupported or lossy geometry must be surfaced explicitly as unsupported,
degraded, or hidden, not silently rendered as wrong board meaning.

## Library GUI Target

The GUI must expose native governed library data before complete native
schematic/PCB authoring can be product-real.

Target surfaces:

- library browser by pool and object kind;
- symbol preview and pin/unit inspection;
- package/footprint preview;
- padstack preview;
- part metadata and lifecycle;
- pin-pad map inspection;
- binding status and validation findings;
- provenance/standards basis where present.

First implementation can be read/preview-focused, but it must not require users
to know object UUIDs or CLI incantations for basic library inspection.

## Schematic GUI Target

The schematic editor target is:

- continuous Design rendering from native schematic state;
- symbols, pins, wires, labels, ports, buses, bus entries, junctions,
  no-connects, hierarchy markers;
- selection and inspector;
- place/move/delete/edit tools;
- net highlighting and ERC finding navigation;
- commit through engine operations.

The historical Phase 1 recovery slice below describes the earlier board-shell
recovery scope. It does not add a board-fidelity prerequisite to the selected
native library/schematic surface planning or change its Frontier authority.

### Schematic Surface Contract — SURFACE-S02 Draft

This is the buildable planning candidate for the existing schematic target.
Inherited numbered decisions remain controlling; proposed presentation choices
await SURFACE-S04 reconciliation, independent review and SURFACE-S05 owner
disposition. This draft grants no implementation, source adoption or seed-library
qualification. Source review is recorded in
`docs/reviews/workflow-delivery-rollout/surface-route-preflight.json`.

**Document and pane composition.** Schematic Design is continuous model space;
Sheet denotes a Publish page in the later Publish workspace brief. Internal
source-shard and hierarchy record names are unchanged. The focused pane identifies
the native document, source definition and occurrence context separately. A
repeated instance must never be silently treated as its source definition or as
another occurrence. Navigator inspection and opening a view are view actions;
authoring requires the current qualified source and writer authority.

The schematic persona inherits PM019/PM021's single window, recursive pane tree,
project navigation, focused-pane Inspector and context controls, real terminal,
and output-only pane Console. Focus determines tools, menus and Inspector data;
layout, camera, Zoom and Stage remain view state. A second projection refreshes
from the same resolved model after a commit. Cross-domain correspondence is
qualified by model identity; names or matching member lists do not establish it.
No detached window, global editor mode or separate schematic undo stack is added.

**Visual composition.** The central pane renders symbols and pins, scalar wires,
labels, ports, buses and entries, junctions, no-connects and hierarchy objects
from native projection data. Selection, preview and findings remain distinct
layers over that projection. The shared camera, grid, snap, hit testing,
selection, measure and Inspector services supply the behavior; the schematic
persona configures their verbs and object presentation. Existing rendering and
token authorities supply stroke, glyph, typography, color and sizing values.
This draft does not derive production values from HTML mockup pixels or
silently restyle imported symbols.

Inherited viewport dispatch is explicit: pointer preview, wheel and Space-primary
pan address the pointer pane; keyboard/menu actions address the focused pane.
The initial click focuses and dispatches together. Escape cancels an active
gesture before leaving Stage or clearing idle selection; capture loss suppresses
an accidental commit. Empty canvas preserves selection. A placed Symbol pin
selects its owning Symbol, not an independently editable pin. Electrical click
granularity and modifier behavior use the shared tooling contract unchanged.
Tool prompts and refusals use PM033's accessible output feedback; persistent
degradation uses the owning banner/Notices surface and ERC findings use the
finding surface. Console is never an input field or a replacement findings log.

The following is the proposed frame coverage for implementation review. It
defines observable content; exact hierarchy-opening choreography and placement
of readiness feedback require the owner disposition below.

| Frame | Content that must be legible | Visual evidence disposition |
|---|---|---|
| Native Design, unselected | Document and occurrence context, active tool, grid/camera context, dirty/check state | Existing schematic-editor study is historical; candidate reconciliation pending |
| Selected object | Shared selection presentation and object-specific Inspector; merely related objects retain baseline appearance | Inherit existing shared selection contract; complete schematic coverage pending |
| Symbol picker / authoring preview | Named native object, binding availability, placement preview and cancel affordance | Bounded ordinary-app evidence exists; full browser is SURFACE-S03 |
| Refused / pending / stale | Textual reason or pending result without a false success indication | Candidate frame reconciliation and ordinary-app proof pending |
| Tiled, narrow, Zoom and Stage | Focus remains visible; consequential controls and context remain reachable | Shared pane authority inherited; schematic frame/HUMAN qualification pending |
| Net / ERC navigation | Finding, affected qualified objects, currentness and lawful pending/mismatch | Full schematic visual and workflow proof pending |

**Readiness and authoring states.** These are presentation categories over
existing native state, not a new persistence or protocol enum. Visible controls
must use their actual registry binding and expose unavailable actions honestly.
No CLI incantation is a substitute for a manual editor action.

| Condition | Available behavior | Required observable result |
|---|---|---|
| No project / loading | Project opening and shell navigation appropriate to that state | No fictitious document or authoring-ready indication |
| Rendered, read-only | Inspect, navigate and use inherited view tools | Authoring unavailable with its actual reason; inspection never writes |
| Native authoring ready | Supported commands for the current document, revision, occurrence and writer | Named target and current binding are visible before mutation |
| Preview | Inspect the proposed local gesture, complete or cancel through the shared tool | Preview does not alter canonical source; cancel leaves source and journal unchanged |
| Commit pending | Present the in-flight result using the existing mutation admission rules | Do not report committed success before the engine result |
| Refused / unresolved / stale | Explain the actual failure and offer applicable navigation or refresh | No partial write, substitute occurrence, invented default or private-writer fallback |
| Completed / history / reopened | Refresh projections, canonical undo/redo and project Save/reopen | Resolved authored result agrees with the journal and persisted native source |

**Command and mutation boundary.** Menus, tool palettes, shortcuts and command
palette use the existing application registry. The target families remain native
Symbol placement, transform and heterogeneous deletion; Wire creation; label
and marker kinds; port creation/editing; Bus/member/entry authoring; symbol fields;
and ERC inspection/waiver flows. Listing a target family does not claim a GUI
binding exists. Unsupported bindings remain visible with an honest disposition
in `DATUM_GUI_MENU_BINDINGS.md`; implementation must reconcile that ledger.

A supported local manual gesture uses the native operation builder, typed
OperationBatch, engine commit/journal and resolver refresh. Preview and cancel
must not acquire private write powers. Canonical history owns undo/redo; Save is
project-scoped persistence, not a competing shard writer. Automation, broad batch,
cross-domain and review-required changes retain the existing proposal boundary.
ERC inspection is read-only; waiver mutation retains its governed authority.
The authoring contract's updated journaled-operation grounding and final PM053
identity amendment control contradictory older no-undo/no-hierarchy summaries.
Those summaries remain explicit SURFACE-S04 reconciliation items.

**Electrical context.** Inspectors and finding navigation expose stable native
Net identity and occurrence-qualified relationships. PM053 C01–C03 governs
lawful pending and mismatch states, read-only identity resolution and Bus
declarations. A Bus has no board projection. Equal names, geometry or member sets
cannot create correspondence. Findings disclose the revision they describe and
must not imply a check is current after the authored model changes.

**Accessibility.** Consequential controls have discoverable names and keyboard
operation through the shared shell/tool contracts. Focus remains visible through
pane switching, narrow layouts, Zoom and Stage. Binding, severity, pending and
refusal information uses text or icons as well as color. The implementation
inherits token scaling, contrast and reduced-motion requirements. Keyboard and
assistive-technology qualification must cover the ordinary application; widget
unit tests and screenshots alone do not establish it.

**Delivery and conformance.** Existing receipts may be reused only within their
recorded scope. The first Symbol/Wire and isolated Move proofs use synthetic
native test data; they do not establish an approved production seed library.
No full surface, physical-input, assistive-technology or HUMAN acceptance is
asserted by this draft. The matrix names the required workflow and proof home;
future execution must attach actual evidence rather than mark a prose target PASS.

| Ordinary application workflow | Required assertion / negative control | Evidence or future proof home |
|---|---|---|
| Open native Design; navigate source and repeated occurrence; tile with Board; switch focus; Zoom/Stage | View actions have no authored delta; focus owns tools/Inspector; occurrence remains qualified | Shared pane/focus checks plus schematic integration and HUMAN frames, pending |
| Pick named Symbol; preview/cancel/place; Wire from pin through vertices to pin; undo/redo; Save/reopen | Cancel writes nothing; one completed gesture uses normal canonical history; reopened result agrees | `native-schematic/first-symbol-wire-delivery.json` and `independent-review.json`, bounded existing evidence |
| Select isolated Symbol; Move preview/cancel/commit; history; Save/reopen | Exact origin/pins/bindings move together; connected, repeated, ambiguous and positioned-owner cases retain current refusal | `native-schematic/symbol-move-delivery.json` and `symbol-move-delivery-review.json`, bounded existing evidence |
| Attempt unsupported, unresolved-binding, stale-revision or wrong-writer edit | Actual reason is visible; source and journal remain unchanged; no false pending success | Native admission/protocol checks and ordinary GUI refusal integration, remaining coverage pending |
| Inspect Net/Bus and navigate ERC finding | PM053 qualified identity; pending/mismatch honest; no fabricated Bus board identity; revision currentness visible | Identity/check producer tests plus schematic integration, pending |
| Operate picker, tools, Inspector and findings by keyboard/assistive technology | Reachable controls, meaningful labels, visible focus, non-color feedback and reduced motion | Shared accessibility checks plus ordinary-app qualification, pending |
| Compare normal/narrow/tiled/Zoom/Stage, preview and refusal frames | Controlling tokens and inherited shell/selection agree; no clipping or false readiness | Same-engine regression frames plus owner HUMAN visual disposition, pending |

Evidence paths in this matrix are relative to
`docs/reviews/workflow-delivery-rollout/`. Implementation seams are the existing
native schematic projection/loading, authoring protocol, shared selection and
runtime native-edit owners identified by the SURFACE-S01 inventory. Extend those
owners under a bounded execution grant; do not introduce parallel editor services.

**Owner disposition and visual reconciliation.** SURFACE-S05 must settle the
candidate hierarchy/occurrence discovery and opening presentation, readiness and
refusal presentation across pane sizes, unsupported-family presentation and
initial implementation breadth, and the library-browser integration/warning
presentation after SURFACE-S03. It must record actual visual/product acceptance
and select any implementation successor explicitly. Retained target families
cannot be silently reduced. Expanded connected/repeated transforms, new commit
classes, library source adoption, standards thresholds or new mechanisms require
their actual owner/numbered-decision authority separately.

Claude's bounded prototype reconciliation is: review the schematic-editor
study's historical Sheet labels and navigation region against continuous Design
and qualified occurrence context; provide native readiness/refusal/pending and
preview frames; verify focus/Inspector/Console in narrow and tiled states. Preserve
PM019/PM021, shared selection and Publish distinctions. This is a requested review
outcome, not permission to choose new owner-reserved semantics or a claim that
HTML has been changed. The historical study remains evidence at its recorded
scope until its owning visual lane and owner disposition are complete.

## PCB GUI Target

The PCB editor target is:

- board rendering and layer appearance;
- selection/inspector over components, pads, tracks, vias, zones, text, outline,
  dimensions, rules, and findings;
- move/align/distribute, route track, place via, place text, zone tools;
- undo/redo through journal;
- DRC/check integration;
- manufacturing visibility.

Phase 1 proves rendering and shell usability before broader editing.

## Manufacturing And Checks Target

Checks and manufacturing are first-class GUI surfaces.

Required target capabilities:

- run checks from visible commands;
- inspect findings and affected objects;
- navigate from finding to canvas object;
- waive/deviate through governed flows;
- list output jobs;
- generate/validate/compare artifacts;
- view artifact files and projection proofs;
- see whether outputs are current with the active model revision.

The Tier E supervision snapshot may contribute status data, but primary GUI
surfaces should speak EDA language, not implementation telemetry.

## Phase 1 Recovery Slice

Phase 1 is **Application Shell + Board Render Fidelity**.

Deliverables:

1. real top menu and application command structure;
2. project/document load path visible from the shell;
3. board fixture opens from the GUI;
4. board rendering fidelity sufficient for human recognition;
5. layer/appearance controls for the accepted fixture subset;
6. selection and inspector over rendered board objects;
7. real terminal tab remains available but no AGENTS dock as a separate lane;
8. screenshot-golden and human-review acceptance path.

Phase 1 explicitly excludes:

- full schematic editor;
- full PCB editing;
- full library editor;
- new assistant surface;
- new backend domain work not needed to render the fixture honestly.

## Human Acceptance

Every GUI implementation milestone must produce one or more of:

- running GUI session for owner review;
- screenshot set from canonical fixtures;
- visual-diff report;
- explicit known-gap list.

A milestone is not complete until the human review result is recorded in the
work item or progress update.

## Governance

This spec is governed by decision 019 and classified in
`specs/spec_governance_manifest.json`.

Implementation that changes GUI product shape must update this spec and
`specs/PROGRESS.md` in the same change. Implementation that changes code
surface shape may also require a spec parity inventory, but this planning spec
does not add one by itself.
