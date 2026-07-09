#!/usr/bin/env python3
"""Honest reference-capture gate for the board-editor human-review loop.

This gate guards ONE fact and states it plainly: the owner-approved reference
image of the controlling prototype `docs/gui/prototypes/board-editor.html` has
NOT been captured yet. It FAILS while either is true:

  1. `docs/gui/reference/board-editor.png` is missing, OR
  2. a `docs/gui/reference/board-editor.png.PENDING` placeholder exists.

**This gate is EXPECTED to be RED right now, and that red is the honest signal —
not a bug.** The owner has not yet captured the reference image (the automated
capture SIGTRAPs in the sandbox that stood up the loop; see
`docs/gui/reference/README.md` §5). Until a real `board-editor.png` is committed
and the `.PENDING` placeholder is deleted in the same change, the board-editor
human-review loop has no reference to review against, and this gate says so out
loud instead of letting a `.PENDING` note masquerade as a captured reference.

Why a gate at all: the shell visual-parity gate
(`scripts/check_gui_visual_parity.py`) can be GREEN today, but it protects only a
SINGLE-PANE interim shell golden — NOT the full board-editor.html composition
(split Board+Schematic view with a populated inspector), which cannot be captured
or reviewed until Phase-2 builds the split view. Without this gate, the absence of
the real reference image is invisible and the loop reads as "done" when it is not.

Resolving the red (owner action, on a machine with a working headless browser):
run the §2 capture command in `docs/gui/reference/README.md`, verify the image,
`git add docs/gui/reference/board-editor.png`, and delete
`docs/gui/reference/board-editor.png.PENDING` in the same commit. This gate then
goes GREEN on its own — do NOT satisfy it with a fabricated or blank image.

Usage::

    python3 scripts/check_gui_reference_capture.py
"""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

REFERENCE = ROOT / "docs/gui/reference/board-editor.png"
PENDING = ROOT / "docs/gui/reference/board-editor.png.PENDING"


def main() -> int:
    problems: list[str] = []

    if PENDING.is_file():
        problems.append(
            f"a PENDING placeholder still exists ({PENDING.relative_to(ROOT)}) — "
            "it is a note, not a captured reference."
        )
    if not REFERENCE.is_file():
        problems.append(
            f"the reference image is absent ({REFERENCE.relative_to(ROOT)})."
        )

    if problems:
        print(
            "GUI-REFERENCE-CAPTURE RED (expected, honest): the board-editor "
            "reference image has not been captured yet."
        )
        for problem in problems:
            print(f"  - {problem}")
        print(
            "  This red is the honest signal that the board-editor human-review\n"
            "  loop has no owner-approved reference of\n"
            "  docs/gui/prototypes/board-editor.html to review against. It is NOT a\n"
            "  bug to be silenced with a fake image. To resolve (owner action):\n"
            "    1. run the §2 capture command in docs/gui/reference/README.md\n"
            "       on a machine with a working headless browser,\n"
            "    2. verify board-editor.png shows the full board-editor shell,\n"
            "    3. git add docs/gui/reference/board-editor.png and delete\n"
            "       docs/gui/reference/board-editor.png.PENDING in the same commit,\n"
            "    4. update docs/gui/reference/README.md §5 to CAPTURED.\n"
            "  NOTE: the full board-editor.html composition (split Board+Schematic\n"
            "  view, populated inspector) is a PHASE-2 target; the shell visual-\n"
            "  parity gate today protects only a single-pane interim golden."
        )
        return 1

    print(
        "GUI-REFERENCE-CAPTURE OK: docs/gui/reference/board-editor.png is "
        "committed and no *.PENDING placeholder shadows it."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
