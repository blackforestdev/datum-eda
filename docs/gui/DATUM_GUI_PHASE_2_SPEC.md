# Datum GUI — Phase 2 Implementation Spec

> **Status**: Governed **planning/sequencing** spec. Spec-only: it sequences and
> specifies Phase 2; it does **not** authorize a build. Each deliverable below is
> explicitly marked **"spec only — build is a separate authorized execution
> phase"** and stays at planning altitude (CLAUDE.md → Working Posture:
> planning vs execution).
> **Authority**: The second build phase of the GUI recovery under
> `docs/decisions/PRODUCT_MECHANICS_019_GUI_PRODUCT_MODEL.md`, on the systems of
> decisions 014 (Taffy layout) + 015 (Design Book tokens). Extends
> `docs/gui/DATUM_GUI_PHASE_1_SPEC.md` and inherits the check-disposition
> discipline of `docs/gui/DATUM_GUI_CONFORMANCE_SPEC.md`.
> **Scope of this phase**: the surfaces the Phase-1 board shell deliberately
> deferred — the **populated component inspector** and the prototype's **dual-pane
> Board+Schematic** composition with **cross-probe** — realizing the *full*
> `docs/gui/prototypes/board-editor.html`. Read-only where possible; anything that
> mutates is gated behind `docs/gui/DATUM_GUI_WRITE_PATH_PLAN.md` and is out of
> scope here.
> **This is a rail for sequencing, not for code.** Build only the slice a separate
> workflow is explicitly authorized to build; the "Do NOT" list is as binding as
> the deliverables.

## The one job

Phase 1 delivered a shell you can open on `datum-test` and see the board at real
EDA fidelity. Phase 2 makes that shell **match the controlling prototype's single
composition end to end**: a *populated* component inspector beside a *dual-pane*
Board+Schematic view whose two panes **cross-probe** the same selected object.
Everything in Phase 2 serves that sentence — and each piece is sequenced so a
freshly authorized build agent picks up exactly one tractable slice at a time.

## Controlling inputs (read before sequencing or building)

- `docs/gui/prototypes/board-editor.html` — **the visual truth.** Its populated
  component inspector and split Board+Schematic layout are the Phase-2 target
  composition. (Its schematic pane content and dual-pane geometry are the
  Phase-2 work; Phase 1 rendered only the board-focused single pane.)
- `docs/gui/DATUM_GUI_PHASE_1_SPEC.md` — the phase this one extends; the shell,
  board scene, selection→inspector cross-highlight, and golden harness it built
  are the substrate Phase 2 reuses, not rebuilds.
- `docs/gui/DATUM_GUI_CONFORMANCE_SPEC.md` — the check-disposition discipline
  (ENFORCED / TO-ENFORCE / HUMAN) every deliverable below inherits, and the
  same-engine visual-parity gate (`scripts/check_gui_visual_parity.py`, §0.1)
  whose canonical capture Phase 2 extends.
- `docs/gui/DATUM_GUI_PRODUCT_SPEC.md` — shell contract, document model, Human
  Acceptance rule.
- `docs/gui/DATUM_GUI_DESIGN_SPEC.md` — the *how*: visual language, workspace/mode
  model, the dock-vs-overlay and split-view design (its still-open decisions bind
  P2.1's layout).
- `docs/gui/DATUM_GUI_CODE_LEVERAGE_AUDIT.md` — the reuse map; Phase 2 leans on it
  hard (see per-deliverable reuse notes).
- Fixture: `~/Documents/kicad_projects/Datum-eda/datum-test/` — board
  (`datum-test.kicad_pcb`) and its schematic workspace.

## Scope

**Phase 2 IS:**
- A **populated single-pane component inspector** on the `datum-test` parity
  capture (P2.0 — landed).
- The prototype's **dual-pane Board+Schematic** layout (P2.1).
- The **schematic pane populated from the engine** (P2.2).
- **Cross-probe** selection between the two panes (P2.3).
- The **full Identity/Placement/Checks inspector** sections (P2.4).

**Phase 2 IS NOT** (deferred out, as in Phase 1):
- Any authoring/editing in either pane (place/move/route/wire/delete) — needs the
  write path (`DATUM_GUI_WRITE_PATH_PLAN.md`).
- A schematic *editor*. The schematic pane is a **read-only** populated
  cross-probe/reference surface in Phase 2.
- New object-class- or format-specific tools (CLAUDE.md Lean ethos).
- Marking-menu / command-console ops wiring (Frontier steps 3–5).

## Deliverables (sequenced in dependency order)

Each deliverable names its dependency, its reuse leverage, an honest check
disposition per the conformance discipline, and the binding **build-phase
marker**.

- **P2.0 — Populated single-pane component inspector on the parity capture.**
  **spec only for the roadmap; build LANDED** (this workflow's entry deliverable).
  The `datum-test` visual-parity capture now presets a component selection via the
  `--select <refdes>` launch flag (wired in
  `crates/gui-app/src/app_bootstrap.rs` / `crates/gui-app/src/main.rs`), so the
  canonical capture command resolves `SelectionTarget::AuthoredObject` to a
  component and the inspector renders its **populated** component view
  (`crates/gui-render/src/side_panels/render_inspector.rs`) instead of the empty
  route-review action selection. This repoints the poor `--demo-known-good` empty
  target onto the prototype's single-pane composition.
  *Dependency:* Phase-1 shell + board fidelity (Phase-1 D1–D7) landed.
  *Reuse:* the existing `--select` flag, `select_authored_object`, and the
  already-populated inspector renderer — no new inspector code.
  *Check disposition:* **ENFORCED** — `scripts/check_gui_visual_parity.py`
  captures the canonical `--select`-preset command and diffs it against the
  owner-approved shell golden
  `crates/gui-render/testdata/golden/shell/datum-shell.golden.png`
  (same-engine, build-vs-build; re-approval via `--bless`), wired through
  `check_gui_conformance.py` → `run_drift_gates.sh`.

- **P2.1 — Split Board+Schematic dual-pane view. FIRST SLICE LANDED.**
  Realize the prototype's dual-pane central viewport: Board and Schematic panes
  side by side under the Taffy solve, each with its own pane-header tool strip,
  one pane focused. No hand-tuned offsets; geometry is the layout's.
  **Landed (first slice):** `ShellLayout::viewport_panes()` derives two panes +
  a divider gutter from the resolved `viewport` as a pure post-split (after the
  Taffy/fallback solve and after `scale_by`, so neither the solver nor `scale_by`
  becomes pane-aware). Pane A = Board · Layout, focused (lightened header, active
  tool cluster, accent focus dot, inset ACCENT pane frame, the board world scene
  re-projected into its canvas — `scene_viewport()` now returns pane A's canvas, so
  the RetainedScene projection, gpu scissor, and `world_point_at_screen` follow it
  with no further change). Pane B = Schematic · Sheet 1, unfocused (muted header,
  dimmed tools, no accent frame/dot) over a VIEWPORT_BG canvas with a centered
  "Schematic (coming)" placeholder caption — **no world geometry**. Focus is a
  single source of truth (`ViewportPanes::focused_document()`) driving the header
  chrome and, structurally, context-follows-focus (Inspector/Layers read the
  focused pane's document — pane A → board this slice). Invariant tests +
  render-contract tests added; shell/board goldens re-blessed; parity gate green.
  **Deferred to later P2.1/P2.2 slices:** real schematic world geometry in pane B
  (a second RetainedScene buffer + second projection uniform + second scissored
  gpu pass — a multi-scene GPU refactor), focus-switch input, the
  context-follows-focus toggle for the unfocused document, independent per-pane
  camera, and the exact split-ratio owner-approval against `board-editor.html`.
  *Dependency:* P2.0 landed.
  *Reuse:* the Phase-1 Taffy layout + invariant tests; P2.1's first slice derives
  the panes as a post-split, it does not invent a new layout engine.
  *Check disposition:* **ENFORCED (first slice)** — layout-invariant tests carry
  the dual-pane region assertions (`viewport_split_holds_two_pane_invariants_*`)
  and render-contract tests pin both pane headers, the placeholder caption, and
  pane-A-only focus chrome; shell/board goldens re-captured. Real schematic
  content in pane B stays **TO-ENFORCE** with P2.2.

- **P2.1 pane-tiling depth (decision 021). LANDED.** The hard-coded fixed split
  became the dynamic recursive tile tree: **dynamic single/split/close/nesting +
  Zoom (maximize)** driven from the View menu and Tab/Z/click keys, **independent
  per-pane warm cameras** (focus-switch never refits; the board camera is bound to
  the board scene leaf so it persists across focus), and **divider-drag resize** —
  dragging a split's gutter re-apportions its `ratio` live, clamped to
  `PANE_RATIO_MIN..MAX`, addressed by the split's root-to-node `path`
  (`WorkspaceLayout::set_ratio_at_path`) so nested splits resize independently. The
  renderer tags each divider with its split frame/orientation/path
  (`ViewportPanes::divider_at`); the runtime grabs the gutter before click-to-focus
  and consumes the release so a resize never changes focus.
  *Dependency:* P2.1 first slice landed.
  *Reuse:* the tile tree (`WorkspaceLayout`/`PaneNode`), `ViewportPanes` tiling,
  and the warm-camera store — no new layout engine.
  *Check disposition:* **ENFORCED** — model tests
  (`set_ratio_at_path_targets_the_named_split_and_clamps`), render tests
  (`dividers_carry_split_paths_and_are_grabbable`,
  `board_scene_stays_in_board_pane_regardless_of_focus`), app tests
  (`pane_resize` ratio math, `pane_cameras` warmth), and layout invariants. The
  **exact split-ratio owner-approval against `board-editor.html`** (the static
  default framing) remains a **HUMAN** review item — divider-drag now lets the
  owner set any ratio interactively; the shipped default preset value is the piece
  still pending owner sign-off.

- **P2.2 — Schematic pane rendered to the prototype (read-only).**
  Render the `datum-test` schematic in its pane so it MATCHES
  `docs/gui/prototypes/schematic-editor.html` — not merely "geometry appears" but
  the prototype's **colour, grid, and interaction**. Read-only (no authoring);
  symbols at the locked IEC-rectangular standard (Rendering Book). The target is
  the prototype element-for-element (audited 2026-07-10); the sub-slices enumerate
  it so no detail (e.g. the canvas grid) is dropped again.
  > **Scope discipline:** the original P2.2 entry ("fill the pane… symbols render
  > IEC… no editing") was too vague and produced a monochrome, gridless, static
  > pane that did not match the design. This entry is the corrected definition.

  **Landed (partial):**
  - *P2.2a — multi-scene render.* A second live world scene (the companion
    `schematic_scene`, a projected `BoardReviewSceneV1`) renders scissored to the
    schematic pane simultaneously with the board — the multi-scene GPU pass;
    companion carried through the KiCad materialize path from the original
    `.kicad_sch`. (`36a977a`, `f4fbd29`, `a901b42`.)
  - *P2.2b — symbol structure.* Hollow IEC symbol bodies (polyline outline), pin
    lines + terminal dots from `SymbolPin` positions, refdes/value/pin-name/
    pin-number text; labels/ports/sheet-instances render their names. (`e76543c`.)
  **Known gaps at "landed" (all P2.2, not P2.3/P2.4):** everything renders ONE
  monochrome layer colour — schematic geometry is smuggled onto board layer
  `F.SilkS` → off-white `#E8E6DC`; no schematic grid; static (non-interactive)
  camera; buses / power symbols / global-labels lack distinct geometry.

  **P2.2 completion (build to the prototype):**
  - **P2.2c — per-element colour fidelity.** Give schematic geometry its own colour
    path (schematic-specific `SceneLayer`s per net-role, resolved to prototype
    tokens) — this **lifts** the "projection-only, reuse board layers" posture,
    which is precisely what forces monochrome. Targets: wires `--wire` green
    `#4FA75A`; symbol bodies dark fill `#12141a` + `--sym #AEB4BB` stroke; pin
    lines/terminals `--sym`; junctions `--wire`; refdes `--tx`, value/pin-number
    `--tx3`, pin-name `--tx2`; no-connect `--tx2`.
  - **P2.2d — interactive schematic camera.** The FOCUSED pane (board OR schematic)
    is pan/zoom interactive; generalize the per-pane warm-camera model (the P2.1b
    board-bound scene) so the schematic is a first-class scene — interactive when
    focused, still rendered when not. (The owner must be able to zoom the schematic.)
  - **P2.2e — typed-object geometry.** Project the engine's typed schematic objects
    with prototype geometry + colour: `Bus` as gold `--bus #C2A13A` thick (≈3.2)
    lines + diagonal bus entries; power symbols (+3V3 bar / GND stack, `--pwr
    #B7BEC9`) detected by `lib_id`; `LabelKind::Global` as the `--info` blue
    pentagon tag, `Local` as the chip, hierarchical/`HierarchicalPort` styled;
    decoupling / crystal passive glyphs. Data is present in the engine `Sheet` model
    (`Bus`, `NetLabel`+`LabelKind`, `HierarchicalPort`) — this is projection plus a
    power-symbol classification pass.
  - **P2.2f — schematic canvas grid + frame.** Draw the prototype's SQUARE schematic
    grid (`#141821`, schematic pitch, zoom-tiered) as the schematic-pane underlay
    (the companion pass currently draws no grid); REMOVE the extraneous gold
    `Edge.Cuts` padded-bounds frame the projection currently emits (the prototype
    schematic has no sheet border). A proper title-block frame is future/out of
    scope.
  *Dependency:* P2.1 layout landed (done).
  *Reuse:* `schematic_scene_import.rs` projection, the world render pipeline, the
  per-pane warm-camera store, and the engine `Sheet` typed model — no new import.
  *Check disposition:* **TO-ENFORCE** per sub-slice — a `datum-test` schematic-pane
  screenshot golden (colour + grid) across the scale matrix; scene-contract
  structural tests asserting per-net-role layers/colours, bus/power/label geometry,
  and an interactive-camera test. **HUMAN** backstop: owner review against
  `schematic-editor.html`, golden-backed. NOTE the whole pane will **not** pixel-
  match the prototype's *zoomed single-circuit* mockup — it renders the real full
  sheet; the review target is per-element fidelity (colour, symbol, grid), verified
  by zooming in.
  *NOT P2.2 — do not absorb:* net selection/highlight/glow + cross-probe = **P2.3**;
  the net-centric inspector, the Sheets hierarchy panel, and schematic status-bar
  segments (Sheet/Grid/ERC) = **P2.4**; a schematic whose `.kicad_sch` only
  skeleton-imports (e.g. DOA2526) is the separate **schematic-import track**,
  upstream of render — P2.2 renders faithfully whatever the importer produces.

- **P2.3 — Schematic selection + cross-probe between the panes.**
  **spec only — build is a separate authorized execution phase.**
  Two coupled pieces: (1) **schematic-side net/object selection + highlight** — the
  prototype's `--acc` pink highlight with `netglow` on the selected net's wires,
  pins, and label (there is no schematic selection colour path today; it depends on
  the P2.2c schematic colour path); and (2) **cross-probe** — selecting an object in
  one pane highlights the linked object in the other (component ↔ its schematic
  symbol; net ↔ its copper), built on the existing
  `SelectionTarget`/`select_authored_object` substrate and
  `crates/gui-protocol/src/context_envelope.rs::from_selection` — one selection
  identity projected into both scenes. Read-only: selection/cross-probe are
  consumer-side projections, **not** journaled operations (CLAUDE.md:
  selection/hover are consumer-specific, never operations).
  *Dependency:* P2.1 + P2.2 landed.
  *Reuse:* `SelectionTarget`, `select_authored_object`,
  `context_envelope::from_selection`, and the Phase-1 canvas↔panel
  cross-highlight (D4) — extend the same selection substrate across panes.
  *Check disposition:* **TO-ENFORCE** — a selection-projection test asserting one
  selected identity resolves to the highlighted object in both scenes
  (`crates/gui-render/tests/selection_ownership.rs` is the natural home), plus a
  cross-probe screenshot golden; lands with the P2.3 build slice.

- **P2.4 — Populated inspector + schematic context surfaces.**
  **spec only — build is a separate authorized execution phase.**
  Three coupled surfaces the prototypes show and that today render board-only:
  (1) the **component inspector** — extend the P2.0 populated inspector into the
  named **Identity / Placement / Checks** sections (`render_inspector.rs` marks this
  deferred); (2) the **net-centric inspector** — when a schematic net is selected,
  the inspector shows **Net / Members / Checks (ERC)** per `schematic-editor.html`
  (context-follows-focus already routes the inspector to the focused pane); and
  (3) the **Sheets hierarchy panel** — when a schematic pane is focused, the left
  Layers slot shows the sheet hierarchy (the model exposes one root sheet today; a
  multi-sheet panel + selection lands here), plus the schematic status-bar segments
  (Sheet n/m, Grid, ERC count). Read-only fields only; editable fields stay Phase-3
  gated on the write path. ERC *findings* feed from the ERC engine (the P2.2e ERC
  *marker geometry* renders whatever findings exist).
  *Dependency:* P2.0 (component inspector); the net inspector + Sheets panel depend
  on P2.2 (schematic render) and share P2.3's selection substrate. May sequence in
  parallel after P2.0 for the component-inspector piece.
  *Reuse:* the P2.0 populated-component render branch in
  `render_inspector.rs` — add sections, do not fork the inspector.
  *Check disposition:* **TO-ENFORCE** — inspector-content structural test naming
  the three sections + a populated-inspector golden; lands with the P2.4 build
  slice. **HUMAN** backstop: owner review of section content/composition against
  the prototype, golden-backed.

## Sequencing summary (dependency graph)

```
Phase-1 shell (D1–D7)  ──►  P2.0 (LANDED)  ──►  P2.1 (first slice LANDED)  ──►  P2.2  ──►  P2.3
                                     └──────────────────────────────────────────────►  P2.4
```

P2.1's split-view first slice (two-pane LAYOUT + headers + focus + placeholder
pane B) has **landed**; real schematic content in pane B (P2.2) and the remaining
P2.1 depth (focus-switch, per-pane camera, exact split-ratio approval) are still
**deferred build phases** — this spec sequences and specifies them; it does not
build them in the P2.0 workflow. P2.4 branches off P2.0 directly and may run
parallel to P2.1–P2.3.

## Reuse map (from the leverage audit — do not rebuild these)

**Keep / extend:** the Phase-1 Taffy layout + invariant tests (P2.1), the
`--select`/`select_authored_object` selection substrate (P2.0/P2.3), the
already-populated inspector renderer `render_inspector.rs` (P2.0/P2.4),
`load_kicad_schematic_workspace_state` + `schematic_scene_import.rs` (P2.2),
`context_envelope::from_selection` (P2.3), and the visual-regression /
shell-golden harness (all deliverables' goldens).
**Do NOT replace:** none of the above is a rebuild; Phase 2 is depth on landed
substrate.

## Acceptance gates (how each slice is proven — per slice, not all at once)

Each deliverable is "done" only when, **for that slice**:
1. Its named check disposition is satisfied (ENFORCED gate green, or the
   TO-ENFORCE test/golden landed **with** the slice — never red against un-built
   structure).
2. Its screenshot goldens are committed and, where it carries a HUMAN backstop,
   owner-reviewed against the prototype with a written known-gap list (product
   spec's Human Acceptance rule).
3. Drift gates green (`run_drift_gates.sh`), including token + layout-invariant +
   visual-parity gates.

Phase 2 as a whole is complete when the canonical parity capture reproduces the
prototype's dual-pane, populated-inspector, cross-probing composition and the
owner signs off.

## Do NOT (binding)

- **Do not build any deliverable without separate execution authorization.** This
  is a planning/sequencing spec; P2.0 and the P2.1 split-view first slice
  (two-pane LAYOUT + headers + focus + placeholder pane B) have landed. The
  remaining P2.1 depth (real schematic content is P2.2) and P2.2–P2.4 are
  spec-only until separately authorized.
- **Do not build a write/authoring path** or wire any pane/inspector/menu item to
  a mutation. Phase 2 is read-only; authoring is gated on
  `DATUM_GUI_WRITE_PATH_PLAN.md` (Frontier step 5).
- **Do not build a schematic *editor*.** The schematic pane is read-only
  cross-probe/reference in Phase 2.
- **Do not journal cross-probe or selection.** Selection/hover are consumer-side
  projections, never operations (CLAUDE.md).
- **Do not synthesize CLI strings** into the terminal as an action mechanism
  (decision 019); reads use `run_cli_json`.
- **Do not pixel-diff wgpu against the HTML prototype** — same-engine goldens
  only; HTML is the HUMAN reference image (conformance §0, hard rule).
- **Do not hand-tune pixel offsets** — the Taffy layout owns geometry.
- **Do not add new object-class or format-specific tools** — capability is a
  parameter of a small verb set (CLAUDE.md Lean ethos).
- **Do not resurrect a supervision/journal meta-panel** (the vacated 013
  misfire).

## Verification / handoff

For each authorized slice: run the app on `datum-test`, produce the slice's
goldens + known-gap list, land its TO-ENFORCE check with the slice, and record the
owner review (Human Acceptance) in the work item. The Active Frontier
(`specs/PROGRESS.md`) is the single authority for which slice comes next.
