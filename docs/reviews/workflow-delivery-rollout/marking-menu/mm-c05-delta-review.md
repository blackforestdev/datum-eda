# MM-C05 delta review — pcb.multiselect resolver + marking-menu overlay-stream fixes

<!-- EVIDENCE:MM-C05-DELTA-REVIEW:2026-10-27 -->

**Reviewer:** audit-agent-01 (independent, read-only; distinct from the implementing session). **Date:** 2026-10-27.
**Trigger:** PM recorded the MM-C06 gate disposition as **REVISE**, surfacing two real production defects in the inert marking-menu shell. The lane reopened at MM-C03. The prior C05 independent replay stays valid for the unchanged inert baseline; this record is the **delta only**, covering both fixes.
**Landing commit (landing_commit):** **be9f709e8a81d9bd5b9f543e3701322f3cc5e997** (stream fix; descendant of the resolver fix aff7059b).

## VERDICT: DELTA PASS on both deltas — each anchored to its landed commit, zero drift.

---

## Delta 1 — multiselect resolver fix, commit aff7059b3906bd7837552ecc8e4f911db7aca847
**Defect:** `pcb.multiselect` (its Align/Distribute sub-wheels) was unreachable in production — the resolver keyed only off the object under the cursor, so a multi-object selection opened that object's per-type wheel.
**Commit shape:** "fix(gui): resolve pcb.multiselect for multi-object marking-menu invocation"; author == committer == blackforestdev; no AI attribution; exactly 3 Rust files +44/−8 (27/6, 7/1, 10/1); focused tests in existing main_tests.rs (no new file).

**Pins (reviewed bytes == landed bytes, exact):**
- diff sha256 `3ae52ffe9b5af5b78fd78a6199ea6c74b1950026ec9722d2b10c3e04c0cc770c`
- blob `686dc5bb2394d5267bc15865974285477f488e0e` crates/gui-app/src/application/main_tests.rs
- blob `c790df506bdd8fe59e11c9d68b3243f2da131bb8` crates/gui-app/src/editor/runtime_selection_menu.rs
- blob `2961951dedd28218f70260e19702e9a7ebb0bf76` crates/gui-app/src/viewport/runtime_geometry_helpers.rs

**Findings:**
1. **Production-reachable from real selection state (not a test-only setter).** `marking_menu_key_for_target` gains `selected_count: usize`; `if selected_count > 1 → "pcb.multiselect"`, else the unchanged per-type / empty-canvas fallthrough. The single production caller `Runtime::open_marking_menu_at_cursor` (runtime_selection_menu.rs:121) derives the count from the live Runtime selection — `self.workspace().selection` → `SelectionTarget::Native(view).members.len()` (`members: BTreeSet<Id>`, gui-protocol selection_view.rs:68) — reached by the real right-click path (viewport/pan_gesture.rs:89) and the keyboard path. A genuine 2+ PCB selection opens the multiselect wheel regardless of the object under the cursor.
2. **Scope = contextual-key-resolution-only.** Only the `menu_key` string selection changes; no activation/dispatch (runtime_menu_actions.rs untouched), no render, no Operation/commit/journal. Inert boundary intact — changes *which* inert wheel renders, not what a leaf does. No enabled verb, no write path.
3. **Callers / arity:** exactly one production caller updated; all other references are the two focused tests; no stale old-arity caller in crates/.
4. **Boundary logic:** `>1 → multiselect`; `==1 → per-type`; `None/0 → empty`. Focused-test log (read, not re-run): `marking_menu_key_maps_phase_one_board_objects ... ok` + `marking_menu_key_resolves_multiselect_over_per_type ... ok` → 2 passed / 0 failed.

---

## Delta 2 — marking-menu overlay-stream fix, commit be9f709e8a81d9bd5b9f543e3701322f3cc5e997
**Defect:** the marking menu emitted through `Stream::Panel` (frame_preparation.rs), painting under/interleaved with the PCB `ViewportOverlay` content — the wheel was occluded over a real board.
**Commit shape:** "fix(gui): paint the marking menu in the overlay stream, above the PCB"; author == committer == blackforestdev; no AI attribution; 4 gui-render files +90/−19 (marking_menu.rs 19/6, frame_preparation.rs 15/12, menu_overlay_contract_tests.rs 53/0 new test, marking_shell_tests.rs 3/1). Descendant of aff7059b (two claim-extension commits between).

**Pins (reviewed bytes == landed bytes, exact):**
- diff sha256 `d0cbc9064ab1b316ca751fbb78072f96697e6b6475545d39caf590db84db415d`
- blob `28108a486dccd99c3d2fc04787f24c451347c81c` crates/gui-render/src/marking_menu.rs
- blob `032b75b667f416ac585e29b61de5c59d1fbb895d` crates/gui-render/src/render/frame_preparation.rs
- blob `f5769382fa095215d888f5f631832413b521639a` crates/gui-render/src/menu_chrome/menu_overlay_contract_tests.rs
- blob `302f7b6579a55f4d3281e8b7d2ef6719f436fb15` crates/gui-render/src/render/marking_shell_tests.rs

**Findings:**
(a) **Routing moved to the overlay stream.** `render_marking_menu` now sinks into `menu_overlay_quads` / `menu_overlay_text_runs` (was `panel_quads` / `text_runs`), and `frame_preparation.rs:464` records `consumers.note(Stream::Overlay, Consumer::Menu, …)` and `Stream::OverlayText` (was `Stream::Panel` / `Stream::Text`) — the same foreground menu-overlay streams used by the GlobalPreferences modal, composited after the viewport passes. So the wheel composites above the PCB.
(b) **Compositing/z-order only — no behavior change.** Inside `render_marking_menu` the body is unchanged (same `render_preview` → `clip_content`); only the sink buffers changed. The function is `pub(super)` with the same arity and param types (`Vec<Quad>`, `Vec<TextRun>`), so all callers compile; the single production caller is updated. **Hit-region ordering preserved:** the `hit_regions[menu_hit_start..].rotate_left(menu_hit_count)` at frame_preparation.rs:460 is retained and hit_regions is still passed through unchanged, so the menu's hits still win and clicks still land. No verb enablement, no Operation/commit/journal, no write path. The inert boundary is untouched.
(c) **Tests real; log read, not re-run** (frontend holds the runner). New assertion `marking_menu_composites_into_menu_overlay_above_the_pcb` proves an open menu grows `menu_overlay_vertices()` beyond the closed baseline (foreground sink, not the panel sink). The inert-boundary test `marking_menu_shell_renders_manifest_items_without_command_targets` is preserved — it now reads `menu_overlay_text_runs()`, consistent with the move, and still asserts no command targets. Guarded `cargo test -p datum-gui-render` → **9 passed / 0 failed** (log `_streamfix2.log`), both tests `ok`.

---

## Explicit non-claim
Both deltas' focused tests exercise resolver branching and render-sink routing at the unit/render-prep level; they do not drive real OS input and do not assert pixels. This review therefore does **not** claim a full-matrix / ordinary-workflow PASS, nor the visual outcome. Production-reachability (Delta 1) rests on static caller wiring; the overlay paint-order (Delta 2) is established at the stream-routing level. The literal visual outcome — the wheel readable over a real board, and the ordinary OS-input run — are separate records (frontend's recapture, inspected first-hand by PM; the ordinary-input run). **This verdict covers the code delta only.**

## Scope
Independent read-only review. No project source edits, no commit, no claim, no marker/digest mutation by the reviewer. This record is authored for verbatim installation and staging by the implementing/dev session in the single consolidated C06 acceptance commit (landing_commit = be9f709e); the reviewer does not commit it.
