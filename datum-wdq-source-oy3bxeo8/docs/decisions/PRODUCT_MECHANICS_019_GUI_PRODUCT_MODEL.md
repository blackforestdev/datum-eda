# Product Mechanics Decision 019: GUI Product Model

> **Status**: Ratified planning authority.
> **Date**: 2026-07-05.
> **Scope**: Datum's human-driven desktop GUI product model, application shell,
> document/view model, editor-to-operation authority, terminal/agent placement,
> and GUI recovery planning.
> **Builds on**:
> `PRODUCT_MECHANICS_001_CANONICAL_EDIT_MODEL`,
> `PRODUCT_MECHANICS_002_MANUAL_EDITOR_BASELINE`,
> `PRODUCT_MECHANICS_005_EMBEDDED_TERMINAL`,
> `PRODUCT_MECHANICS_007_PROJECT_WORKSPACE_MODEL`,
> `PRODUCT_MECHANICS_014_UI_LAYOUT_SYSTEM`,
> `PRODUCT_MECHANICS_015_UI_DESIGN_SYSTEM`, and
> `PRODUCT_MECHANICS_016_PRODUCT_NORTH_STAR`.
> **Layout contract**:
> `docs/contracts/UI_LAYOUT_SYSTEM_CONTRACT.md`.
> **Governed realization**:
> `docs/gui/DATUM_GUI_PRODUCT_SPEC.md`.
> **Current-code audit**:
> `docs/gui/DATUM_GUI_CODE_LEVERAGE_AUDIT.md`.

## Decision

Datum's GUI is a human-driven professional EDA application, not a route-review
spike, backend telemetry panel, terminal macro recorder, or AI-first shell.

The GUI product model is:

1. a normal desktop application shell with discoverable File/Edit/View/Tools/
   Project/Checks/Manufacturing/Window/Help surfaces;
2. a document/view workspace for project, library, schematic, PCB,
   manufacturing, and report surfaces;
3. direct graphical editors whose actions resolve to typed engine
   `OperationBatch` commits or proposals through the canonical edit model;
4. a real PTY-backed terminal for user shell sessions, scripts, and
   terminal-launched agents;
5. optional AI assistance launched through explicit terminal/session or future
   governed tool surfaces, never as a hidden mutation authority;
6. a board/schematic/library rendering and inspection experience that a human
   EDA user can evaluate visually before any workflow is called complete.

The existing M7 route-review shell is reusable infrastructure only. It is not
the GUI product model.

## Relationship To Existing Decisions

This decision does not supersede decisions 014, 015, or 016.

- Decision 014 remains the authority for solver-backed GUI layout geometry.
- Decision 015 remains the authority for visual tokens, density, typography,
  state encoding, and component styling.
- Decision 016 remains the authority for Datum's product north star and
  implementation priority.

This decision must also be reconciled with, and does not override:

- Decision 002 (Manual Editor Baseline): 019 is the GUI expression of the
  manual-first editor baseline; the GUI edit authority here realizes 002.
- Decision 007 (Project Workspace Model): 019's document/view workspace must
  resolve to 007's project/workspace authority split, not stand up a second
  workspace authority.
- Decisions 004 (AI Tooling Contract) and 006 (Assistant Surface): 019 is
  downstream of the existing AI/assistant tooling decisions. It does not remove
  the optional assistant surface ratified there. It only states that the
  recovered GUI product model does not make a separate AGENTS dock or
  assistant-first lane part of Phase 1, and that any assistant surface that
  appears in the GUI must still obey decisions 004 and 006.

Decision 019 fills the missing layer between those decisions: what the GUI is
as a product, what documents it hosts, how users invoke work, and how graphical
edits become canonical engine mutations.

Historical M7 review-shell documents remain evidence only unless the governed
GUI product spec explicitly adopts a part of them. Governed M7 renderer/text
documents that remain active are subordinate technical references only; they do
not define the GUI product shell, roadmap, or interaction model.

## Rationale

The current GUI implementation can open a narrow board-review workspace, render
some board context, expose a bottom terminal dock, and show engine supervision
data. That is not enough to make Datum usable as an EDA application.

The failure mode is structural:

- the shell has no real application menu or document model;
- the visible tools are a small board-review subset, not a coherent EDA tool
  model;
- GUI authoring commands currently synthesize CLI strings and send them through
  the terminal lane;
- board rendering still reads as proxy/review geometry rather than a
  trustworthy EDA board surface;
- the AGENTS dock entry creates product confusion when agents should be normal
  terminal-launched sessions unless a later decision ratifies a separate
  assistant surface.

Continuing to bolt tactical screens onto that shell would repeat the same
directional error. Datum needs a governed GUI product model before further GUI
implementation expands.

## Application Shell

The GUI must present a conventional, discoverable desktop application surface
while preserving Datum's custom high-density EDA workspace.

Required shell families:

- File: new, open, import, save, save as, export, close.
- Edit: undo, redo, cut/copy/paste/delete where applicable, preferences.
- View: fit, zoom, layer/appearance controls, panels, docks, documents.
- Tools: selection and active editor tools.
- Project: project settings, native validation, resolver/debug surfaces.
- Checks: ERC, DRC, rules, findings, waivers/deviations.
- Manufacturing: output jobs, artifacts, validation, compare/export.
- Window: documents, workspace layout, terminal sessions.
- Help: version/about, diagnostics, command reference.

The menu and toolbar model is not optional polish. It is the user's primary
proof that the program can be operated without hidden CLI knowledge.

## Document And View Model

Datum GUI work is organized around project documents and views.

Required document classes:

- Project overview.
- Library/pool browser and object editors.
- Schematic sheets.
- PCB boards.
- Rules/check reports.
- Manufacturing output jobs and artifacts.
- Terminal sessions.

Views are presentations of documents, not separate truth. A board may have 2D,
future 3D, layer-focused, and review views. A schematic may have sheet and
hierarchy views. All resolve back to one engine-owned `DesignModel`.

## GUI Edit Authority

Graphical editor actions must not synthesize CLI strings as their mutation
mechanism.

The required GUI mutation path is:

1. user gesture or property edit;
2. GUI action model with explicit document, selection, tool, and parameters;
3. engine-native write builder or proposal builder;
4. `OperationBatch`;
5. direct `commit()` for local, visible, undoable manual edits, or proposal
   creation for automation, cross-domain, destructive, or review-required
   edits;
6. journal append, diff/update, undo/redo, and GUI refresh through the resolver.

The terminal may run the CLI and may show command provenance, but it is not the
GUI editor's internal commit path. A GUI editor that works only by typing a
command into the terminal is not considered complete.

## Terminal And Agent Placement

Decision 005 remains controlling: the terminal is a real PTY-backed shell
surface. It may launch `codex`, `claude`, `aider`, shell scripts, build tools,
Datum CLI commands, vendor tools, or any user-installed program.

Agents are terminal-launched sessions by default. A separate AGENTS dock is not
part of the recovered product model. The existing AGENTS affordance should be
classified during implementation as either:

- deleted;
- folded into terminal session creation;
- or retained only as a terminal launcher button with no separate dock/pane.

A future non-terminal assistant UI would require a later numbered decision
because it would add a distinct product surface.

## Board Render Fidelity As First GUI Recovery Gate

Before Datum claims GUI authoring progress, the GUI must render a representative
board in a way an experienced PCB user recognizes and can inspect.

Phase 1 acceptance must include:

- loading the `datum-test` board fixture from the GUI shell;
- visible footprint/package geometry, pads, tracks, vias, zones, silkscreen,
  board outline, layer identity, reference/value text where supported, and
  object selection;
- a human-reviewed reference screenshot or fixture target;
- screenshot-golden regression coverage for the Datum render;
- a written unsupported-geometry policy where fidelity is intentionally
  bounded.

This gate is visual and human-reviewed. Unit tests alone cannot declare it
complete.

## Supervision Snapshot Placement

The existing `GuiSupervisionSnapshot` and Outputs `ENGINE SUPERVISION` strip are
not a product center. They are implementation telemetry produced by the Tier E
supervision slice.

The recovery plan must classify that surface as:

- adapt: keep selected EDA counts/status in a real status bar or diagnostics
  panel; or
- delete: remove journal-tip/read-only telemetry from the primary workspace if
  it does not serve human EDA work.

It must not grow into another meta/provenance panel.

## Non-Goals

This decision does not:

- specify every schematic, PCB, library, or manufacturing editor interaction;
- reopen the engine authority model;
- adopt a GUI framework;
- add code implementation tasks by itself;
- make imported-board fidelity the product north star;
- authorize AI or terminal surfaces to bypass the canonical operation model.

## Planning Deliverables

The immediate governed planning pass consists of exactly two realization
documents:

1. `docs/gui/DATUM_GUI_PRODUCT_SPEC.md` — the single GUI product specification.
2. `docs/gui/DATUM_GUI_CODE_LEVERAGE_AUDIT.md` — current-code classification:
   keep, adapt, replace, delete/deprecate, missing.

Additional per-area GUI specs must not be created until this decision's product
model is reconciled with the owner and implementation needs a narrower contract.

## Completion Gate

Decision 019 is satisfied when:

1. the governed product spec defines the shell, document model, editor authority,
   terminal/agent placement, and Phase 1 acceptance;
2. the leverage audit maps current `gui-protocol`, `gui-render`, and `gui-app`
   code to keep/adapt/replace/delete/missing categories;
3. `specs/PROGRESS.md` names GUI product recovery planning as the active GUI
   target instead of Tier E supervision;
4. governance manifest coverage classifies this decision, product spec, and
   leverage audit in the same change;
5. no GUI implementation work begins from the old M7 shell without checking it
   against the product spec and audit.
