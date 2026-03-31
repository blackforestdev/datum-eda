# M7 Frontend Opening Spec

> **Status**: Active opening specification artifact for the opening `M7` slice.
> This document defines the first concrete frontend-facing `M7` workspace
> contract and architecture spike boundary. It is intentionally narrow and
> derives from the frontend foundation under `docs/gui/`.

## Purpose

Translate the approved frontend foundation into one bounded `M7` opening slice:
- one read-only route-proposal review workspace
- one first frontend-consumed board review scene contract
- one architecture spike plan for proving the GUI direction without broadening
  into a general GUI milestone

This document must not reopen `M5` routing semantics or `M6` strategy
semantics.

## Governing Inputs

This spec follows:
- `docs/gui/FOUNDATION.md`
- `docs/gui/WORKSPACE_MODEL.md`
- `docs/gui/INTERACTION_MODEL.md`
- `docs/gui/CANVAS_REVIEW_MODEL.md`
- `docs/gui/VISUAL_LANGUAGE.md`
- `docs/gui/TECHNICAL_PRINCIPLES.md`
- `specs/progress/m7_opening.md`

If this document conflicts with milestone governance in `specs/progress/` or
status tracking in `specs/PROGRESS.md`, those documents remain authoritative
for milestone state while this document defines the concrete opening slice.

## 1. Opening Slice Definition

### 1.1 Exact Goal Statement

Open `M7` as the first native visual review workspace for Datum:
one read-only board route-proposal review surface that renders authoritative
authored board state together with one deterministic selected route proposal or
one saved route-proposal artifact, without mutating project state or changing
existing `M5`/`M6` semantics.

### 1.2 Exact First Workflow

The first supported user workflow is:
1. Open one native project in the GUI.
2. Load one deterministic `board_review_scene_v1` payload for the board.
3. Load one deterministic route review payload through the existing
   `review_route_proposal` machine surface, or open one saved
   route-proposal artifact review.
4. Render authored board context, proposal overlay, and review/evidence state
   in one read-only workspace.
5. Select one authored object, proposal action, or evidence item and inspect
   it through synchronized panels.
6. Use the integrated terminal lane for deterministic direct workflow/log
   visibility when needed.
7. Use the integrated AI lane for explanation and review support when needed.
8. Exit without mutating design state.

### 1.3 Explicit Non-Goals

The opening slice does **not** include:
- applying route proposals from the GUI
- generating routes from inside the GUI
- changing selector profile or routing policy in the GUI
- board editing or manual routing tools
- schematic review or editing
- 3D view or 2D/3D synchronization UI
- generalized diagnostics browsing outside the current route review target
- broad workspace customization or plugin systems
- terminal or AI lanes as independent authoring milestones

### 1.4 Explicit Persistent Panel Set For M7 v1

The `M7` opening workspace uses one fixed panel set:
- `Project`
  - project and board identity
  - review source summary
- `Filters`
  - authored/proposed visibility toggles
  - layer visibility toggles
  - dim-unrelated toggle for route review
- `Inspector`
  - currently selected authored object metadata
  - currently selected proposal action metadata
- `Review`
  - proposal summary
  - selected contract/profile/source identity
  - segment evidence list

Outside that panel set, the workspace also includes:
- one integrated command/terminal lane
- one integrated AI assistant lane

Both non-canvas lanes are present as first-class supporting lanes, but neither
expands beyond the read-only review workflow in this slice.

Locked workspace composition for `M7` v1:
- one viewport-centered three-column shell
- left sidebar containing `Project` above `Filters`
- right sidebar containing `Inspector` above `Review`
- one bottom dock strip containing the integrated `Terminal` and `Assistant`
  lanes

Post-spike validation items for this area:
- exact sidebar widths and heights beyond initial implementation defaults
- exact collapsed/default-open behavior details for bottom dock tabs

### 1.5 Explicit Selection Model For M7 v1

`M7` v1 uses a single-selection model only.

Selectable object classes:
- authored `component`
- authored `pad`
- authored `track`
- authored `via`
- authored `zone`
- proposal `action`
- proposal `overlay_primitive`
- review `evidence_item`

Selection rules:
- only one active selection at a time
- one separate active review target may establish related highlights
- panel selection and canvas selection must remain synchronized
- terminal and AI lanes may consume the current explicit selection context
- no multi-select
- no selection expansion commands
- no selection-triggered mutation

Selection-state vocabulary for `M7` v1:
- `hover`
- `focus`
- `selection`
- `active_review_target`

### 1.6 Explicit Authored / Proposed / Diagnostic Visual-State Model

The workspace must preserve a stable visual-state vocabulary:

- `authored_base`
  - normal authored board context
- `authored_dimmed`
  - unrelated authored geometry de-emphasized under review focus
- `authored_related`
  - authored objects related to the current selection or route anchors
- `proposed_overlay`
  - route proposal geometry not present in authored state
- `proposed_focus`
  - selected proposal action or overlay primitive emphasis
- `diagnostic_evidence`
  - evidence-linked markers or emphasis tied to the current proposal review

The opening slice must not visually blur authored and proposed geometry into a
single indistinguishable board state.

Locked visual differentiation rules for `M7` v1:
- authored geometry remains the baseline with normal solid rendering
- proposed geometry must remain visually distinct from authored geometry
- diagnostic/evidence emphasis must remain visually distinct from both authored
  and proposed geometry
- proposed geometry must not rely on color alone for differentiation from
  authored geometry

Post-spike validation items for this area:
- exact proposal accent palette
- final diagnostic/evidence palette

### 1.7 Explicit Read-Only Interaction Model

Supported interactions:
- pan and zoom
- hover preview
- click-to-select
- fit-to-board or fit-to-review-target
- visibility toggles through the `Filters` panel
- next/previous review item navigation
- terminal lane access for deterministic workflow/log inspection
- AI lane access for explanation and review support

Not supported:
- drag-edit
- manipulate geometry
- apply/commit from canvas, terminal, or AI lane
- hidden submodes with mutating consequences

Locked review-flow rule for `M7` v1:
- when a route review opens, the first active review target is the first
  proposal action in deterministic review order

### 1.8 Explicit Deferrals

Deferred beyond the opening slice:
- schematic review workspace
- 3D review lane
- multi-view synchronization
- command palette breadth beyond what the opening workspace needs
- workspace persistence/customization
- route accept/reject/apply controls in the GUI
- write-capable inspector editing
- generalized diagnostics browser
- any expansion of the terminal lane beyond read-only/supporting workflow and
  logs
- any expansion of the AI lane beyond explanation and review support

Post-spike validation items intentionally left open:
- exact dark-first versus equal dual-theme launch commitment
- finer-grained typography sizes and spacing tokens
- the full amount of review-list density and subsection collapsing behavior
  inside panels

## 2. board_review_scene_v1

### 2.1 Purpose

`board_review_scene_v1` is the first deterministic frontend-consumed board
review scene contract for `M7`.

It exists to provide:
- authoritative authored board context for rendering and picking
- stable identity and draw ordering for the opening review workspace
- bounded proposal and review companion primitives needed to render one route
  review target without inventing a parallel semantic model

### 2.2 Ownership Boundary

`board_review_scene_v1` owns:
- board context rendering data
- stable authored object identities used by the GUI
- bounded overlay and review companion primitives needed by the opening route
  review workspace

`review_route_proposal` remains authoritative for:
- proposal selection semantics
- review source identity
- contract/profile/policy/source metadata
- proposal action meaning
- segment evidence meaning

When `board_review_scene_v1` includes proposal overlay primitives, those
primitives are a deterministic render companion for the current reviewed
proposal, not a new semantic source of routing truth.

### 2.3 Determinism Rules

The contract must be:
- versioned
- byte-stable in ordering on unchanged persisted state
- identity-stable on unchanged persisted state
- explicit about units and draw ordering

The frontend must not infer missing geometry or semantics that materially alter
the review meaning of the scene.

### 2.4 Envelope

```json
{
  "kind": "board_review_scene",
  "version": 1,
  "scene_id": "string",
  "project_uuid": "uuid",
  "project_name": "string",
  "board_uuid": "uuid",
  "board_name": "string",
  "units": "nm",
  "source_revision": "string",
  "bounds": {
    "min_x": 0,
    "min_y": 0,
    "max_x": 0,
    "max_y": 0
  },
  "layers": [],
  "outline": [],
  "components": [],
  "pads": [],
  "tracks": [],
  "vias": [],
  "zones": [],
  "proposal_overlay_primitives": [],
  "review_primitives": []
}
```

### 2.5 Identity Rules

Identity rules for authored objects:
- reuse canonical persisted UUIDs where they already exist
- each renderable authored entry must carry:
  - `object_id`
  - `object_kind`
  - `source_object_uuid`

Identity rules for proposal and review companions:
- `proposal_overlay_primitives[*].overlay_id` must be stable for the reviewed
  payload on unchanged state
- `review_primitives[*].review_primitive_id` must be stable for the reviewed
  payload on unchanged state
- where a proposal primitive derives from a proposal action, it must carry
  `proposal_action_id`
- where a review primitive derives from an evidence row, it must carry
  `evidence_item_id` when available

The frontend must not mint replacement semantic IDs.

### 2.6 Required Authored Board Content

#### Layers

`layers[]` must include:
- `layer_id`
- `name`
- `kind`
- `render_order`
- `visible_by_default`

#### Board Outline

`outline[]` must include only the geometry needed to draw the board boundary in
the opening workspace.

#### Components

`components[]` must include:
- `component_uuid`
- `reference`
- `value` when present
- `placement_layer`
- `position`
- `rotation`
- `bounds`

The opening slice only requires coarse bounds and label placement sufficient
for pick/highlight and context rendering.

#### Pads

`pads[]` must include:
- `pad_uuid`
- `component_uuid`
- `net_uuid` when present
- `layer_id` or layer affinity
- `center`
- `bounds`
- `shape_kind`

#### Tracks

`tracks[]` must include:
- `track_uuid`
- `net_uuid`
- `layer_id`
- `width`
- `path`

#### Vias

`vias[]` must include:
- `via_uuid`
- `net_uuid`
- `position`
- `drill`
- `diameter`
- `start_layer_id`
- `end_layer_id`

#### Zones

`zones[]` must include:
- `zone_uuid`
- `net_uuid` when present
- `layer_id`
- `polygon`

### 2.7 Proposal Overlay Primitives

`proposal_overlay_primitives[]` must include only what is needed to render the
current reviewed proposal in the opening workspace.

Required fields:
- `overlay_id`
- `primitive_kind`
- `proposal_action_id`
- `layer_id` when applicable
- geometry sufficient to render the overlay
- `render_role`

Accepted opening primitive kinds:
- `track_path`
- `anchor_marker`

Accepted opening render roles:
- `proposed_overlay`
- `proposed_focus`
- `authored_related`

These primitives must be derivable from the currently reviewed proposal and
must not introduce new routing semantics.

### 2.8 Review Primitives

`review_primitives[]` must include only bounded review support data needed for
the first route review workflow.

Accepted opening review primitive kinds:
- `from_anchor_highlight`
- `to_anchor_highlight`
- `selected_segment_highlight`
- `related_authored_object_highlight`

These are visual review aids, not alternate design objects.

## 3. M7 Architecture Spike Plan

### 3.1 Crate Structure

The opening spike should use:
- `crates/gui-app`
  - application shell
  - workspace composition
  - lane orchestration
  - daemon/session wiring
- `crates/gui-protocol`
  - typed Rust models for `board_review_scene_v1`
  - typed Rust models for existing route review payloads
  - version checks, fixture loading, contract decoding
- `crates/gui-render`
  - board scene renderer
  - proposal overlay renderer
  - picking and highlight pipeline
  - camera/navigation

### 3.2 What The Spike Must Prove

The spike must prove:
- one Linux-native window can host the opening review workspace cleanly
- the canvas lane can render authored board context plus proposal overlay with
  acceptable clarity and responsiveness
- the locked viewport-centered three-column shell plus bottom dock strip works
  cleanly for the opening slice
- the fixed M7 panel set supports synchronized inspection
- the integrated terminal lane can surface deterministic review workflows/logs
  without becoming a separate truth model
- the integrated AI lane can consume current review context for explanation
  without becoming a separate truth model
- stable picking and selection can be maintained using contract identities
- the ownership boundary between engine outputs and frontend view-model/render
  state remains clean

### 3.3 Explicit Deferrals In The Spike

The spike explicitly defers:
- editing tools
- apply flows
- schematic rendering
- 3D rendering
- full workspace persistence
- packaging and distribution concerns
- full command palette breadth
- broad AI workflow design outside the opening review context

### 3.4 Success Criteria

The architecture is good enough to commit to if the spike demonstrates:
- clear authored/proposed/diagnostic review presentation
- reliable selection and pick identity
- synchronized panel/canvas/terminal/AI review context
- no evidence of frontend-owned semantic drift
- no obvious shell or renderer blockage for later schematic and 3D growth

### 3.5 Failure Criteria

The architecture should be reconsidered if the spike reveals:
- unstable picking or identity mapping
- awkward state ownership between protocol, render, and workspace layers
- inability to keep terminal and AI lanes aligned to explicit authoritative
  context
- a shell model that fights the fixed opening workspace
- rendering architecture that already shows clear blockage for future
  multi-view or 3D coexistence
