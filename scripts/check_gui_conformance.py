#!/usr/bin/env python3
"""Aggregate machine-checkable Datum GUI conformance gates."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CARGO_GUARD = [
    sys.executable,
    "scripts/run_cargo_guarded.py",
    "--workload",
    "proof",
    "--",
]

GATES = [
    (
        "output-only Datum Console boundary",
        [sys.executable, "scripts/check_datum_console_boundary.py"],
    ),
    (
        "Console typed state and producer routing tests",
        CARGO_GUARD
        + [
            "cargo",
            "test",
            "-p",
            "datum-gui-protocol",
            "-p",
            "datum-gui-app",
            "console_",
        ],
    ),
    (
        "Console render behavior tests",
        CARGO_GUARD
        + [
            "cargo",
            "test",
            "-p",
            "datum-gui-render",
            "datum_console",
            "--lib",
            "--features",
            "visual",
        ],
    ),
    (
        "Console exact visual goldens",
        CARGO_GUARD
        + [
            "cargo",
            "test",
            "-p",
            "datum-gui-render",
            "--test",
            "visual_goldens",
            "--features",
            "visual",
            "console_visual_goldens_match",
            "--",
            "--nocapture",
        ],
    ),
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
        CARGO_GUARD
        + [
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
        # canonical command (datum-test board + --select R1, fixed 1680x1050) and
        # diff it against the committed shell golden. This is a single-pane INTERIM
        # target — NOT owner-approved against board-editor.html; the full split
        # Board+Schematic composition is a Phase-2 target. It replaces the former
        # HUMAN "paperwork" disposition that reported but never failed.
        "shell visual parity (single-pane interim golden)",
        [sys.executable, "scripts/check_gui_visual_parity.py"],
    ),
]

# Visual parity is now an ENFORCED regression gate, not an owner-eye disposition
# this aggregate merely reported. The former non-failing HUMAN_SIGNOFF note (which
# printed "reports HUMAN rows but does not fail on owner-eye disposition") was the
# defect in prose form and has been DELETED. The golden-exists and *.PENDING-shadow
# checks — the anti-paperwork teeth — are delegated entirely to the shell
# visual-parity gate above (it FAILS if the interim shell golden is absent or a
# PENDING placeholder shadows it), and because GATES run under check=True, that
# failure makes this aggregate exit non-zero. The one remaining human step is the
# OUT-OF-BAND one-time approval of the golden itself (--bless).


def main() -> int:
    for name, command in GATES:
        print(f"GUI-CONFORMANCE {name}: {' '.join(command)}")
        subprocess.run(command, cwd=ROOT, check=True)
    print("GUI conformance gate passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
