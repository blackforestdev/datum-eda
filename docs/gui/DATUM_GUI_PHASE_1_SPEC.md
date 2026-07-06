# Datum GUI — Phase 1 Implementation Spec

> **Status**: Governed active implementation spec.
> **Authority**: The first build phase of the GUI recovery under
> `docs/decisions/PRODUCT_MECHANICS_019_GUI_PRODUCT_MODEL.md`; realizes the
> "Phase 1 Recovery Slice" of `docs/gui/DATUM_GUI_PRODUCT_SPEC.md` on the design of
> `docs/gui/DATUM_GUI_DESIGN_SPEC.md`.
> **Scope of this phase**: **Application shell + board render fidelity, READ-ONLY.**
> Phase 1 needs **no write path** — it is buildable today (reads work via
> `run_cli_json`; authoring waits for `DATUM_GUI_WRITE_PATH_PLAN.md`).
> **This is a rail for execution.** Build exactly what is listed, in order; the
> "Do NOT" list is as binding as the deliverables.

## The one job

A shell a human can **open, point at the `datum-test` fixture, and see the board
rendered at real EDA fidelity** — the first version of Datum you can actually
test. Everything in Phase 1 serves that sentence.

## Controlling inputs (read before building)

- `docs/gui/DATUM_GUI_PRODUCT_SPEC.md` — shell contract, document model.
- `docs/gui/DATUM_GUI_DESIGN_SPEC.md` — the *how*: visual language, workspace/mode
  model, design-language consistency, modularity.
- `docs/gui/prototypes/board-editor.html` — **the visual truth.** The built shell
  must match its composition, density, and token usage.
- `docs/gui/DATUM_GUI_CODE_LEVERAGE_AUDIT.md` — the reuse map (keep/adapt/replace).
- `docs/gui/menu_model.json` — the menu bar is rendered *from this data*, not
  hardcoded.
- Design Book tokens (decision 015) + `UI_LAYOUT_SYSTEM_CONTRACT.md` + decision 014
  (Taffy layout) — all chrome uses tokens and the solved layout; no hand-tuned px.
- `specs/M7_FRONTEND_SPEC.md` §2 (historical) — the concrete `board_review_scene_v1`
  data contract to reuse for the scene payload.
- Fixture: `~/Documents/kicad_projects/Datum-eda/datum-test/` (the M7 regression
  fixture).

## Scope

**Phase 1 IS:**
- A real application shell (menu bar + tool rail + panels + status bar) laid out
  by the Taffy layout at the Design Book tokens.
- The board rendered from the resolved engine model at professional fidelity.
- Read-only inspection: select, inspect, filter, navigate.

**Phase 1 IS NOT** (deliberately deferred):
- Any authoring/editing (place/move/route/delete) — needs the write path.
- Marking menus wired to operations — needs the write path + the `not_built` ops.
- The inline AI ghost overlay or the agent lane beyond a read-only terminal.
- Schematic / footprint / symbol editors (board mode only in Phase 1).

## Deliverables (build in this order)

- **D1 — App shell composition.** The regions of the prototype, solved by the
  Taffy layout in logical pixels, styled from tokens: top menu bar · left tool rail
  · left column (Project tree over Layers) · central board viewport · right column
  (Inspector) · bottom dock (read-only Terminal) · status bar. No hand-tuned
  offsets. Passes `check_gui_design_tokens` + layout-invariant tests.
- **D2 — Menu bar from `menu_model.json`.** Render the menu bar and items **from the
  manifest**. Items whose binding is `not_built` render **disabled** (visible,
  greyed — placement defines the IA); `gui_local` items wire to view actions;
  `verb` items are enabled read-only-safe or disabled if they mutate (Phase 1 is
  read-only). Icons resolve from the icon set (see D6).
- **D3 — Board scene at fidelity.** Load `datum-test` via `ProjectResolver::resolve()`
  + `materialized_source_shard_value(BoardRoot)` (never the raw promoted shard).
  Render, recognizably: board outline + field, **footprints as grouped physical
  objects**, **pads** (shape/size/layer/drill), **tracks as copper** (not generic
  lines), **vias as vias**, **zones** (filled/unfilled/stale), **silkscreen +
  ref/value text**, **layer identity + visibility**. This closes
  `M7_BOARD_REVIEW_FIDELITY_GAP.md`. Unsupported geometry is surfaced as
  unsupported/degraded, never silently drawn wrong.
- **D4 — Selection → Inspector.** Single-select any board object (component/pad/
  track/via/zone) → the Inspector shows its properties **read-only**, and canvas ↔
  panel cross-highlight. (Matches the design-spec selection model; edit fields are
  Phase-2, gated on the write path.)
- **D5 — Layers / Filters panel.** From the board stackup: per-layer color swatch +
  visibility toggle, active layer, dim-unrelated. Toggles are consumer-side view
  state.
- **D6 — Icon set.** Icons are declared in `docs/gui/icon_set.json` (Tabler MIT
  base + custom EDA glyphs); `check_menu_model.py` already enforces that every
  menu icon is declared and every `exists` EDA glyph is on disk. Author the **21
  `to_author` EDA glyphs** in the Design Book style (`crates/engine/assets/icons/eda/`),
  flipping each `to_author` → `exists`, and map the Tabler-sourced ids to real
  Tabler glyphs. Do not invent an undeclared icon — add the entry first.
- **D7 — Screenshot-golden acceptance.** Wire the visual-regression harness
  (`DATUM_GUI_VISUAL_REGRESSION_HARNESS.md`) to capture `datum-test` board goldens
  across the layout scale matrix {1.0, 1.25, 1.5, 2.0}, committed and human-reviewed.

## Reuse map (from the leverage audit — do not rebuild these)

**Keep:** `gui-render` wgpu renderer, design tokens + design-book checks, the
Taffy layout + invariant tests, hit-region/picking, the visual-capture/diff/runner
harness, `gui-protocol` view-model boundary, resolver-backed scene loading, the
PTY terminal infra (read-only dock).
**Adapt:** `ReviewWorkspaceState` → broaden into the app/document workspace model;
`BoardReviewSceneV1` → PCB-native fidelity (D3); Project/Filters/Inspector/Review
panels → the product panels.
**Replace:** the menu-less shell → real menu bar from `menu_model`; the M7
route-review-centered IA → the product IA; the six-button board-tool menu → (later)
editor tool palettes.

## Acceptance gates (how "done" is proven — not optional)

1. Launch the GUI, **Open → `datum-test`**, board renders with every D3 object class
   recognizable to a PCB engineer.
2. **Screenshot goldens committed** for `datum-test` across the scale matrix, plus a
   human-reviewed reference and a written **known-gap list** (per the product spec's
   Human Acceptance rule).
3. Menu bar is rendered from `menu_model.json`; `not_built` items are visibly
   disabled; `check_menu_model` + `check_gui_icon_assets` green.
4. Select a component → Inspector shows its real properties; layer toggles work.
5. Drift gates green (`run_drift_gates.sh`), including token + layout-invariant +
   menu_model + icon gates.

## Do NOT (binding)

- **Do not build a write/authoring path** or wire any menu/marking-menu item to a
  mutation. Phase 1 is read-only; authoring is gated on `DATUM_GUI_WRITE_PATH_PLAN.md`.
- **Do not synthesize CLI strings** into the terminal as an action mechanism
  (decision 019). Reads use `run_cli_json`; that is the only bridge in Phase 1.
- **Do not resurrect a supervision/journal meta-panel** (the vacated 013 misfire —
  see the `013` tombstone). Build the real board surface, not an instrument panel.
- **Do not add new object-class or format-specific tools** — capability is a
  parameter of a small verb set (CLAUDE.md Lean ethos).
- **Do not hand-tune pixel offsets** — the Taffy layout owns geometry.

## Verification / handoff

Run the app on `datum-test`; produce the golden set + known-gap list; record the
owner review result in the work item (Human Acceptance). Phase 1 is complete when
the acceptance gates pass and the owner signs off on the rendered board.
