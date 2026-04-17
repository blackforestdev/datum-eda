# M7 Imported Board Fidelity Plan

> **Status**: Active execution plan for the post-spike `M7` board-review
> fidelity track.
> This document defines the bounded correction program required before Datum
> should treat imported KiCad PCB board review as a credible product surface.

## Purpose

Turn the existing `M7` board-review fidelity diagnosis into a trackable
execution plan that fits the current roadmap.

This plan exists to:
- keep the current `M7` opening architecture spike narrow
- prevent ad hoc renderer polish from hiding import-truth problems
- define where imported-board fidelity work belongs in the roadmap
- give the team one bounded sequence with explicit ownership and acceptance
  gates

This plan follows:
- `specs/progress/m7_opening.md`
- `specs/M7_FRONTEND_SPEC.md`
- `docs/gui/M7_BOARD_REVIEW_FIDELITY_GAP.md`
- `docs/gui/M7_IMPORTED_BOARD_FIDELITY_CHECKLIST.md`
- `docs/gui/M7_IMPORTED_BOARD_FIDELITY_FIXTURES.md`
- `docs/gui/M7_IMPORTED_BOARD_FIDELITY_ARTIFACTS.md`
- `docs/gui/M7_IMPORTED_BOARD_FIDELITY_ISSUES.md`
- `docs/gui/TECHNICAL_PRINCIPLES.md`

## Roadmap Placement

This work belongs:
- after the opening `M7` architecture spike proves the crate boundary and
  review-workspace viability
- before `M7` broadens into richer board review claims or additional GUI
  workflows

This work does **not**:
- reopen `M5` routing semantics
- reopen `M6` strategy semantics
- broaden `M7` into editing, apply flows, schematic review, or 3D work

Correct roadmap interpretation:
- the opening spike proves architecture
- the imported-board fidelity track proves that imported KiCad board review is
  semantically trustworthy enough to support the opening product claims

## Problem Statement

The current `M7` route-review client can render and inspect a
`board_review_scene_v1`, but imported KiCad PCB review is not yet trustworthy
enough to be treated as a credible board-review surface.

The gap is not one issue. It is the combined effect of:
- imported board geometry that is too lossy
- scene data that is too weak to distinguish authored copper, unrouted
  connectivity, and proposed route geometry cleanly
- renderer semantics that can still drift toward linework or proxy-object
  readability instead of PCB-native readability

## Canonical Review Question

For the accepted imported-board fixture set, can an experienced PCB user look
at the Datum `M7` review screen and quickly distinguish:
- authored copper already present on the board
- unrouted connectivity / ratsnest state
- proposed route-review overlay geometry
- footprint and board context that match the imported KiCad design closely
  enough to trust the review

If the answer is "not yet," this track remains open.

## Fixture Policy

This track requires a small fixed fixture set rather than open-ended visual
spot checks.

The minimum fixture set should include:
- the half-routed Datum test board already used for live review checks
- at least one board with multilayer copper and nontrivial layer naming
- at least one board exercising pad-shape fidelity, including `oval` and
  `roundrect`
- at least one board with zone and outline edge cases that remain inside the
  supported KiCad ownership rules

Each fixture should have:
- the source `.kicad_pcb`
- the Datum-imported board review payload or launch path
- one checked human-reviewed KiCad reference screenshot where useful
- one checked Datum screenshot baseline once the track stabilizes

## Scope Split

This track is explicitly split into three subtracks.

### A. Import Fidelity

Owned primarily by:
- `crates/engine/src/import/kicad`

Focus:
- layer identity preservation without silent fallback corruption
- top-level board outline handling under explicit supported ownership rules
- pad-shape fidelity
- pad width/height fidelity
- pad rotation semantics
- roundrect / radiused-corner semantics where supported
- zone and authored copper extraction fidelity for supported KiCad forms

Non-goal:
- broad unsupported-geometry guessing in the frontend

### B. Scene Contract Fidelity

Owned primarily by:
- `crates/gui-protocol`

Focus:
- explicit authored-board primitive coverage needed for credible review
- explicit unrouted / airwire / ratsnest companion primitives for imported
  boards
- stable object identity and ordering
- explicit separation between authored, unrouted, proposed, and diagnostic
  render categories
- no frontend inference that materially changes board meaning

Non-goal:
- inventing a parallel semantic model outside engine-backed facts

### C. Renderer Fidelity

Owned primarily by:
- `crates/gui-render`
- `crates/gui-app` where workspace behavior matters

Focus:
- authored copper reads as copper
- unrouted connectivity reads as connectivity linework, not copper
- proposed overlays read as proposed copper, not airwires
- authored imported geometry inherits visibility and base appearance from
  layer/material semantics by default rather than ad hoc object-class styling
- selected and non-selected review states remain distinct without semantic
  confusion
- footprint and board context remain readable under review focus

Non-goal:
- using styling alone to compensate for missing scene truth

## Sequenced Work Order

The team should execute this track in the following order.

### Stage 0: Freeze The Truth Set

Deliverables:
- checked fixture manifest for the imported-board fidelity set
- named acceptance screenshots or screenshot targets
- one tracked issue inventory grouped by `import`, `scene-contract`, and
  `renderer`

Exit condition:
- the team is reviewing the same boards and the same expected distinctions

### Stage 1: Stop Silent Import Corruption

Priority:
- highest

Deliverables:
- no silent collapse of unresolved layer identities onto `F.Cu` or other
  default copper layers
- explicit supported/unsupported behavior for outline extraction
- explicit supported/unsupported behavior for pad geometry that cannot yet be
  represented faithfully

Exit condition:
- unsupported import cases fail explicitly or are clearly bounded
- supported cases no longer silently produce materially wrong board meaning

### Stage 2: Raise Pad And Footprint Fidelity

Deliverables:
- supported `circle`, `rect`, `oval`, and `roundrect` pads preserve width /
  height / drill semantics
- supported pad rotation semantics are preserved or explicitly bounded
- footprint-native display primitives or richer component display companions
  are available where needed for credible review

Exit condition:
- imported footprints no longer read only as coarse proxy boxes and generic pad
  dots on the accepted fixture set

### Stage 3: Add Unrouted Connectivity As A First-Class Scene Lane

Deliverables:
- explicit scene primitives for unrouted connectivity / ratsnest state on
  imported boards
- independent visibility control for that lane
- stable separation from authored and proposed geometry

Exit condition:
- the half-routed canonical board clearly shows authored routed copper,
  remaining unrouted connectivity, and any reviewed proposal as distinct
  visual classes

### Stage 4: Lock Board-Review Semantic Rendering

Deliverables:
- renderer contract for authored / unrouted / proposed / diagnostic states
- explicit layer/material discipline for authored board geometry with bounded
  exceptions for cases such as vias, through-hole pads, or board-boundary views
- screenshot-backed visual review on the canonical fixture set
- selected-state review emphasis that does not collapse back into airwire-like
  linework

Exit condition:
- the viewport is semantically readable to a PCB user without side-by-side
  explanation

## Acceptance Gates

This track is complete only when all of the following are true for the
accepted imported-board fixture set.

### Gate 1: Import Trust

- supported layers retain correct identities
- supported outline geometry is imported under explicit ownership rules
- supported pads preserve the physical dimensions and shape semantics needed
  for review
- unsupported cases do not silently degrade into wrong board meaning

### Gate 2: Scene Trust

- `board_review_scene_v1` or its accepted successor carries the explicit
  authored / unrouted / proposed / diagnostic categories needed by the review
  surface
- object identity is stable on unchanged persisted state
- no frontend-owned geometry inference changes review meaning materially

### Gate 3: Review Readability

- authored copper does not read like unrouted connectivity
- unrouted connectivity does not read like copper
- proposed route overlays do not read like generic connectivity linework
- footprint and board context are recognizable enough to support review
  confidence

### Gate 4: Regression Coverage

- fixture-backed tests cover the supported import subset
- screenshot or image-based checks cover the canonical board-review states
- the team has one standing review board with both routed and intentionally
  unrouted regions

## Tracking Rules

This plan should be tracked in roadmap/status documents as:
- a post-spike `M7` fidelity track
- still inside the opening `M7` review milestone
- required before broadening imported-board review claims

Status updates belong in:
- `specs/PROGRESS.md`

Scope/ordering references belong in:
- `specs/progress/m7_opening.md`
- `PLAN.md`

Working diagnosis and visual-gap notes remain in:
- `docs/gui/M7_BOARD_REVIEW_FIDELITY_GAP.md`

Concrete issue-sized execution work now lives in:
- `docs/gui/M7_IMPORTED_BOARD_FIDELITY_CHECKLIST.md`

Active defect inventory now lives in:
- `docs/gui/M7_IMPORTED_BOARD_FIDELITY_ISSUES.md`

Stage 0 fixture and artifact authorities now live in:
- `docs/gui/M7_IMPORTED_BOARD_FIDELITY_FIXTURES.md`
- `docs/gui/M7_IMPORTED_BOARD_FIDELITY_ARTIFACTS.md`

## What Counts As Success

Success is not "the screen looks nicer."

Success is:
- imported KiCad board review is credible on the accepted fixture set
- the review surface preserves the semantic distinctions KiCad users expect
- the frontend remains a disciplined consumer of engine-backed truth
- the team can tell, from tracked roadmap language, that this fidelity request
  is an active `M7` execution item rather than an optional polish pass
