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
    (
        # The real, FAILING visual-parity gate: capture the running app at the
        # canonical command + curated demo scene + fixed window and diff it
        # against the owner-approved shell golden. This replaces the former
        # HUMAN "paperwork" disposition that reported but never failed.
        "shell visual parity (owner-approved golden)",
        [sys.executable, "scripts/check_gui_visual_parity.py"],
    ),
]

# Visual parity is now an ENFORCED regression gate (shell visual parity above),
# not an owner-eye disposition that this aggregate merely reported. The one
# remaining human step is the OUT-OF-BAND approval of the golden itself (a human
# judges the app-vs-prototype match once, then blesses the golden); after that,
# drift from the approved look is a machine failure.
HUMAN_SIGNOFF_LINES = [
    "The shell golden (crates/gui-render/testdata/golden/shell/datum-shell.golden.png) is",
    "owner-approved to match docs/gui/prototypes/board-editor.html; the visual-parity gate",
    "then FAILS on any regression from that approved look (re-approve via --bless).",
]


def main() -> int:
    for name, command in GATES:
        print(f"GUI-CONFORMANCE {name}: {' '.join(command)}")
        subprocess.run(command, cwd=ROOT, check=True)
    for line in HUMAN_SIGNOFF_LINES:
        print(f"GUI-CONFORMANCE NOTE: {line}")
    print("GUI conformance gate passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
