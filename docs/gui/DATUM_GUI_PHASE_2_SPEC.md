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

- **P2.1 — Split Board+Schematic dual-pane view.**
  **spec only — build is a separate authorized execution phase.**
  Realize the prototype's dual-pane central viewport: Board and Schematic panes
  side by side under the Taffy solve, each with its own pane-header tool strip,
  one pane focused. No hand-tuned offsets; geometry is the layout's. Resolves the
  design-spec's still-open split-view / dock-vs-overlay decision for this
  composition (owner call recorded before the build slice).
  *Dependency:* P2.0 landed.
  *Reuse:* the Phase-1 Taffy layout + invariant tests and the
  central-split-pane region already present in the Phase-1 shell composition (D1
  built a "board/schematic split-pane viewport with Board focused"); P2.1
  populates the second pane's geometry, it does not invent a new layout engine.
  *Check disposition:* **TO-ENFORCE** — extend the layout-invariant tests
  (`crates/gui-render`) with the dual-pane region assertions and re-capture the
  shell/board goldens; lands **with** the P2.1 build slice, never red against
  un-built structure (conformance §1 rule).

- **P2.2 — Schematic pane populated from the engine.**
  **spec only — build is a separate authorized execution phase.**
  Fill the P2.1 schematic pane with the `datum-test` schematic **read-only**,
  from resolved engine truth — reusing the existing
  `load_kicad_schematic_workspace_state` path and the
  `crates/gui-protocol/src/schematic_scene_import.rs` scene contract (both already
  present). Symbols render at the locked IEC-rectangular standard (Rendering
  Book); no editing, no wiring authoring.
  *Dependency:* P2.1 layout landed.
  *Reuse:* `load_kicad_schematic_workspace_state`, the schematic scene import
  contract, and the Rendering Book symbol standard — no new import path.
  *Check disposition:* **TO-ENFORCE** — add a schematic-pane screenshot golden for
  `datum-test` across the scale matrix and a scene-contract structural test; lands
  with the P2.2 build slice. **HUMAN** backstop: owner review of the populated
  schematic against the prototype (conformance HUMAN layer), backed by the
  committed golden so the approved look is regression-gated.

- **P2.3 — Cross-probe between the panes.**
  **spec only — build is a separate authorized execution phase.**
  Selecting an object in one pane highlights the linked object in the other
  (component ↔ its schematic symbol), built on the existing
  `SelectionTarget`/`select_authored_object` substrate and
  `crates/gui-protocol/src/context_envelope.rs::from_selection` — one selection
  identity projected into both scenes. Read-only: cross-probe is a
  consumer-side selection projection, **not** a journaled operation (CLAUDE.md:
  selection/hover are consumer-specific, never operations).
  *Dependency:* P2.1 + P2.2 landed.
  *Reuse:* `SelectionTarget`, `select_authored_object`,
  `context_envelope::from_selection`, and the Phase-1 canvas↔panel
  cross-highlight (D4) — extend the same selection substrate across panes.
  *Check disposition:* **TO-ENFORCE** — a selection-projection test asserting one
  selected identity resolves to the highlighted object in both scenes
  (`crates/gui-render/tests/selection_ownership.rs` is the natural home), plus a
  cross-probe screenshot golden; lands with the P2.3 build slice.

- **P2.4 — Full populated inspector (Identity / Placement / Checks sections).**
  **spec only — build is a separate authorized execution phase.**
  Extend the P2.0 populated inspector into the named
  **Identity / Placement / Checks** sections the prototype shows and that
  `render_inspector.rs` currently marks deferred (its title-band docstring notes
  the "deferred populated-component inspector"). Read-only fields only; editable
  fields stay Phase-3 gated on the write path.
  *Dependency:* P2.0 (extends the same inspector branch); independent of
  P2.1–P2.3 and may be sequenced in parallel after P2.0.
  *Reuse:* the P2.0 populated-component render branch in
  `render_inspector.rs` — add sections, do not fork the inspector.
  *Check disposition:* **TO-ENFORCE** — inspector-content structural test naming
  the three sections + a populated-inspector golden; lands with the P2.4 build
  slice. **HUMAN** backstop: owner review of section content/composition against
  the prototype, golden-backed.

## Sequencing summary (dependency graph)

```
Phase-1 shell (D1–D7)  ──►  P2.0 (LANDED)  ──►  P2.1  ──►  P2.2  ──►  P2.3
                                     └────────────────────────────►  P2.4
```

P2.1 and P2.2 are **deferred build phases** — this spec sequences and specifies
them; it does not build them in the P2.0 workflow. P2.4 branches off P2.0
directly and may run parallel to P2.1–P2.3.

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
  is a planning/sequencing spec; only P2.0 has landed. P2.1–P2.4 are spec-only.
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
