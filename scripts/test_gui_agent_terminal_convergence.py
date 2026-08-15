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
    def test_terminal_input_mode_is_exclusive_and_reattachable(self) -> None:
        production = """
enum TerminalInputOwner { AttachedPty, RenameChrome, DetachedReadOnly }
pub(crate) fn terminal_input_owner() {
    use TerminalInputOwner::{AttachedPty, RenameChrome, DetachedReadOnly};
}
fn commit_terminal_ime_text() {}
HitTarget::TerminalSessionReattachActive
"""
        bottom_dock = '("REATTACH", HitTarget::TerminalSessionReattachActive)'
        failures: list[str] = []
        guard.check_terminal_input_mode(production, bottom_dock, failures)
        self.assertEqual([], failures)

        failures = []
        guard.check_terminal_input_mode(
            production.replace("DetachedReadOnly", "LegacyDockLineEdit"),
            "",
            failures,
        )
        self.assertIn(
            "legacy dock line-edit routing must not remain in production", failures
        )
        self.assertIn(
            "detached terminal chrome is missing \"REATTACH\"", failures
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
    pub(crate) fn set_keyboard_focus(&mut self) {
        let report = keyboard_focus::terminal_focus_report_transition(old, next);
        self.keyboard_focus = next;
        self.workspace.ui.terminal.has_keyboard_focus = next.is_terminal();
    }
    fn after() {}
"""
        window = """
            WindowEvent::Focused(focused) => { if !focused {} }
            WindowEvent::CursorLeft { .. } => {}
"""
        failures: list[str] = []
        guard.check_terminal_focus_reporting(window, valid, valid, failures)
        self.assertEqual([], failures)
        invalid_window = window.replace(
            "if !focused {}", "self.report_terminal_focus_event(focused);"
        )
        invalid_authority = valid.replace(
            "self.keyboard_focus = next;",
            "self.keyboard_focus = next;\nself.keyboard_focus = old;\n"
            "self.workspace.ui.terminal.has_keyboard_focus = false;",
        )
        guard.check_terminal_focus_reporting(
            invalid_window, invalid_authority, invalid_authority, failures
        )
        self.assertIn("OS window focus must not emit terminal focus-report bytes", failures)
        self.assertIn("keyboard focus must mutate only through set_keyboard_focus", failures)
        self.assertIn(
            "terminal cursor focus projection must mutate only with keyboard focus", failures
        )

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
