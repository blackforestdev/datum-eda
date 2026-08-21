#!/usr/bin/env python3
"""Enforce DTC-P25's bounded, test-only shadow-comparison boundary."""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SRC = Path("crates/gui-app/src")
ADAPTER = SRC / "terminal_core_adapter.rs"
SHADOW = SRC / "terminal_shadow.rs"
PRODUCTION_OWNERS = (
    SRC / "main.rs",
    SRC / "terminal_session.rs",
    SRC / "terminal_session_drain.rs",
    SRC / "terminal_session_render.rs",
    SRC / "terminal_session_spawn.rs",
)

REQUIRED_PROOFS = (
    "dtc_p25_recorded_overlap_matches_whole_recorded_and_arbitrary_chunks",
    "dtc_p25_non_overlap_uses_terminal_core_normative_unicode_and_link_proof",
    "dtc_p25_shadow_is_bounded_and_has_no_production_selector",
)

REQUIRED_COMPARISON_MARKERS = (
    "struct DeclaredOverlap",
    "TerminalScreen::default()",
    "TerminalCoreSessionAdapter::new(",
    "assert_recorded_boundaries_match(recording)",
    'assert_recording_matches(recording, "whole"',
    'assert_recording_matches(recording, "recorded PTY chunks"',
    'assert_recording_matches(recording, "bytewise"',
    'assert_recording_matches(recording, "seeded irregular"',
)

PINNED_BOUNDS = (
    "const MAX_SHADOW_RECORDING_BYTES: usize = 64 * 1024;",
    "const MAX_SHADOW_REPLAY_CHUNKS: usize = 4_096;",
    "const MAX_SHADOW_RECORDINGS: usize = 16;",
)


def read(root: Path, relative: Path) -> str:
    path = root / relative
    return path.read_text(encoding="utf-8") if path.is_file() else ""


def check(root: Path) -> list[str]:
    failures: list[str] = []
    adapter = read(root, ADAPTER)
    shadow = read(root, SHADOW)

    registration = re.compile(
        r'#\[cfg\(test\)\]\s*#\[path\s*=\s*"terminal_shadow\.rs"\]\s*mod\s+shadow\s*;'
    )
    if not registration.search(adapter):
        failures.append("DTC-P25 shadow module must be registered under exact cfg(test)")
    if "cfg(debug_assertions)" in adapter or "cfg!(debug_assertions)" in adapter:
        failures.append("DTC-P25 shadow must not enter debug production binaries")

    for marker in REQUIRED_PROOFS:
        if shadow.count(marker) != 1:
            failures.append(f"DTC-P25 shadow proof missing or duplicated: {marker}")
    for marker in REQUIRED_COMPARISON_MARKERS:
        if marker not in shadow:
            failures.append(f"DTC-P25 comparison structure missing: {marker}")
    for marker in PINNED_BOUNDS:
        if shadow.count(marker) != 1:
            failures.append(f"DTC-P25 test bound missing or drifted: {marker}")

    forbidden_shadow_io = (
        "std::process::Command",
        "std::fs::",
        "std::net::",
        "UnixStream",
        "TcpStream",
        "mpsc::channel",
        "thread::spawn",
    )
    for marker in forbidden_shadow_io:
        if marker in shadow:
            failures.append(f"DTC-P25 shadow must replay in memory without I/O: {marker}")

    for relative in PRODUCTION_OWNERS:
        source = read(root, relative)
        for marker in ("terminal_shadow", "DeclaredOverlap", "shadow::"):
            if marker in source:
                failures.append(
                    f"production owner {relative} must not reference DTC-P25 shadow: {marker}"
                )
    return failures


def main() -> int:
    failures = check(ROOT)
    if failures:
        print("terminal shadow boundary: FAIL")
        for failure in failures:
            print(f"- {failure}")
        return 1
    print("terminal shadow boundary: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
