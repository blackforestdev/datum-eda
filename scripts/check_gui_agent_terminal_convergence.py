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
    assignments = re.findall(r"self\.keyboard_focus\s*=(?!=)", mutation_sources)
    if len(assignments) != 1:
        failures.append("keyboard focus must mutate only through set_keyboard_focus")


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
    check_terminal_focus_reporting(main, keyboard_focus, focus_mutation_sources, failures)

    raw_write_marker = "    fn write_foreign_shell_bytes"
    if raw_write_marker not in main:
        failures.append("foreign-shell byte writer must remain an explicit runtime boundary")
    else:
        raw_write_body = main.split(raw_write_marker, 1)[1].split("\n    fn ", 1)[0]
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
