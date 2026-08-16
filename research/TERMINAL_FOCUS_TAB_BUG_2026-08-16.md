# Datum terminal: Tab cycles panes / terminal-screen click does not arm focus

Captured 2026-08-16 while dogfooding a Claude Code session running INSIDE the
Datum terminal. This is a handoff note for work performed outside the embedded
terminal. The diagnosis comes from code reading and has not yet been reproduced
with a live click trace.

## Symptoms (owner report)
1. Pressing Tab inside the embedded terminal (Claude Code TUI running in it)
   cycles keyboard focus through the board/schematic editor panes instead of
   sending `\t` to the PTY.
2. Clicking on the terminal SCREEN area does not give the terminal keyboard
   focus. Clicking the terminal TAB (dock tab / session tab) does.
3. Copy/paste in and out of the terminal does not work for the owner (separate
   issue, not diagnosed here — see "Not investigated").
4. First Claude Code session inside the terminal locked up (not diagnosed).

## Root cause of 1 + 2 (same bug)
- Tab is encoded correctly: `crates/gui-app/src/terminal_input.rs`
  (`terminal_tab_sequence` -> `\t`, Shift+Tab -> `\x1b[Z`). Not the problem.
- Tab cycles panes only when `KeyboardFocus == Editor`
  (`crates/gui-app/src/keyboard_focus.rs` ~line 474,
  `workspace_action_should_fire`). So symptom 1 is really symptom 2:
  keyboard focus is never becoming `Terminal`.
- Click-to-focus for the terminal screen relies on
  `PreparedScene::hit_test` returning `HitTarget::TerminalScreen`
  (`crates/gui-app/src/main.rs` `handle_primary_click` ->
  `select_hit_target` -> `focus_after_hit_target`).
- `PreparedScene::hit_test` (`crates/gui-render/src/render/scene.rs` ~line
  250) returns the LAST pushed hit region that contains the point.
- Scene build order (`crates/gui-render/src/render/scene.rs` ~lines 109-131):
  `render_bottom_tabs` (pushes `TerminalScreen` region) runs BEFORE
  `render_scene` (board), so board hit regions win on overlap.
- In `crates/gui-render/src/render/overlay.rs` the component reference labels
  (~lines 61-70 and 89-97) and component texts (~lines 135-143) push
  `HitTarget::AuthoredObject(...)` hit regions in SCREEN space **without
  clipping to the board viewport**, even though the text itself is drawn with
  `draw_text_clipped(..., scene_viewport, ...)`. Only the top edge is clamped
  (`.max(board_field.y + 6.0)`); nothing clamps the bottom/sides.
  `ReviewAction` overlay regions (~lines 242, 258) are likewise unclipped.
- Result: on a real board (DOA2526 was loaded), whenever the camera is
  panned/zoomed so component labels project BELOW the board pane into the
  bottom dock, a click on the terminal screen resolves to a stale
  `AuthoredObject` label rect, not `TerminalScreen`. `click_terminal_screen`
  never runs, focus stays `Editor`, and Tab becomes pane cycling. The
  terminal TAB regions are not shadowed, which is why clicking the tab works.

## Suggested fix (small, contained)
1. Add `RectPx::intersect(self, other: RectPx) -> Option<RectPx>` in
   `crates/gui-render/src/render/layout.rs` (positive-area overlap or None).
2. In `render_scene` (`crates/gui-render/src/render/scene.rs` ~line 605),
   record `hit_regions.len()` before `push_scene_overlay_and_hits`, then clip
   every region pushed after that index to `scene_viewport` and drop empties
   (`retain`/`filter_map`). This enforces "hit regions match what is drawn"
   in one place instead of per call site.
3. Test in `crates/gui-render/src/terminal_dock_contract_tests.rs` (model on
   `terminal_screen_rect_is_the_dedicated_content_hit_target`): open the
   terminal dock, use a `CameraState` with `center_y_nm` shifted so fixture
   components project into the dock, and assert `hit_test` at the terminal
   screen center (and corners) still returns `Some(&HitTarget::TerminalScreen)`;
   additionally assert no `AuthoredObject`/`ReviewAction` region lies outside
   the scene viewport.

## Not investigated (out of time; session had to end)
- Copy/paste in/out of the terminal (owner cannot copy text out of the
  Datum terminal to hand to other agents — that is why this file exists).
  Relevant code: `is_copy_shortcut`/`is_paste_shortcut`,
  `copy_terminal_scrollback`, `paste_terminal_input` in gui-app.
- The first-session lockup. Candidate areas: PTY read backpressure /
  queued input (`f7fc50f`, `fdd6446`), heavy alt-screen redraw from the TUI.
- Owner said they have local uncommitted changes intended to address some of
  these; check `git status` before assuming the above still reproduces.
