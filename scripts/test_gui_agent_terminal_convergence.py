#!/usr/bin/env python3
"""Hermetic regressions for the terminal convergence guard."""

from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("check_gui_agent_terminal_convergence.py")
SPEC = importlib.util.spec_from_file_location("terminal_convergence", MODULE_PATH)
assert SPEC and SPEC.loader
guard = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(guard)


class TerminalGridWriterGuardTest(unittest.TestCase):
    def test_claude_keyboard_controls_cannot_alias_cursor_restore(self) -> None:
        terminal_escape = "match final_byte { b'u' if self.params.is_empty() => restore() }"
        terminal_tests = r'''
fn claude_keyboard_controls_cannot_restore_a_stale_cursor() {
    apply(b"\x1b[>1u\x1b[?u\x1b[<uclaude");
}
fn split_claude_keyboard_controls_are_cursor_state_invariant() {}
'''
        failures: list[str] = []
        guard.check_claude_completion_controls(
            terminal_escape, terminal_tests, failures
        )
        self.assertEqual([], failures)

        failures = []
        guard.check_claude_completion_controls(
            terminal_escape.replace(" if self.params.is_empty()", ""),
            terminal_tests.replace(
                "split_claude_keyboard_controls_are_cursor_state_invariant", "removed"
            ),
            failures,
        )
        self.assertIn(
            "parameterized CSI u must not restore the terminal cursor", failures
        )
        self.assertTrue(
            any("split_claude_keyboard_controls" in failure for failure in failures)
        )

    def test_agent_tui_focus_batching_glyph_and_cache_contract_is_pinned(self) -> None:
        main = """
    fn window_event() {
        match event {
            MouseButton::Left, ElementState::Pressed => {
                focus_terminal_screen_before_mouse_report();
                report_terminal_mouse_button();
            }
            MouseButton::Left, ElementState::Released => {}
        }
    }
fn terminal_mouse_reporting_active() {
    terminal_mouse_report_allowed();
    self.terminal_screen_cell_at(x, y);
}
"""
        runtime_dock = "focus_before_terminal_mouse_press(); terminal_screen_cell_at();"
        drain = """
flush_output_batch(); tiny_chunk_flood_is_applied_once_per_session_per_turn();
slot.remove_when_closed = is_active;
fn natural_shell_exit_removes_its_tab_without_second_close() {}
"""
        geometry = 'include_bytes!("JetBrainsMono-Regular.ttf"); TextFace::Terminal;'
        cache = """
fn begin_text_buffer_frame() { entry.last_used_frame = 1; }
fn animated_agent_text_cache_retains_only_two_visible_generations() {}
fn ensure_text_buffer() { buffer.set_rich_text(); }
"""
        render_gpu = """
self.begin_text_buffer_frame();
self.cached_text_buffer_indices();
"""
        bottom_dock = """
TERMINAL_FONT_SIZE_PX: f32 = 12.0;
TERMINAL_LETTER_SPACING_EM;
draw_rich_text();
"""
        terminal_font_tests = """
fn terminal_font_advance_matches_shared_logical_cell_width() {}
fn terminal_cell_advance_combines_smaller_ink_with_explicit_spacing() {}
fn styled_terminal_colors_share_one_shaping_origin() {}
fn colored_shell_prompt_preserves_dollar_space_command_and_cursor_cells() {}
fn prompt_style_boundaries_do_not_restart_glyph_positioning() {}
fn terminal_rich_span_colors_participate_in_the_buffer_cache_key() {}
"""
        terminal_cursor = """
const CURSOR_HORIZONTAL_INSET_PX: f32 = 1.0;
fn trailing_slash_cursor_paint_stays_inside_the_next_logical_cell() {}
"""
        failures: list[str] = []
        guard.check_agent_tui_runtime(
            main,
            runtime_dock,
            drain,
            geometry,
            cache,
            render_gpu,
            bottom_dock,
            terminal_font_tests,
            terminal_cursor,
            failures,
        )
        self.assertEqual([], failures)

        failures = []
        guard.check_agent_tui_runtime(
            main.replace(
                "        match event {",
                "        poll_terminal_output();\n        match event {",
            ).replace(
                "focus_terminal_screen_before_mouse_report();\n"
                "                report_terminal_mouse_button();",
                "report_terminal_mouse_button();\n"
                "                focus_terminal_screen_before_mouse_report();",
            ),
            runtime_dock,
            drain.replace("flush_output_batch", "apply_each_chunk")
            .replace("slot.remove_when_closed = is_active", "removed")
            .replace(
                "natural_shell_exit_removes_its_tab_without_second_close",
                "removed",
            ),
            geometry.replace("JetBrainsMono-Regular.ttf", "IBMPlexMono-Medium.ttf"),
            cache.replace("last_used_frame", "unbounded_generation").replace(
                "set_rich_text", "set_text"
            ),
            render_gpu.replace("self.begin_text_buffer_frame();\n", ""),
            bottom_dock
            .replace("TERMINAL_LETTER_SPACING_EM", "removed")
            .replace("draw_rich_text", "draw_text"),
            terminal_font_tests.replace(
                "styled_terminal_colors_share_one_shaping_origin", "removed"
            ).replace(
                "prompt_style_boundaries_do_not_restart_glyph_positioning", "removed"
            ),
            terminal_cursor.replace("CURSOR_HORIZONTAL_INSET_PX", "removed"),
            failures,
        )
        self.assertIn(
            "terminal output must not drain before window input dispatch", failures
        )
        self.assertIn(
            "terminal focus must precede child mouse-report forwarding", failures
        )
        self.assertIn(
            "terminal tiny-output batching is missing flush_output_batch", failures
        )
        self.assertIn(
            "terminal glyph face is missing JetBrainsMono-Regular.ttf", failures
        )
        self.assertIn(
            "terminal text-cache bound is missing last_used_frame", failures
        )
        self.assertTrue(any("TERMINAL_LETTER_SPACING_EM" in failure for failure in failures))
        self.assertTrue(any("draw_rich_text" in failure for failure in failures))
        self.assertTrue(any("rich-text shaping buffer" in failure for failure in failures))
        self.assertTrue(any("slot.remove_when_closed" in failure for failure in failures))
        self.assertTrue(any("natural_shell_exit" in failure for failure in failures))
        self.assertTrue(
            any("styled_terminal_colors" in failure for failure in failures)
        )
        self.assertTrue(any("prompt_style_boundaries" in failure for failure in failures))
        self.assertTrue(
            any("terminal cursor-cell separation" in failure for failure in failures)
        )
        self.assertIn(
            "renderer must begin exactly one text-cache generation per frame", failures
        )

    def test_terminal_transport_boundary_is_pinned_and_cell_free(self) -> None:
        transport = """
fn open_pty_pair() {
    let _ = "/dev/ptmx"; grantpt(); unlockpt(); ptsname_r(); TIOCSCTTY(); TIOCSWINSZ();
    configure_child_pty();
}
"""
        failures: list[str] = []
        guard.check_terminal_transport_boundary(transport, transport, failures)
        self.assertEqual([], failures)

        failures = []
        guard.check_terminal_transport_boundary(
            transport + "\nfn rogue() { portable_pty(); TerminalScreen::default(); }",
            transport + "\nTerminalScreen::default();",
            failures,
        )
        self.assertIn(
            "terminal transport must not own cell/core marker TerminalScreen",
            failures,
        )
        self.assertIn(
            "third-party terminal dependency must not remain: portable_pty", failures
        )

    def test_terminal_input_mode_is_exclusive_and_never_detached(self) -> None:
        production = """
enum TerminalInputOwner { AttachedPty, RenameChrome, Unowned }
pub(crate) fn terminal_input_owner() {
    use TerminalInputOwner::{AttachedPty, RenameChrome, Unowned};
}
fn commit_terminal_ime_text() {}
fn write_attached_terminal_bytes() { write_bytes(); }
"""
        bottom_dock = '("CLOSE", HitTarget::TerminalSessionCloseActive)'
        failures: list[str] = []
        guard.check_terminal_input_mode(production, bottom_dock, failures)
        self.assertEqual([], failures)

        failures = []
        guard.check_terminal_input_mode(
            production.replace("Unowned", "LegacyDockLineEdit")
                + "\nDetachedReadOnly\n"
                + "\nfn complete_terminal_rename_input() {}\n",
            '("REATTACH", HitTarget::TerminalSessionReattachActive)',
            failures,
        )
        self.assertIn(
            "dead terminal line-edit marker must not remain: LegacyDockLineEdit",
            failures,
        )
        self.assertIn(
            "retired detached terminal mode remains: \"REATTACH\"", failures
        )

    def test_terminal_input_state_has_explicit_authorities(self) -> None:
        terminal_lane = """
pub rename_input: String,
pub rename_cursor: usize,
pub screen_cursor_row: usize,
pub screen_cursor_col: usize,
"""
        production = """
fn append_terminal_rename_text(&mut self, text: &str) -> bool {
    self.ui.terminal.rename_input.push_str(text);
    true
}
    fn after() {}
"""
        failures: list[str] = []
        guard.check_terminal_input_identity(terminal_lane, production, failures)
        self.assertEqual([], failures)

        failures = []
        invalid_lane = terminal_lane + "pub input: String;\npub cursor: usize;\n"
        invalid_production = production.replace(
            "self.ui.terminal.rename_input.push_str(text);",
            "self.ui.terminal.input.push_str(text);\n"
            "    self.write_foreign_shell_bytes(text.as_bytes());",
        ) + "\nfn append_dock_text() {}\n"
        guard.check_terminal_input_identity(
            invalid_lane, invalid_production, failures
        )
        self.assertIn(
            "terminal protocol must not expose generic input/cursor fields", failures
        )
        self.assertIn(
            "terminal production code must not use generic .terminal.input", failures
        )
        self.assertIn(
            "terminal chrome editor must not use generic marker fn append_dock_text",
            failures,
        )
        self.assertIn(
            "terminal rename text must never reach the foreign shell", failures
        )

    def test_workspace_hotkeys_are_press_timed_through_one_focus_predicate(self) -> None:
        valid = """
pub(crate) fn workspace_action_should_fire(focus: KeyboardFocus, visible: bool,
    state: ElementState, repeat: bool) -> bool {
    state == ElementState::Pressed && !repeat
        && key_route(focus, KeyClass::WorkspaceHotkey, visible) == RouteDecision::Editor
}
let workspace_action_pressed =
    workspace_action_should_fire(focus, dock_visible, event.state, event.repeat);
// Pane focus cycling
if tab && workspace_action_pressed { next(); }
if character && workspace_action_pressed { apply(); }
    if escape_released {}
"""
        failures: list[str] = []
        guard.check_workspace_hotkey_timing(valid, failures)
        self.assertEqual([], failures)

        invalid = valid.replace("ElementState::Pressed", "ElementState::Released")
        guard.check_workspace_hotkey_timing(invalid, failures)
        self.assertIn(
            "workspace hotkey timing predicate is missing ElementState::Pressed", failures
        )

        failures = []
        invalid = valid.replace(
            "if tab && workspace_action_pressed { next(); }",
            "if tab && event.state == ElementState::Released "
            "&& workspace_action_pressed { next(); }",
        )
        guard.check_workspace_hotkey_timing(invalid, failures)
        self.assertIn("workspace hotkey dispatch must not fire on key release", failures)

    def test_terminal_focus_reports_have_one_keyboard_owner(self) -> None:
        valid = """
            WindowEvent::Focused(focused) => { if !focused {} }
            WindowEvent::CursorLeft { .. } => {}
    pub(crate) fn set_application_focus(&mut self) {
        let report = keyboard_focus::terminal_focus_report_transition(old, next);
        self.workspace_mut().ui.focus = next;
    }
    fn initialize(state: &mut State) { state.ui.focus = ApplicationFocus::default(); }
    fn after() {}
"""
        workspace_layout = """
pub enum ApplicationFocus { Editor(PaneId), Terminal, Overlay }
"""
        window = """
            WindowEvent::Focused(focused) => { if !focused {} }
            WindowEvent::CursorLeft { .. } => {}
"""
        failures: list[str] = []
        guard.check_terminal_focus_reporting(
            window, valid, workspace_layout, "", valid, failures
        )
        self.assertEqual([], failures)
        invalid_window = window.replace(
            "if !focused {}", "self.report_terminal_focus_event(focused);"
        )
        invalid_authority = valid + "\nself.keyboard_focus = next;\n"
        guard.check_terminal_focus_reporting(
            invalid_window,
            invalid_authority,
            workspace_layout.replace("Terminal", "Shell"),
            "pub has_keyboard_focus: bool,",
            invalid_authority,
            failures,
        )
        self.assertIn("OS window focus must not emit terminal focus-report bytes", failures)
        self.assertIn(
            "runtime must not retain a rival keyboard-focus field", failures
        )
        self.assertIn("terminal lane must not retain a rival focus projection", failures)

    def test_editor_hits_are_clipped_and_terminal_focus_proofs_are_pinned(self) -> None:
        layout = "pub fn intersect() { right > x; bottom > y; }"
        scene = """
let scene_hit_start = prepared.hit_regions.len();
hit_clipping::clip_new_hit_regions();
"""
        clipping = ".drain(first_new_region..); region.rect.intersect(viewport);"
        focus_tests = """
fn non_mouse_child_click_selects_terminal_and_tab_never_cycles_editor_panes() {
    workspace_action_should_fire(); terminal_tab_sequence();
}
fn mouse_reporting_press_selects_same_terminal_authority_before_forwarding() {}
"""
        hit_tests = "fn editor_scene_hits_cannot_shadow_terminal_screen_at_adversarial_cameras() {}"
        failures: list[str] = []
        guard.check_terminal_hit_ownership(
            layout, scene, clipping, focus_tests, hit_tests, failures
        )
        self.assertEqual([], failures)

        failures = []
        guard.check_terminal_hit_ownership(
            layout.replace("intersect", "overlap"),
            scene.replace("hit_clipping::clip_new_hit_regions", "leave_unclipped"),
            clipping.replace(".drain(first_new_region..)", ".iter()"),
            focus_tests.replace("terminal_tab_sequence", "pane_focus_next"),
            "",
            failures,
        )
        self.assertIn("viewport hit clipping is missing pub fn intersect", failures)
        self.assertIn(
            "editor scene hit clipping is missing hit_clipping::clip_new_hit_regions",
            failures,
        )
        self.assertIn(
            "terminal focus convergence proof is missing terminal_tab_sequence", failures
        )
        self.assertIn("adversarial editor-hit ownership proof is missing", failures)

    def test_only_protocol_declaration_terminal_core_and_tests_may_mutate_grid(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            files = {
                "crates/gui-protocol/src/terminal_lane.rs": "fn pty_grid_mut() {}",
                "crates/gui-app/src/terminal_screen.rs": "state.pty_grid_mut();",
                "crates/gui-app/src/terminal_screen/parser.rs": "state.pty_grid_mut();",
                "crates/gui-render/src/terminal_contract_tests.rs": "state.pty_grid_mut();",
                "crates/gui-app/src/rogue_writer.rs": "state.pty_grid_mut();",
            }
            for relative, source in files.items():
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(source, encoding="utf-8")
            previous_root = guard.ROOT
            guard.ROOT = root
            try:
                failures: list[str] = []
                guard.check_terminal_grid_writers(failures)
            finally:
                guard.ROOT = previous_root
            self.assertEqual(
                ["terminal grid mutation escaped PTY interpretation: "
                 "crates/gui-app/src/rogue_writer.rs"],
                failures,
            )


if __name__ == "__main__":
    unittest.main()
