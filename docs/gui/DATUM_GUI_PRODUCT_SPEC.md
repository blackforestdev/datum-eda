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
- Application Status Bar candidate (retention and contents reopened under
  `DATUM_APPLICATION_STATUS_BAR_GUIDANCE.md`); selection, coordinates, and active
  tool MUST remain legible near their engaged pane regardless, while model/check/
  background state needs a durable global route whether or not the strip remains.

Minimum top menu:

- File: New Project, Open Project, Import, Save, Save As, Export, Close.
- Edit: Undo, Redo, Cut, Copy, Paste, Delete, Preferences.
- View: Fit, Zoom, Pan, Layer Visibility, Panels, Reset Workspace.
- Place: symbol/component/text/label/wire/track/via where enabled by the active
  editor.
- Route/PCB Tools: track, via, zone, move, align/distribute, measure.
- Project: validate, resolve/debug, project settings.
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
- Schematic sheet.
- PCB board.
- Rules/check report.
- Manufacturing outputs/artifacts.
- Terminal session.

Views are presentations of documents, not separate state authorities.

Examples:

- A PCB board document may have layout, layer-focused, review, and future 3D
  views.
- A schematic document may have sheet and hierarchy views.
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

- sheet rendering from native schematic state;
- symbols, pins, wires, labels, ports, buses, bus entries, junctions,
  no-connects, hierarchy markers;
- selection and inspector;
- place/move/delete/edit tools;
- net highlighting and ERC finding navigation;
- commit through engine operations.

This target is not Phase 1 unless board fidelity and shell usability are
accepted.

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
