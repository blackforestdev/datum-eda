#!/usr/bin/env python3
"""Aggregate machine-checkable Datum GUI conformance gates."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

GATES = [
    (
        "token/value/prototype parity",
        [sys.executable, "scripts/check_gui_design_tokens.py"],
    ),
    (
        "menu model manifest",
        [sys.executable, "scripts/check_menu_model.py"],
    ),
    (
        "menu model csv parity",
        [sys.executable, "scripts/menu_model_csv.py", "check"],
    ),
    (
        "GUI render conformance tests",
        [
            "cargo",
            "test",
            "-p",
            "datum-gui-render",
            "conformance_",
            "--lib",
            "--features",
            "visual",
            "--",
            "--nocapture",
        ],
    ),
]

HUMAN_SIGNOFF_LINES = [
    "HUMAN rows are reviewed against docs/gui/reference/board-editor.png and committed datum-test goldens.",
    "This gate reports HUMAN rows but does not fail on owner-eye disposition.",
]


def main() -> int:
    for name, command in GATES:
        print(f"GUI-CONFORMANCE {name}: {' '.join(command)}")
        subprocess.run(command, cwd=ROOT, check=True)
    for line in HUMAN_SIGNOFF_LINES:
        print(f"GUI-CONFORMANCE HUMAN: {line}")
    print("GUI conformance gate passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
