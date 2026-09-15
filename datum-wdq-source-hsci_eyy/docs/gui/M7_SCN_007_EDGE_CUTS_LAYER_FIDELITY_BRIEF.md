# M7-SCN-007 Implementation Brief

> **Ticket**: `M7-SCN-007`
> **Stage**: Stage 1 follow-on (directly observed fidelity bug on canonical fixture)
> **Track**: Imported board fidelity inside opening `M7`
> **Status**: Ready for review; implementation contract against Option B
> **Product Decision**: Option B chosen (two coordinated views of the same truth)
>
> Authority for the decision:
> [docs/gui/M7_SCN_007_EDGE_CUTS_LAYER_FIDELITY_DECISION.md](/home/bfadmin/Documents/datum-eda/docs/gui/M7_SCN_007_EDGE_CUTS_LAYER_FIDELITY_DECISION.md)

## Purpose

Define the bounded implementation contract for making imported Edge.Cuts
geometry participate in the authored-layer model alongside the existing
board-boundary primitive, under one extraction source of truth and two
coordinated scene outputs.

This brief exists so the coding pass does not:
- reopen `M7-IMP-003` extraction or composition logic
- let the two views drift semantically (the assembled outline describing one
  shape while per-contributor primitives describe another)
- let `scene.outline` behave as a rogue always-on special case
- let the Edge.Cuts per-contributor primitives bypass the normal authored /
  layer visibility model

## Problem

After `M7-IMP-003` and `M7-REN-005`, outline recovery and outline visibility
gating both work. But imported Edge.Cuts is still represented only as a
dedicated board-frame primitive (`scene.outline` consumed with
`BoardSurfaceRole::Edge` styling). There is no per-contributor Edge.Cuts
primitive family in the scene, so Edge.Cuts does not participate in the
authored-layer model the way tracks, pads, zones, and component graphics do.

Observed on canonical `datum-test` (screenshot 2026-04-15): Edge.Cuts toggles
hide the outline, but the visible geometry is drawn as a board frame rather
than as authored layer graphics on Edge.Cuts.

## Scope

This ticket covers:
- scene-contract extension to add a second, authored-layer view of imported
  Edge.Cuts contributors
- renderer participation of that second view in the normal authored /
  layer-visibility model
- identity coordination between the two views
- picking/inspection semantics for imported Edge.Cuts

This ticket does **NOT** cover:
- any change to `outline_from_edge_cuts` or its assembly logic (`M7-IMP-003`
  extraction remains untouched)
- any change to import-side IR (`PlacedPad`, `Board`, etc.)
- extending authored-layer primitives to silkscreen, courtyard, fab, or other
  layers (those are separate scene-contract slices if/when needed)
- editing behavior on Edge.Cuts (M7 remains read-only review)

## Required Change

### Extraction Source Of Truth

Both views are derived from the same set of imported Edge.Cuts contributor
segments:
- top-level `gr_line` / `gr_arc` on Edge.Cuts
- footprint-embedded `fp_line` / `fp_arc` on Edge.Cuts under the footprint
  `(at x y rot)` transform (per the `M7-IMP-003` Option A rule)

For this ticket the extractor used by the scene builder is the same KiCad
parser the scene builder already uses today (the gui-protocol-side parse path
that produces `scene.outline`, `component_graphics`, etc.). No engine-side
IR expansion, no new engine API, no change to `outline_from_edge_cuts`.

The scene builder must walk the contributor set exactly once and emit both
views from that single walk — no double-parse path that could drift.

### View 1: `scene.outline` (authoritative board boundary)

Unchanged in shape. Continues to carry the assembled rounded-rect polygon
with `layer_id: "Edge.Cuts"` (added in `M7-SCN-006`).

Semantics:
- authoritative for the board-boundary concept
- consumed by fit-to-board, board-frame overlays, and any future
  dimension/packaging surfaces
- visibility already gated by `M7-REN-005` on `authored_visible` +
  `layer_visible("Edge.Cuts")`

### View 2: `scene.board_graphics` (authored Edge.Cuts graphics)

New primitive family on the scene.

New type `BoardGraphicPrimitive` in `crates/gui-protocol/src/lib.rs`:
- `object_id: String`
- `object_kind: String` — always `"board_graphic"` (stable coarse class,
  matching the existing scene style; keeps selection/filtering vocabulary
  unified and avoids proliferating object classes)
- `primitive_kind: String` — `"line"` or `"arc"`
- `source_object_uuid: String` — stable, derived from the KiCad contributor
  `(uuid ...)` when present; deterministic fallback otherwise
- `layer_id: String` — `"Edge.Cuts"` for this ticket (room to grow for
  future board-level authored layers)
- `path: Vec<PointNm>` — for lines: `[start, end]`; for arcs: interpolated
  point list produced by the same arc-interpolation used by `scene.outline`
- `width_nm: Option<i64>` — source stroke width, preserved when available

New scene field: `scene.board_graphics: Vec<BoardGraphicPrimitive>`.

The scene builder emits one `BoardGraphicPrimitive` per extracted contributor
segment during the same walk that produces `scene.outline`. Under footprint
transform, the emitted primitive carries world-space coordinates (same
transform as the outline view).

### Identity Coordination Rules

- `source_object_uuid` on each `BoardGraphicPrimitive` is derived from the
  KiCad contributor's own `(uuid ...)` form when present; for footprint-
  embedded contributors, it is the `fp_line`/`fp_arc` UUID. For contributors
  that have no UUID (edge case in synthesized boards), a deterministic
  fallback derived from `board_uuid` + contributor ordinal is used.
- `scene.outline[0].source_object_uuid` remains `board_uuid` (unchanged from
  `M7-SCN-006`). The two views do NOT share a source UUID at the top level;
  they are coordinated by both being produced from the same contributor list
  in the same extraction pass.
- Stable ordering: contributor primitives are emitted in the same
  deterministic order they are walked during extraction, and retain that
  order across unchanged persisted state.

### Visibility Behavior

Both views share the same visibility rule:
- hidden when `!authored_visible(state)`
- hidden when `!layer_visible(state, "Edge.Cuts")` (or the primitive's
  declared `layer_id`)

`scene.outline` is not allowed to draw when `authored_visible` is false or
the carrying layer is hidden. `M7-REN-005` already enforces this; this
ticket must not regress it.

`scene.board_graphics` must use identical visibility gating.

Neither view may be styled as always-on. Neither view may draw a ghost
outline when its layer is hidden.

### Appearance / Styling

- `scene.outline` continues to render via the board-frame role
  (`BoardSurfaceRole::Edge`) so it retains its distinct board-boundary
  readability under the current visual grammar.
- `scene.board_graphics` uses the standard authored-layer appearance system
  (same path used by component silkscreen graphics on F.SilkS / B.SilkS):
  layer-family color, stroke width from `width_nm`, normal dim/highlight
  behavior under `DIM UNRELATED` and selection.

The two views' styling must be distinguishable at a glance on the canonical
fixture — the board-boundary retains its frame character, the layer
graphics read as authored Edge.Cuts content, and both coexist without
double-drawing on top of one another to a degree that produces a muddy
result. If double-drawing is a visual problem in practice, the resolution is
a stacking rule (board-frame renders last, thinner; layer graphics render
first, wider) documented in `gui-render`; not a contract change.

### Picking / Inspection Rule

Click hit-testing on imported Edge.Cuts geometry follows this priority:
1. **`BoardGraphicPrimitive` is the primary pick target.** A click that
   lands on a specific Edge.Cuts contributor selects that contributor
   primitive. Inspector shows the contributor's `object_id`,
   `source_object_uuid`, `layer_id`, `shape_kind`, and stroke width.
2. **`scene.outline` is NOT a click target under this ticket.** Its role
   is viewport framing and board-frame rendering; the M7 review workspace
   does not select "the whole board outline as one object." If such a
   selection becomes useful later, it is a separate ticket.

This keeps picking behavior consistent with other authored geometry (tracks,
zones, component graphics all select their individual primitives), while
preserving the board-boundary's role as a visual/geometric reference rather
than a user-selectable object.

## Minimum Code Surface To Audit

The implementation pass must inspect at least:
- `crates/gui-protocol/src/lib.rs` — scene contract types (`OutlinePolyline`,
  add `BoardGraphicPrimitive`), `BoardReviewSceneV1.outline`, add
  `BoardReviewSceneV1.board_graphics`, scene builder outline-assembly site,
  fixture round-trip test
- `crates/gui-protocol/testdata/board_review_scene_v1.json` — fixture must
  remain round-trippable; new field gets `#[serde(default)]` so the checked-
  in fixture does not need to be regenerated unless adding coverage for the
  new primitive family is desired
- `crates/gui-render/src/lib.rs` — new draw path for `scene.board_graphics`
  with authored-layer appearance + normal visibility gate; hit-region
  participation for picking; confirmation that the existing outline draw
  path (M7-REN-005) is unchanged

## Supported Behavior Rule

After this ticket lands:
- imported Edge.Cuts geometry is represented in the scene both as the
  assembled board-boundary `scene.outline` and as per-contributor
  `BoardGraphicPrimitive` entries under `scene.board_graphics`
- both views obey identical authored + Edge.Cuts-layer visibility gating
- layer appearance for the authored view follows the same appearance system
  used for other authored-layer graphics
- picking returns individual `BoardGraphicPrimitive` targets; the
  board-boundary is not itself a pick target
- identities are stable on unchanged persisted state

## Fixture Proof

Implementation proof uses the frozen Stage 0 fixture authority:
- [docs/gui/M7_IMPORTED_BOARD_FIDELITY_FIXTURES.md](/home/bfadmin/Documents/datum-eda/docs/gui/M7_IMPORTED_BOARD_FIDELITY_FIXTURES.md)

Minimum proof obligations:
- canonical `m7-datum-test-half-routed`:
  - with `EDGE.CUTS ON` + `AUTHORED ON`: both the board-frame outline AND
    per-contributor Edge.Cuts authored graphics are visible, stylistically
    distinguishable, stable across repeated launches
  - with `EDGE.CUTS OFF` or `AUTHORED OFF`: both views hide together
  - clicking on a specific Edge.Cuts line selects the corresponding
    `BoardGraphicPrimitive`; the inspector surfaces its source UUID and
    metadata
- `simple-demo-board` still imports and renders its existing outline with
  no regression
- one repo-native fixture containing top-level-only Edge.Cuts contributors
  also produces per-contributor primitives (not only the assembled outline)

If the frozen fixture set does not conclusively exercise a case the
implementation needs (e.g., a board with only footprint-embedded Edge.Cuts
contributors and no top-level Edge.Cuts), the implementing engineer must
ask the user to provide a real exported KiCad file — no synthetic KiCad
strings.

## Acceptance Criteria

`M7-SCN-007` is complete only when all of the following are true:
- `scene.board_graphics` is populated for every imported Edge.Cuts
  contributor under the extraction rule already landed in `M7-IMP-003`
- `scene.outline` continues to carry the assembled board boundary exactly as
  before this ticket
- both views coordinate identity via the single extraction pass and retain
  stable ordering/identifiers on unchanged persisted state
- renderer gates both views on `authored_visible` + `layer_visible(layer_id)`
- renderer styles the two views distinctly (board-frame vs. authored-layer
  appearance) with no muddy double-draw on the canonical fixture
- picking selects individual `BoardGraphicPrimitive` entries; `scene.outline`
  is not a pick target under this ticket
- `board_review_scene_v1` fixture round-trip still passes, using
  `#[serde(default)]` back-compat for the new field
- existing scene-contract tests still pass
- added tests use real fixtures or pure helper unit tests only; no
  synthetic KiCad strings

## Suggested Test Additions

Allowed additions:
- pure helper / contract unit tests: `BoardGraphicPrimitive` serde round-trip
  against a small hand-authored JSON snippet (this is contract JSON, not
  synthesized KiCad content, so it is allowed)
- fixture round-trip test confirming `board_review_scene_v1.json` deserializes
  with the new field defaulted
- real-fixture integration test: load one real `.kicad_pcb` via the existing
  scene-loader path, confirm `scene.outline.len() == 1` and
  `scene.board_graphics.len() > 0`, confirm layer_id and ordering
- real-fixture integration test: identity stability across two loads of the
  same unchanged file

Not allowed:
- fabricated `.kicad_pcb` / `.kicad_sch` strings in test bodies

## Deliverable Summary

Expected patch shape:
- new `BoardGraphicPrimitive` type in `gui-protocol`
- new `scene.board_graphics` field with `#[serde(default)]`
- scene builder emits both views from one extraction walk
- `gui-render` consumes the new field with layer-appearance draw + hit-region
  participation, preserves current outline draw path
- contract + real-fixture tests; no synthetic fixtures

## Follow-On Relationship

This ticket lands after:
- `M7-IMP-003` (outline recovery) — source of truth extractor is already in
  place
- `M7-SCN-006` (outline layer_id) — `scene.outline` already carries Edge.Cuts
  identity
- `M7-REN-005` (outline visibility gate) — outline view already respects
  authored/layer visibility

This ticket does NOT remove the need for:
- future extension of `BoardGraphicPrimitive` to other board-level layers
  (courtyard, fab, mechanical) when relevant — separate slices
- `M7-IMP-011` (explicit multi-layer pad membership) — independent concern
- `M7-IMP-008`, `M7-IMP-009`, `M7-IMP-005` — remaining pad fidelity work
