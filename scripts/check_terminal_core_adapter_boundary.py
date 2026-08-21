#!/usr/bin/env python3
"""Enforce the single production PTY-to-TerminalCore session boundary."""

from __future__ import annotations

import sys
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
APP = Path("crates/gui-app")
SRC = APP / "src"
ADAPTER = SRC / "terminal_core_adapter.rs"
SESSION = SRC / "terminal_session.rs"
SESSION_RENDER = SRC / "terminal_session_render.rs"
DRAIN = SRC / "terminal_session_drain.rs"
SPAWN = SRC / "terminal_session_spawn.rs"
TERMINAL_PROCESS = SRC / "terminal_process.rs"
TERMINAL_LANE = Path("crates/gui-protocol/src/terminal_lane.rs")
RETIRED_AUTHORITIES = (
    SRC / "terminal_screen.rs",
    SRC / "terminal_screen",
    SRC / "terminal_shadow.rs",
    Path("crates/gui-render/src/terminal_color.rs"),
    Path("crates/gui-render/src/terminal_cursor.rs"),
)

APPROVED_LIMITS = {
    "parameter_count": "64",
    "parameter_digits": "16",
    "parameter_value": "1_000_000",
    "subparameter_count": "64",
    "intermediate_bytes": "16",
    "control_string_bytes": "16 * 1024 * 1024",
    "cluster_bytes": "4_096",
    "title_bytes": "32_768",
    "working_directory_bytes": "65_536",
    "clipboard_bytes": "4 * 1024 * 1024",
    "hyperlink_bytes": "1024 * 1024",
    "input_bytes": "4 * 1024 * 1024",
    "keyboard_stack": "32",
    "notification_bytes": "65_536",
    "reply_bytes": "65_536",
    "pending_events": "4_096",
    "pending_damage": "4_096",
    "history_lines": "100_000",
    "history_bytes": "64 * 1024 * 1024",
    "graphic_objects": "256",
    "graphic_pixels": "16_777_216",
    "graphic_decoded_bytes": "64 * 1024 * 1024",
    "graphic_frames": "1_024",
    "compression_ratio": "1_024",
    "parser_work": "67_108_864",
    "search_work": "67_108_864",
    "reflow_work": "67_108_864",
    "screen_cells": "1_048_576",
    "snapshot_cells": "33_554_432",
}

REQUIRED_PROOFS = (
    "production_profile_matches_the_owner_approved_p22_l1_values",
    "adapter_keeps_session_context_identity_and_projects_core_state",
    "terminal_replies_are_emitted_once_and_never_enter_terminal_cells",
    "adapters_isolate_output_modes_resize_and_context",
    "stream_finish_repairs_incomplete_utf8_before_lifecycle_completion",
    "repeated_bells_remain_bounded_but_preserve_visible_count",
    "render_state_preserves_surface_pixels_and_consumes_damage_once",
)


def read(root: Path, relative: Path) -> str:
    path = root / relative
    return path.read_text(encoding="utf-8") if path.is_file() else ""


def check(root: Path) -> list[str]:
    failures: list[str] = []
    manifest = read(root, APP / "Cargo.toml")
    adapter = read(root, ADAPTER)
    session = read(root, SESSION)
    session_render = read(root, SESSION_RENDER)
    drain = read(root, DRAIN)
    spawn = read(root, SPAWN)
    tests = read(root, SRC / "terminal_core_adapter_tests.rs")
    terminal_lane = read(root, TERMINAL_LANE)
    terminal_process = read(root, TERMINAL_PROCESS)

    for retired in RETIRED_AUTHORITIES:
        candidate = root / retired
        if candidate.is_file() or (candidate.is_dir() and any(candidate.rglob("*"))):
            failures.append(f"retired provisional terminal authority returned: {retired}")
    for marker in ("PtyGrid", "grid_lines", "grid_styled_lines", "pty_grid_mut"):
        if marker in terminal_lane:
            failures.append(f"gui-protocol terminal lane regained screen authority: {marker}")
    for marker in ('.env("TERM", "xterm-256color")', '.env("COLORTERM", "truecolor")'):
        if marker in terminal_process:
            failures.append(f"retired terminal capability overclaim returned: {marker}")

    if 'datum-terminal-core = { path = "../terminal-core" }' not in manifest:
        failures.append("gui-app must consume TerminalCore through the local path crate")
    for forbidden in ("alacritty_terminal", "portable-pty", "ghostty", "libghostty"):
        if forbidden in manifest:
            failures.append(f"gui-app contains forbidden terminal dependency: {forbidden}")

    for marker in (
        "struct TerminalCoreSessionAdapter",
        "parser: StreamingParser",
        "core: TerminalCore",
        "PRODUCTION_CORE_LIMIT_VALUES",
        "parser.feed(bytes",
        "core.apply(action)",
        ".render_snapshot()",
    ):
        if marker not in adapter:
            failures.append(f"production TerminalCore adapter lacks ownership marker: {marker}")
    parser_fields = re.findall(r"(?m)^\s*\w*parser\w*:\s*StreamingParser,", adapter)
    core_fields = re.findall(r"(?m)^\s*\w*core\w*:\s*TerminalCore,", adapter)
    if len(parser_fields) != 1 or len(core_fields) != 1:
        failures.append("session adapter must own exactly one parser and one TerminalCore")

    for field, value in APPROVED_LIMITS.items():
        marker = f"\n    {field}: {value},"
        if adapter.count(marker) != 1:
            failures.append(f"owner-approved P22 limit drifted: {marker}")

    for marker in (
        "core: TerminalCoreSessionAdapter",
        ".resize(cols, rows, pixel_width, pixel_height)?",
        "slot.core = TerminalCoreSessionAdapter::new(",
    ):
        if marker not in session + session_render:
            failures.append(f"session registry lacks TerminalCore ownership marker: {marker}")
    if "screen: TerminalScreen" in session:
        failures.append("production session slot must not retain the provisional TerminalScreen")
    if "TerminalCoreSessionAdapter::new(" not in spawn:
        failures.append("every spawned session must receive its own TerminalCore adapter")
    if "TerminalScreen" in adapter:
        failures.append("the production TerminalCore adapter must not call the provisional parser")

    ordered = (
        "debug_assert_eq!(slot.core.session_id(), slot.session.session_id())",
        "debug_assert_eq!(slot.core.context_id(), slot.session.context_id)",
        "slot.core.apply_output(lane, bytes)",
        "session.write_bytes(&response)",
        "slot.core.finish(lane)",
    )
    for marker in ordered:
        if marker not in drain:
            failures.append(f"PTY/core lifecycle boundary lacks marker: {marker}")
    if "apply_bytes_with_responses" in drain:
        failures.append("production drain must not feed the provisional parser")
    if drain.find(ordered[0]) > drain.find(ordered[2]):
        failures.append("session/context identity must be checked before applying PTY output")

    for proof in REQUIRED_PROOFS:
        if proof not in tests:
            failures.append(f"TerminalCore adapter lacks governed proof: {proof}")
    return failures


def main() -> int:
    failures = check(ROOT)
    if failures:
        print("terminal core adapter boundary: FAIL")
        for failure in failures:
            print(f"- {failure}")
        return 1
    print("terminal core adapter boundary: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
