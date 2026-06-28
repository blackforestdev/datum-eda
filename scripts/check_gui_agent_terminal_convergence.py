#!/usr/bin/env python3
"""Guard that GUI agent entry points converge on the PTY terminal lane."""

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
MAIN = ROOT / "crates" / "gui-app" / "src" / "main.rs"
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


def main() -> int:
    failures: list[str] = []
    main = MAIN.read_text()
    bottom_dock = BOTTOM_DOCK.read_text()
    launcher = LAUNCHER.read_text()
    terminal_controls = TERMINAL_CONTROLS.read_text()
    runtime_terminal_context = RUNTIME_TERMINAL_CONTEXT.read_text()
    production_refresh = PRODUCTION_REFRESH.read_text()
    gui_protocol = GUI_PROTOCOL.read_text()

    for path in RETIRED_BRIDGE_FILES:
        if path.exists():
            failures.append(
                f"retired embedded assistant bridge artifact must not exist: {path.relative_to(ROOT)}"
            )

    if '"AGENTS"' not in bottom_dock:
        failures.append("bottom dock must label the agent entry as AGENTS")
    if '"ASSISTANT"' in bottom_dock:
        failures.append("bottom dock must not reintroduce an ASSISTANT tab label")
    if "HitTarget::AssistantTab => self.open_terminal_agent_launcher()" not in main:
        failures.append("AssistantTab hit target must route to open_terminal_agent_launcher()")
    if "DockTab::Terminal | DockTab::Assistant" not in bottom_dock:
        failures.append("DockTab::Assistant compatibility state must render the terminal lane")
    activity_handler = (
        "HitTarget::TerminalActivitySummary(summary) => {\n"
        "                self.set_active_dock(DockTab::Terminal);"
    )
    if activity_handler not in main:
        failures.append("terminal activity selection must focus DockTab::Terminal")
    forbidden_activity_handler = (
        "HitTarget::TerminalActivitySummary(summary) => {\n"
        "                self.set_active_dock(DockTab::Assistant);"
    )
    if forbidden_activity_handler in main:
        failures.append("terminal activity selection must not focus DockTab::Assistant")
    if "TERMINAL_AGENT_LAUNCHER_PREFILL" not in launcher:
        failures.append("agent launcher must remain a terminal prefill surface")
    if "self.set_active_dock(DockTab::Terminal)" not in launcher:
        failures.append("agent launcher must focus the terminal dock")
    if "self.write_terminal_bytes(TERMINAL_AGENT_LAUNCHER_PREFILL.as_bytes())" not in launcher:
        failures.append("agent launcher must write the prefill into the PTY terminal")
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
