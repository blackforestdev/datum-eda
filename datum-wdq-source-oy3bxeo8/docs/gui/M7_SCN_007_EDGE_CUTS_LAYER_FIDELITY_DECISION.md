# M7-SCN-007 Edge.Cuts Authored-Layer Fidelity Decision Memo

> **Ticket**: `M7-SCN-007`
> **Stage**: Stage 1 follow-on (directly observed fidelity bug on canonical fixture)
> **Track**: Imported board fidelity inside opening `M7`
> **Status**: Product decision required

## Purpose

Define the product decision required before implementing `M7-SCN-007`.

The unresolved question is not "how do we draw Edge.Cuts differently." It is
what Datum treats as the authoritative representation of imported Edge.Cuts
geometry inside the scene contract, and how the board-boundary concept
relates to that representation.

## Current Problem

`M7-IMP-003` recovered outline geometry from top-level and footprint-embedded
`gr_line`/`gr_arc`/`fp_line`/`fp_arc` on Edge.Cuts. The extractor succeeds.

But the scene contract still represents that recovered geometry as a
dedicated board-frame primitive family:
- [`OutlinePolyline`](/home/bfadmin/Documents/datum-eda/crates/gui-protocol/src/lib.rs:80)
  consumed exclusively through `scene.outline`
- [gui-render outline draw path](/home/bfadmin/Documents/datum-eda/crates/gui-render/src/lib.rs:1873)
  renders via `push_world_polyline_segments_capped` with
  `BoardSurfaceRole::Edge` styling — a board-frame role, not a layer role

Observed on canonical `datum-test` (screenshot 2026-04-15):
- Edge.Cuts appears in the Filters panel alongside F.Cu, B.Cu, etc.
- With `M7-REN-005` landed, toggling `EDGE.CUTS OFF` does hide the outline
- But the outline is still *drawn as a special board frame*, not as authored
  layer geometry with the same role-class as tracks on F.Cu or zones on B.Cu
- There is no concept in the scene of "an Edge.Cuts line primitive" equivalent
  to a `TrackPrimitive` on a copper layer

This produces a layer-role-fidelity gap:
- imported Edge.Cuts truth exists
- but does not participate in the authored-layer model on equal footing with
  other layers

## What The Decision Must Answer

For imported Edge.Cuts geometry, choose exactly one representation rule:

### Option A: One Shared Primitive Family

Interpretation:
- Edge.Cuts is just another authored layer.
- Imported Edge.Cuts contributors (`gr_line`, `gr_arc`, `fp_line`, `fp_arc` on
  Edge.Cuts) become authored graphic primitives tagged `layer_id: "Edge.Cuts"`
  in the same primitive family used for other per-layer authored graphics
  (component silkscreen, courtyard, fab graphics, etc.).
- Scene contract drops the dedicated `scene.outline` field, or keeps it only
  as a derived fit-to-board hint computed from the Edge.Cuts layer content.
- Renderer treats Edge.Cuts like any other authored layer: layer visibility,
  layer appearance (color/opacity from the layer appearance model), picking
  returns per-graphic Edge.Cuts primitives.

Pros:
- Conceptually clean: Edge.Cuts is a layer, and all layers produce primitives
  the same way.
- Simpler scene contract — one graphics stream indexed by `layer_id`.
- Edge.Cuts visibility and appearance are controlled uniformly with other
  layers; no special-case styling code path.
- No redundancy between `scene.outline` and any per-layer graphic set.

Cons:
- Loses the first-class "board boundary" concept in the scene.
- Fit-to-board, which currently consumes `scene.outline` for viewport bounds,
  must recompute its bounds from Edge.Cuts layer content (or from overall
  scene bounds).
- Does not match the mental model of most professional EDA tools, which
  typically present board outline as a distinct concept on top of the
  Edge.Cuts layer.
- Picking/inspection on a single Edge.Cuts line no longer implicitly
  identifies "the board boundary"; it identifies one contributor segment.
- Larger migration: existing consumers of `scene.outline` must be reworked.

### Option B: Two Coordinated Views Of The Same Truth

Interpretation:
- Keep `scene.outline` as the assembled board-boundary polygon (used for
  fit-to-board, board-frame overlays, dimension reporting).
- Additionally emit per-contributor Edge.Cuts authored graphic primitives in
  the normal authored-graphics family, tagged `layer_id: "Edge.Cuts"`, so
  Edge.Cuts participates in the authored-layer model alongside F.Cu, B.Cu,
  silkscreen, etc.
- Both views are derived from the same extracted Edge.Cuts contributor set
  and must carry coordinated identities (shared `source_object_uuid`, stable
  on unchanged persisted state).
- Renderer continues to draw the board-frame overlay from `scene.outline`
  when Edge.Cuts visibility is ON, and additionally renders the per-layer
  graphics with standard layer appearance.

Pros:
- Preserves the first-class board-boundary concept.
- Edge.Cuts also participates in the authored-layer model on equal footing
  with other layers.
- Matches conventions of real EDA tools (KiCad itself presents Edge.Cuts as
  a layer and the board outline as a related-but-distinct concept).
- Lower-risk migration: additive. Existing consumers of `scene.outline`
  continue to work; new per-layer Edge.Cuts graphics are additions.
- Fit-to-board semantics stay simple.

Cons:
- Redundant representation in scene data (same geometry described twice from
  different roles).
- Must enforce that both views stay coordinated — shared source UUIDs,
  consistent contributor list, identical extraction pass.
- Renderer carries more draw paths; double-draw risk unless both paths share
  visibility gates cleanly.
- Picking UX becomes richer but also more ambiguous: clicking on an
  Edge.Cuts line could select the board-boundary, the contributor graphic,
  or a user-facing "select the whole outline chain" aggregate — the
  implementation has to pick which.

## Required Outcome Of The Decision

Whichever option is chosen, `M7-SCN-007` must ensure:
- imported Edge.Cuts geometry participates in the authored-layer model with
  the same role-class as other authored layers (not only via the dedicated
  board-frame primitive)
- Edge.Cuts layer visibility, appearance, and picking/inspection behaviors
  are consistent with the chosen representation model
- scene identities remain stable on unchanged persisted state
- `M7-IMP-003` extraction/composition logic is NOT reopened; this ticket is
  scene-contract + renderer only

The following behavior is no longer acceptable:
- Edge.Cuts being treated purely as a dedicated board-frame primitive with
  no authored-layer representation
- Edge.Cuts being styled only via `BoardSurfaceRole::Edge` with no layer
  appearance participation

## Acceptance Criteria For The Decision

This decision is ready to implement when:
- the chosen representation rule (A or B) is recorded in this document
- the follow-on implementation brief for `M7-SCN-007` can be written without
  ambiguity about which primitive family owns imported Edge.Cuts geometry
- picking/inspection semantics for Edge.Cuts are specified (what a click on
  an Edge.Cuts line selects)

Current read:
- unresolved; awaiting product decision

## Next Step After Decision

Create the implementation brief for `M7-SCN-007` against the chosen option,
then code against it. Do NOT fold implementation into `M7-IMP-003` or any
import-side extraction ticket.
