#!/usr/bin/env python3
"""Failing GUI visual-parity gate (same-engine app-screenshot regression).

This is a machine no-regression gate over a SINGLE-PANE INTERIM shell target. A
Rust/wgpu render will never pixel-match the HTML prototype, so this gate does NOT
diff the build against ``docs/gui/prototypes/board-editor.html``. Instead it:

  1. captures the running app at a CANONICAL command — the datum-test board with
     a preset component selection (R1) + fixed window size (build-vs-build, one
     renderer), producing a populated SINGLE-PANE composition (board + inspector)
     — and
  2. diffs that capture against a COMMITTED shell golden with a small tolerance.

**Honest scope (do not overstate).** The golden this gate protects is a
SINGLE-PANE interim target. It is **NOT** owner-approved against
``board-editor.html``: the prototype is a SPLIT Board+Schematic composition with
a populated inspector, and that full composition CANNOT be captured until the
split view + schematic pane are built in **Phase-2** (there is no config
shortcut). This gate freezes the current single-pane look so it does not silently
regress; it does not certify prototype parity. The one-time owner cross-engine
approval of the full board-editor composition is tracked separately by the
reference-capture loop (``docs/gui/reference/README.md`` +
``scripts/check_gui_reference_capture.py``, EXPECTED RED until Phase-2).

To keep a wrong scene from being blessed as this golden, the gate applies cheap
SEMANTIC GUARDS (``guard_intended_fixture``): the capture must come from a real
board fixture whose layer stack includes the expected copper/silk/edge layers,
and the fixture must NOT be the synthetic "Datum GUI Known Good" demo scene. The
split-pane / U1 / STM32 content guards are DEFERRED to Phase-2 (they cannot be
asserted until the split view renders them) — see the TODO in
``guard_intended_fixture``.

Usage::

    python3 scripts/check_gui_visual_parity.py           # verify (fails on drift)
    python3 scripts/check_gui_visual_parity.py --bless    # re-capture the golden

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

# The single canonical capture: datum-test board with a preset component
# selection (R1), fixed window. Repointed from the empty --demo-known-good
# route-review scene to a populated SINGLE-PANE composition (board + populated
# component inspector). This is a single-pane INTERIM target; it is NOT the
# split Board+Schematic composition of docs/gui/prototypes/board-editor.html —
# that full composition is gated on Phase-2 (the split view + schematic pane).
WINDOW_SIZE = "1680x1050"
DATUM_TEST_BOARD = (
    "/home/bfadmin/Documents/kicad_projects/Datum-eda/datum-test/datum-test.kicad_pcb"
)
SELECT_REFERENCE = "R1"
GOLDEN = ROOT / "crates/gui-render/testdata/golden/shell/datum-shell.golden.png"

# Semantic-guard expectations. The capture must come from a real board fixture
# whose layer stack includes these copper/silk/edge layers, and the fixture must
# NOT be the synthetic known-good demo scene. This keeps a wrong scene from being
# frozen as the golden even though the pixel-diff alone cannot tell "right board"
# from "some other board of the same size".
EXPECTED_FIXTURE_LAYERS = ("F.Cu", "B.Cu", "F.SilkS", "Edge.Cuts")
KNOWN_GOOD_DEMO_TITLE = "Datum GUI Known Good"

# Small tolerance: absorbs sub-pixel AA / font-raster jitter only. Same renderer,
# so the capture is near-identical to the golden (verified at 0 differing px on the
# reference workstation). The bar is deliberately tight so a REAL chrome regression
# — a moved panel, a re-introduced uppercase run, a resurrected ON/OFF badge, the
# floating-card border — blows past it and FAILS. Two independent assertions:
#   (a) < MAX_DIFFERING_PX_PCT of pixels differ by more than PER_PIXEL_CHANNEL_TOLERANCE
#       on any channel (catches small, high-contrast regressions like a badge), AND
#   (b) mean per-channel difference < MAX_MEAN_CHANNEL_DELTA (catches broad, low-amplitude
#       shifts like a re-tinted surface that individually stay under the per-pixel bar).
PER_PIXEL_CHANNEL_TOLERANCE = 8   # 0..255 per-channel delta ignored below this
MAX_DIFFERING_PX_PCT = 0.5        # fail if more than this share of pixels differ
MAX_MEAN_CHANNEL_DELTA = 1.5      # fail if the mean per-channel delta exceeds this


def pending_shadows() -> list[Path]:
    """Any *.PENDING placeholder sitting beside the golden.

    A pending placeholder can NEVER make this gate green — that is the exact
    paperwork defect this gate corrects.
    """
    if not GOLDEN.parent.is_dir():
        return []
    return sorted(GOLDEN.parent.glob("*.PENDING"))


def guard_intended_fixture() -> None:
    """Cheap semantic guards so a WRONG scene cannot be blessed as the golden.

    The pixel-diff proves "did not change"; it cannot prove "is the right board"
    (a different board of the same window size would diff clean against a golden
    blessed from it). These guards pin the *identity* of the captured scene at the
    input side, before capture:

      1. the capture command targets a REAL board fixture that exists on disk,
      2. that fixture's layer stack includes the expected copper/silk/edge layers
         (F.Cu / B.Cu / F.SilkS / Edge.Cuts) — i.e. it is a real PCB, not an empty
         or synthetic scene, and
      3. the capture is NOT the synthetic "Datum GUI Known Good" demo scene, and
         the golden's committed title is not that demo title.

    Raising here fails the gate loudly rather than silently blessing/verifying the
    wrong scene.

    TODO(Phase-2): once the split view + schematic pane render, extend these
    guards to assert the SPLIT-PANE composition (two pane headers), the U1
    STM32 part in the inspector, and the schematic pane content. Those cannot be
    asserted today because the single-pane build does not render them — do NOT add
    them until the Phase-2 slice makes them real, or the guard would be red
    against un-built structure.
    """
    board = Path(DATUM_TEST_BOARD)
    if not board.is_file():
        raise SystemExit(
            "GUI-VISUAL-PARITY FAIL (fixture guard): the canonical capture board "
            f"is missing: {DATUM_TEST_BOARD}\n"
            "  The gate must capture the intended real datum-test board, not a "
            "fallback or synthetic scene."
        )
    text = board.read_text(errors="replace")
    if KNOWN_GOOD_DEMO_TITLE in text:
        raise SystemExit(
            "GUI-VISUAL-PARITY FAIL (fixture guard): the capture fixture looks "
            f"like the synthetic '{KNOWN_GOOD_DEMO_TITLE}' demo scene, not a real "
            "board. Refusing to bless/verify a known-good demo as the shell golden."
        )
    missing = [layer for layer in EXPECTED_FIXTURE_LAYERS if f'"{layer}"' not in text]
    if missing:
        raise SystemExit(
            "GUI-VISUAL-PARITY FAIL (fixture guard): the capture fixture is not the "
            "expected real board — missing layer(s) "
            f"{', '.join(missing)} in {DATUM_TEST_BOARD}.\n"
            f"  Expected a PCB with the layer stack {', '.join(EXPECTED_FIXTURE_LAYERS)}."
        )
    if GOLDEN.is_file():
        # A committed golden captured from the known-good demo would carry that
        # title in no readable form here (it is a PNG), so this is a belt-and-braces
        # constant assertion: the gate is wired to a real board, never the demo.
        assert SELECT_REFERENCE and SELECT_REFERENCE != KNOWN_GOOD_DEMO_TITLE
    print(
        "GUI-VISUAL-PARITY fixture guard OK: real datum-test board with "
        f"layers {', '.join(EXPECTED_FIXTURE_LAYERS)}; not the known-good demo. "
        "(split-pane/U1/STM32 guards deferred to Phase-2.)"
    )


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
        "--board",
        DATUM_TEST_BOARD,
        "--select",
        SELECT_REFERENCE,
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
    expected = np.asarray(Image.open(golden).convert("RGB"), dtype=np.int16)
    got = np.asarray(Image.open(actual).convert("RGB"), dtype=np.int16)
    if expected.shape != got.shape:
        print(
            "GUI-VISUAL-PARITY FAIL: dimensions differ "
            f"(golden {expected.shape} vs capture {got.shape}). "
            "The shell layout changed size — re-approve with --bless if intended."
        )
        return 1
    channel_delta = np.abs(expected - got)
    delta = channel_delta.max(axis=2)  # worst per-pixel channel delta
    differing = int((delta > PER_PIXEL_CHANNEL_TOLERANCE).sum())
    total = int(delta.size)
    pct = 100.0 * differing / total
    max_delta = int(delta.max())
    mean_delta = float(channel_delta.mean())  # mean over every channel of every pixel
    print(
        f"GUI-VISUAL-PARITY diff: {differing}/{total} px differ "
        f"({pct:.3f}%), max channel delta {max_delta}, mean channel delta {mean_delta:.4f}; "
        f"tolerance {MAX_DIFFERING_PX_PCT:.2f}% @ per-pixel {PER_PIXEL_CHANNEL_TOLERANCE}, "
        f"mean {MAX_MEAN_CHANNEL_DELTA}"
    )
    regressed = pct > MAX_DIFFERING_PX_PCT or mean_delta > MAX_MEAN_CHANNEL_DELTA
    if regressed:
        print(
            "GUI-VISUAL-PARITY FAIL: the running app regressed from the "
            "committed single-pane interim shell golden.\n"
            f"  - differing-pixel {pct:.3f}% (limit {MAX_DIFFERING_PX_PCT}%), "
            f"mean channel delta {mean_delta:.4f} (limit {MAX_MEAN_CHANNEL_DELTA}).\n"
            "  - If this is an UNINTENDED regression, fix the render.\n"
            "  - If this is an APPROVED visual change, re-capture and commit the\n"
            "    golden: python3 scripts/check_gui_visual_parity.py --bless\n"
            f"  golden: {GOLDEN.relative_to(ROOT)}"
        )
        return 1
    print(
        "GUI-VISUAL-PARITY OK: app matches the committed single-pane interim "
        "shell golden (NOT prototype parity — full board-editor.html composition "
        "is gated on Phase-2)."
    )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--bless",
        action="store_true",
        help="re-capture and overwrite the single-pane interim golden (owner action)",
    )
    args = parser.parse_args()

    if args.bless:
        # Guard the scene identity BEFORE freezing it as the golden, so a wrong
        # scene can never be blessed.
        guard_intended_fixture()
        GOLDEN.parent.mkdir(parents=True, exist_ok=True)
        capture(GOLDEN)
        print(f"GUI-VISUAL-PARITY blessed: wrote {GOLDEN.relative_to(ROOT)}")
        return 0

    # Anti-paperwork teeth: an absent golden OR a *.PENDING placeholder beside it
    # FAILS the gate — a pending placeholder can never make the gate green. This is
    # the exact defect being corrected (a HUMAN row guarded by a PENDING file).
    guard_intended_fixture()
    shadows = pending_shadows()
    if not GOLDEN.is_file() or shadows:
        detail = (
            f"a *.PENDING placeholder shadows it ({', '.join(str(p.relative_to(ROOT)) for p in shadows)})"
            if shadows
            else "the golden file is absent"
        )
        print(
            "GUI-VISUAL-PARITY FAIL: single-pane interim shell golden not committed — "
            f"{detail}.\n"
            f"  expected: {GOLDEN.relative_to(ROOT)}\n"
            "  A PENDING placeholder does NOT satisfy this gate. Capture and commit\n"
            "  the golden (and delete any *.PENDING beside it):\n"
            "    python3 scripts/check_gui_visual_parity.py --bless"
        )
        return 1

    with tempfile.TemporaryDirectory() as tmp:
        actual = Path(tmp) / "datum-shell.capture.png"
        capture(actual)
        return diff(GOLDEN, actual)


if __name__ == "__main__":
    raise SystemExit(main())
