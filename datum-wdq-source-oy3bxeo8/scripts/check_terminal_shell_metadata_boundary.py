#!/usr/bin/env python3
"""Guard OSC 7/133 as useful but non-authoritative session metadata."""

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
LANE = ROOT / "crates/gui-protocol/src/terminal_lane.rs"
REDUCER = ROOT / "crates/gui-app/src/terminal_session_core_events.rs"
EVENTS = ROOT / "crates/gui-app/src/terminal_session_events.rs"
TESTS = ROOT / "crates/gui-app/src/terminal_session_drain_tests.rs"


def check_sources(
    lane: str, reducer: str, events: str, tests: str
) -> list[str]:
    failures: list[str] = []
    for marker in (
        "TerminalShellMetadata",
        "TerminalShellPhase",
        "event_sequence.saturating_add(1)",
        "shell_metadata",
    ):
        if marker not in lane:
            failures.append(f"terminal shell metadata projection is missing {marker}")
    for marker in (
        "CoreEvent::WorkingDirectoryChanged",
        "CoreEvent::ShellMark",
        "lane.shell_metadata.observe",
        "record_terminal_shell_metadata_event",
    ):
        if marker not in reducer:
            failures.append(f"terminal OSC metadata integration is missing {marker}")
    for marker in (
        'event: "terminal_shell_metadata"',
        'origin: "pty_osc"',
        'trust: "untrusted"',
    ):
        if marker not in events:
            failures.append(f"terminal OSC metadata audit boundary is missing {marker}")
    proof = "osc7_and_osc133_are_untrusted_session_metadata_not_design_authority"
    if proof not in tests:
        failures.append("terminal OSC metadata authority proof is missing")
    for forbidden in (
        "prepare_terminal_command_execution",
        "record_manual_terminal_command_handoff",
        "mark_terminal_workspace_refresh_pending",
        "DesignModel",
        "Operation",
        "approval",
    ):
        if forbidden in reducer:
            failures.append(f"untrusted OSC metadata gained authority marker {forbidden}")
    return failures


def main() -> int:
    failures = check_sources(
        LANE.read_text(encoding="utf-8"),
        REDUCER.read_text(encoding="utf-8"),
        EVENTS.read_text(encoding="utf-8"),
        TESTS.read_text(encoding="utf-8"),
    )
    if failures:
        print("Terminal shell metadata boundary failed:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    print("Terminal shell metadata boundary passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
