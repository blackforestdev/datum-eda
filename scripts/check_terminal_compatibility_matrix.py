#!/usr/bin/env python3
"""Validate the governed DTC-P28 compatibility matrix and production witness."""

from __future__ import annotations

import json
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
MATRIX = ROOT / "specs/terminal_compatibility_matrix.json"
TEST = ROOT / "crates/gui-app/src/terminal_compatibility_tests.rs"
REQUIRED_WITNESSES = {
    "bash", "zsh", "fish", "ssh", "tmux", "less", "vim", "neovim",
    "htop", "btop", "python", "git", "cargo",
}
REQUIRED_DOMAINS = {
    "VT/DEC and color", "Unicode 17 text", "modern protocols", "graphics",
    "production adapter",
}


def failures(root: pathlib.Path = ROOT) -> list[str]:
    matrix_path = root / MATRIX.relative_to(ROOT)
    test_path = root / TEST.relative_to(ROOT)
    problems: list[str] = []
    try:
        matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        return [f"cannot read DTC-P28 matrix: {error}"]
    test = test_path.read_text(encoding="utf-8") if test_path.is_file() else ""
    if matrix.get("schema") != "datum-terminal-compatibility-matrix-v1":
        problems.append("DTC-P28 matrix schema is missing or changed")
    if matrix.get("frontier_step") != "DTC-P28":
        problems.append("DTC-P28 matrix is not bound to its Frontier step")
    witnesses = {entry.get("name") for entry in matrix.get("external_witnesses", [])}
    missing = REQUIRED_WITNESSES - witnesses
    extra = witnesses - REQUIRED_WITNESSES
    if missing:
        problems.append("DTC-P28 matrix lacks witnesses: " + ", ".join(sorted(missing)))
    if extra:
        problems.append("DTC-P28 matrix has ungoverned witnesses: " + ", ".join(sorted(extra)))
    domains = {entry.get("domain") for entry in matrix.get("datum_authored_standards_probes", [])}
    if domains != REQUIRED_DOMAINS:
        problems.append("DTC-P28 standards domains do not match the governed set")
    optional = {entry.get("name"): entry for entry in matrix.get("optional_external_conformance", [])}
    for name in ("vttest", "esctest2"):
        entry = optional.get(name, {})
        if "not counted as normative parity" not in entry.get("status", ""):
            problems.append(f"{name} must remain an honest optional black-box witness")
    markers = [
        "production_pty_proves_named_shell_tui_and_tool_compatibility",
        "TerminalCoreSessionAdapter::new_with_profile",
        "spawn_terminal_session(&context)",
        "DATUM_P28_TOOL_ROOT",
        "assert_eq!(results.len(), 13)",
    ]
    for marker in markers:
        if marker not in test:
            problems.append(f"DTC-P28 production witness lacks marker: {marker}")
    return problems


def main() -> int:
    problems = failures()
    if problems:
        for problem in problems:
            print(f"terminal compatibility matrix: {problem}", file=sys.stderr)
        return 1
    print("terminal compatibility matrix passed (13 real programs; 5 Datum-authored standards domains).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
