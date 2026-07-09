#!/usr/bin/env python3
"""Failing GUI visual-parity gate (same-engine app-screenshot regression).

This is the honest machine gate for Phase-1 visual parity. A Rust/wgpu render
will never pixel-match the HTML prototype, so this gate does NOT diff the build
against ``docs/gui/prototypes/board-editor.html``. Instead it:

  1. captures the running app at a CANONICAL command + curated demo scene +
     fixed window size (build-vs-build, one renderer), and
  2. diffs that capture against a COMMITTED, owner-approved app-screenshot
     golden with a small tolerance.

The golden is owner-approved to match ``board-editor.html`` (a human judges the
app-vs-prototype match once); this gate then FAILS on any regression from that
approved look. That converts visual parity from paperwork (a HUMAN row that was
never enforced) into a real failing regression gate.

Usage::

    python3 scripts/check_gui_visual_parity.py           # verify (fails on drift)
    python3 scripts/check_gui_visual_parity.py --bless    # re-approve the golden

``--bless`` re-captures and overwrites the golden; it is an explicit owner
action (the same role the datum-test board goldens have), never run in CI.
"""

from __future__ import annotations

import argparse
import subprocess
import sys
import tempfile
from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]

# The single canonical capture: curated --demo-known-good scene, fixed window.
WINDOW_SIZE = "1680x1050"
GOLDEN = ROOT / "crates/gui-render/testdata/golden/shell/datum-shell.golden.png"

# Small tolerance: absorbs sub-pixel AA / font-raster jitter, but any real chrome
# change (a moved panel, a restyled band, a token shift) moves far more than this.
PER_PIXEL_CHANNEL_TOLERANCE = 24  # 0..255 per-channel delta ignored below this
MAX_DIFFERING_PX_PCT = 1.5        # fail if more than this share of pixels differ


def capture(out_path: Path) -> None:
    command = [
        "cargo",
        "run",
        "-q",
        "-p",
        "datum-gui-app",
        "--bin",
        "datum-gui",
        "--features",
        "visual",
        "--",
        "--demo-known-good",
        "--visual-test",
        "--screenshot-out",
        str(out_path),
        "--window-size",
        WINDOW_SIZE,
        "--exit-after-screenshot",
    ]
    print(f"GUI-VISUAL-PARITY capture: {' '.join(command)}")
    subprocess.run(command, cwd=ROOT, check=True)
    if not out_path.is_file():
        raise SystemExit(
            f"capture did not produce {out_path} — the canonical command failed"
        )


def diff(golden: Path, actual: Path) -> int:
    expected = np.asarray(Image.open(golden).convert("RGBA"), dtype=np.int16)
    got = np.asarray(Image.open(actual).convert("RGBA"), dtype=np.int16)
    if expected.shape != got.shape:
        print(
            "GUI-VISUAL-PARITY FAIL: dimensions differ "
            f"(golden {expected.shape} vs capture {got.shape}). "
            "The shell layout changed size — re-approve with --bless if intended."
        )
        return 1
    delta = np.abs(expected - got).max(axis=2)  # worst per-pixel channel delta
    differing = int((delta > PER_PIXEL_CHANNEL_TOLERANCE).sum())
    total = int(delta.size)
    pct = 100.0 * differing / total
    max_delta = int(delta.max())
    print(
        f"GUI-VISUAL-PARITY diff: {differing}/{total} px differ "
        f"({pct:.3f}%), max channel delta {max_delta}, "
        f"tolerance {MAX_DIFFERING_PX_PCT:.2f}% @ per-pixel {PER_PIXEL_CHANNEL_TOLERANCE}"
    )
    if pct > MAX_DIFFERING_PX_PCT:
        print(
            "GUI-VISUAL-PARITY FAIL: the running app regressed from the "
            "owner-approved shell golden.\n"
            "  - If this is an UNINTENDED regression, fix the render.\n"
            "  - If this is an APPROVED visual change, re-capture and commit the\n"
            "    golden: python3 scripts/check_gui_visual_parity.py --bless\n"
            f"  golden: {GOLDEN.relative_to(ROOT)}"
        )
        return 1
    print("GUI-VISUAL-PARITY OK: app matches the owner-approved shell golden.")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--bless",
        action="store_true",
        help="re-capture and overwrite the owner-approved golden (owner action)",
    )
    args = parser.parse_args()

    if args.bless:
        GOLDEN.parent.mkdir(parents=True, exist_ok=True)
        capture(GOLDEN)
        print(f"GUI-VISUAL-PARITY blessed: wrote {GOLDEN.relative_to(ROOT)}")
        return 0

    if not GOLDEN.is_file():
        print(
            f"GUI-VISUAL-PARITY FAIL: missing golden {GOLDEN.relative_to(ROOT)}.\n"
            "  Capture and commit it once owner-approved:\n"
            "    python3 scripts/check_gui_visual_parity.py --bless"
        )
        return 1

    with tempfile.TemporaryDirectory() as tmp:
        actual = Path(tmp) / "datum-shell.capture.png"
        capture(actual)
        return diff(GOLDEN, actual)


if __name__ == "__main__":
    raise SystemExit(main())
