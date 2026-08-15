#!/usr/bin/env python3
"""Guard that GUI agent entry points stay inside the PTY terminal lane."""

from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
MAIN = ROOT / "crates" / "gui-app" / "src" / "main.rs"
KEYBOARD_FOCUS = ROOT / "crates" / "gui-app" / "src" / "keyboard_focus.rs"
BOTTOM_DOCK = ROOT / "crates" / "gui-render" / "src" / "bottom_dock.rs"
LAUNCHER = ROOT / "crates" / "gui-app" / "src" / "terminal_agent_launcher.rs"
TERMINAL_CONTROLS = ROOT / "crates" / "gui-app" / "src" / "terminal_session_controls.rs"
RUNTIME_TERMINAL_CONTEXT = ROOT / "crates" / "gui-app" / "src" / "runtime_terminal_context.rs"
PRODUCTION_REFRESH = ROOT / "crates" / "gui-app" / "src" / "production_status_refresh.rs"
GUI_PROTOCOL = ROOT / "crates" / "gui-protocol" / "src" / "lib.rs"
TERMINAL_LANE = ROOT / "crates" / "gui-protocol" / "src" / "terminal_lane.rs"
TERMINAL_TRANSPORT = ROOT / "crates" / "gui-app" / "src" / "terminal_transport"
RETIRED_BRIDGE_FILES = [
    ROOT / "crates" / "gui-app" / "src" / "assistant_bridge.rs",
    ROOT / "scripts" / "datum_assistant_bridge.py",
]


def check_terminal_grid_writers(failures: list[str]) -> None:
    """Keep the public cross-crate grid gateway inside PTY interpretation."""
    declaration = Path("crates/gui-protocol/src/terminal_lane.rs")
    terminal_core = Path("crates/gui-app/src/terminal_screen")
    for path in sorted((ROOT / "crates").rglob("*.rs")):
        source = path.read_text(encoding="utf-8")
        if "pty_grid_mut(" not in source:
            continue
        relative = path.relative_to(ROOT)
        is_test = relative.name.endswith("_tests.rs")
        is_terminal_core = relative == Path("crates/gui-app/src/terminal_screen.rs") or (
            terminal_core in relative.parents
        )
        if relative != declaration and not is_test and not is_terminal_core:
            failures.append(
                f"terminal grid mutation escaped PTY interpretation: {relative}"
            )


def check_terminal_focus_reporting(
    main: str,
    authority: str,
    mutation_sources: str,
    failures: list[str],
) -> None:
    """Bind child focus reports to the keyboard-owner setter, not window focus."""
    window_marker = "            WindowEvent::Focused(focused) => {"
    setter_marker = "    pub(crate) fn set_keyboard_focus"
    if window_marker not in main or setter_marker not in authority:
        failures.append("terminal focus-report authority markers are missing")
        return
    window_body = main.split(window_marker, 1)[1].split("            WindowEvent::", 1)[0]
    setter_body = authority.split(setter_marker, 1)[1].split("\n    }", 1)[0]
    if "report_terminal_focus_event" in window_body:
        failures.append("OS window focus must not emit terminal focus-report bytes")
    if "terminal_focus_report_transition" not in setter_body:
        failures.append("keyboard-focus transitions must own terminal focus reporting")
    if "has_keyboard_focus" not in setter_body:
        failures.append("keyboard-focus setter must publish the terminal cursor focus projection")
    assignments = re.findall(r"self\.keyboard_focus\s*=(?!=)", mutation_sources)
    if len(assignments) != 1:
        failures.append("keyboard focus must mutate only through set_keyboard_focus")
    projection_assignments = re.findall(r"\.has_keyboard_focus\s*=(?!=)", mutation_sources)
    if len(projection_assignments) != 1:
        failures.append("terminal cursor focus projection must mutate only with keyboard focus")


def check_workspace_hotkey_timing(authority: str, failures: list[str]) -> None:
    """Keep editor actions on initial Press and outside terminal ownership."""
    if "workspace_action_should_fire(focus, dock_visible, event.state, event.repeat)" not in authority:
        failures.append("workspace hotkeys must use the focus-aware initial-Press predicate")
    timing_body = authority.split("pub(crate) fn workspace_action_should_fire", 1)
    if len(timing_body) != 2:
        return
    timing_body = timing_body[1].split("\n}", 1)[0]
    for marker in ("ElementState::Pressed", "!repeat", "KeyClass::WorkspaceHotkey"):
        if marker not in timing_body:
            failures.append(f"workspace hotkey timing predicate is missing {marker}")
    dispatch_tail = authority.split("// Pane focus cycling", 1)
    if len(dispatch_tail) == 2:
        dispatch_tail = dispatch_tail[1].split("    if escape_released", 1)[0]
        if dispatch_tail.count("workspace_action_pressed") != 2:
            failures.append("pane and character hotkeys must share the Press predicate")
        if "ElementState::Released" in dispatch_tail:
            failures.append("workspace hotkey dispatch must not fire on key release")


def check_terminal_input_identity(
    terminal_lane: str,
    production_sources: str,
    failures: list[str],
) -> None:
    """Keep shell, screen-cursor, and chrome-rename state distinguishable."""
    for marker in (
        "pub rename_input: String",
        "pub rename_cursor: usize",
        "pub screen_cursor_row: usize",
        "pub screen_cursor_col: usize",
    ):
        if marker not in terminal_lane:
            failures.append(f"terminal input classification is missing {marker}")
    if re.search(r"pub\s+(?:input|cursor)\s*:", terminal_lane):
        failures.append("terminal protocol must not expose generic input/cursor fields")
    for marker in (".terminal.input", ".terminal.cursor"):
        if marker in production_sources:
            failures.append(f"terminal production code must not use generic {marker}")
    for marker in (
        "fn dock_accepts_text_input",
        "fn append_dock_text",
        "fn dock_tab_accepts_edit",
        "fn current_dock_input",
        "fn current_dock_input_mut",
        "fn backspace_dock_input",
        "fn move_dock_cursor",
        "fn move_dock_cursor_to_edge",
        "fn complete_dock_input",
        "fn submit_dock_input",
        "KeyClass::DockLineEdit",
        "KeyClass::EscapeWithEmptyInput",
    ):
        if marker in production_sources:
            failures.append(f"terminal chrome editor must not use generic marker {marker}")
    rename_marker = "fn append_terminal_rename_text"
    if rename_marker not in production_sources:
        failures.append("terminal rename editor must expose an explicit text boundary")
    else:
        rename_body = production_sources.split(rename_marker, 1)[1].split("\n    fn ", 1)[0]
        if "write_foreign_shell_bytes" in rename_body:
            failures.append("terminal rename text must never reach the foreign shell")


def check_terminal_input_mode(
    production_sources: str,
    bottom_dock: str,
    failures: list[str],
) -> None:
    """Require one exclusive attached/rename/detached input-mode authority."""
    for marker in (
        "enum TerminalInputOwner",
        "AttachedPty",
        "RenameChrome",
        "DetachedReadOnly",
        "fn terminal_input_owner",
        "fn commit_terminal_ime_text",
        "fn write_attached_terminal_bytes",
        "HitTarget::TerminalSessionReattachActive",
    ):
        if marker not in production_sources:
            failures.append(f"terminal input-mode authority is missing {marker}")
    for marker in (
        "LegacyDockLineEdit",
        "complete_terminal_rename_input",
        "terminal_rename_editor_active",
    ):
        if marker in production_sources:
            failures.append(f"dead terminal line-edit marker must not remain: {marker}")
    for marker in ("\"REATTACH\"", "TerminalSessionReattachActive"):
        if marker not in bottom_dock:
            failures.append(f"detached terminal chrome is missing {marker}")
    mode_marker = "pub(crate) fn terminal_input_owner"
    if mode_marker in production_sources:
        mode_body = production_sources.split(mode_marker, 1)[1].split("\n}", 1)[0]
        for marker in ("RenameChrome", "AttachedPty", "DetachedReadOnly"):
            if marker not in mode_body:
                failures.append(f"terminal input owner does not classify {marker}")
    writer_marker = "fn write_attached_terminal_bytes"
    if writer_marker in production_sources:
        writer_body = production_sources.split(writer_marker, 1)[1].split("\n}", 1)[0]
        for marker in ("active_attached", "write_bytes"):
            if marker not in writer_body:
                failures.append(f"attached terminal byte gate is missing {marker}")


def check_terminal_transport_boundary(
    production_sources: str,
    transport: str,
    failures: list[str],
) -> None:
    """Keep Linux PTY ownership inside Datum and outside terminal semantics."""
    for marker in (
        "posix_openpt",
        "grantpt",
        "unlockpt",
        "ptsname_r",
        "TIOCSCTTY",
        "configure_child_pty",
    ):
        if marker not in transport:
            failures.append(f"Datum PTY boundary is missing {marker}")
        elif production_sources.count(marker) != transport.count(marker):
            failures.append(f"Datum PTY ownership escaped terminal_transport/: {marker}")
    if "TIOCSWINSZ" not in production_sources:
        failures.append("Datum PTY resize ownership is missing TIOCSWINSZ")
    for marker in ("TerminalScreen", "TerminalLaneState", "pty_grid_mut", "apply_bytes"):
        if marker in transport:
            failures.append(f"terminal transport must not own cell/core marker {marker}")
    for marker in (
        "libghostty",
        "alacritty_terminal",
        "portable_pty",
        "portable-pty",
    ):
        if marker in production_sources:
            failures.append(f"third-party terminal dependency must not remain: {marker}")


def main() -> int:
    failures: list[str] = []
    check_terminal_grid_writers(failures)
    main = MAIN.read_text()
    keyboard_focus = KEYBOARD_FOCUS.read_text()
    focus_mutation_sources = "\n".join(
        path.read_text(encoding="utf-8")
        for path in sorted((ROOT / "crates/gui-app/src").rglob("*.rs"))
        if not path.name.endswith("_tests.rs")
    )
    bottom_dock = BOTTOM_DOCK.read_text()
    launcher = LAUNCHER.read_text() if LAUNCHER.exists() else ""
    terminal_controls = TERMINAL_CONTROLS.read_text()
    runtime_terminal_context = RUNTIME_TERMINAL_CONTEXT.read_text()
    production_refresh = PRODUCTION_REFRESH.read_text()
    gui_protocol = GUI_PROTOCOL.read_text()
    terminal_lane = TERMINAL_LANE.read_text()
    terminal_transport = "\n".join(
        path.read_text(encoding="utf-8")
        for path in sorted(TERMINAL_TRANSPORT.rglob("*.rs"))
    )
    check_terminal_focus_reporting(main, keyboard_focus, focus_mutation_sources, failures)
    check_workspace_hotkey_timing(keyboard_focus, failures)
    check_terminal_input_identity(terminal_lane, focus_mutation_sources, failures)
    check_terminal_input_mode(focus_mutation_sources, bottom_dock, failures)
    check_terminal_transport_boundary(focus_mutation_sources, terminal_transport, failures)

    raw_write_marker = "    fn write_foreign_shell_bytes"
    if raw_write_marker not in main:
        failures.append("foreign-shell byte writer must remain an explicit runtime boundary")
    else:
        raw_write_body = main.split(raw_write_marker, 1)[1].split("\n    fn ", 1)[0]
        if "write_attached_terminal_bytes" not in raw_write_body:
            failures.append("foreign-shell writer must use the attached-session byte gate")
        if "mark_terminal_" in raw_write_body or "refresh_pending" in raw_write_body:
            failures.append(
                "raw foreign-shell input must not infer mutation or schedule workspace refresh"
            )
    authoring_marker = "    fn queue_authoring_terminal_handoff"
    if authoring_marker not in main or "self.mark_terminal_workspace_refresh_pending();" not in main.split(
        authoring_marker, 1
    )[1].split("\n    fn ", 1)[0]:
        failures.append("typed authoring handoffs must explicitly schedule workspace refresh")

    for path in RETIRED_BRIDGE_FILES:
        if path.exists():
            failures.append(
                f"retired embedded assistant bridge artifact must not exist: {path.relative_to(ROOT)}"
            )

    if '"terminal"' not in bottom_dock:
        failures.append("bottom dock must render the terminal tab")
    for forbidden_label in ('"AGENTS"', '"ASSISTANT"', '"OUTPUT"'):
        if forbidden_label in bottom_dock:
            failures.append(f"bottom dock must not render {forbidden_label} as a dock tab")
    for marker in ("AssistantTab", "OutputsTab", "DockTab::Assistant", "DockTab::Outputs"):
        if marker in main or marker in bottom_dock or marker in gui_protocol:
            failures.append(f"terminal-only dock must not contain {marker}")
    # T0-C02 removes application activity summaries from the foreign-shell
    # viewport entirely. The data remains available to the future Command
    # Console, but must not reclaim terminal rows or a terminal hit target.
    for surface, source in (
        ("GUI runtime", main),
        ("bottom dock", bottom_dock),
        ("GUI protocol", gui_protocol),
    ):
        if "TerminalActivitySummary" in source:
            failures.append(
                f"{surface} must not expose a terminal activity-summary surface"
            )
    if "mod terminal_agent_launcher;" in main:
        failures.append("agent launcher module must not be wired as a dock tab surface")
    forbidden_runtime_markers = [
        "AssistantLaneState",
        "AssistantMessage",
        "assistant: AssistantLaneState",
        "mod assistant_bridge;",
        "spawn_assistant_session",
        "AssistantSession",
        "AssistantBridgeInput",
        "poll_assistant_output",
        "send_assistant_message",
        "sync_assistant_context",
        "push_assistant_message",
        "submit_assistant_input",
        "complete_assistant_input",
        "handle_assistant_meta_command",
        "ui.assistant.input",
        "ui.assistant.transcript",
    ]
    for marker in forbidden_runtime_markers:
        if marker in main or marker in production_refresh:
            failures.append(f"GUI runtime must not own embedded assistant bridge marker {marker!r}")
    forbidden_protocol_markers = [
        "AssistantLaneState",
        "AssistantMessage",
        "assistant: AssistantLaneState",
    ]
    for marker in forbidden_protocol_markers:
        if marker in gui_protocol:
            failures.append(f"GUI protocol must not own assistant lane marker {marker!r}")
    forbidden_render_markers = [
        "render_assistant_lane",
        "state.ui.assistant",
        "ui.assistant",
    ]
    for marker in forbidden_render_markers:
        if marker in bottom_dock:
            failures.append(f"GUI render must not own assistant lane marker {marker!r}")
    close_refresh_marker = (
        "close_active(&mut self.session.workspace_mut().ui.terminal)\n"
        "        {\n"
        "            Ok(()) => {\n"
        "                self.refresh_terminal_context_snapshot();"
    )
    if close_refresh_marker not in terminal_controls:
        failures.append(
            "closing the active terminal tab must refresh the surviving session context alias"
        )
    if "self.terminal_launch_context = context;" not in runtime_terminal_context:
        failures.append(
            "terminal context refresh must update Runtime.terminal_launch_context for future tabs/restarts"
        )

    if failures:
        print("GUI agent/terminal convergence guard failed:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print("GUI agent/terminal convergence guard passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
