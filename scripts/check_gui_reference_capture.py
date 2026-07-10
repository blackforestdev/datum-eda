#!/usr/bin/env python3
"""Reference-capture gate for the board-editor human-review loop.

Guards ONE fact: the owner's design reference — a browser capture of the
controlling prototype ``docs/gui/prototypes/board-editor.html`` — is committed at
``docs/gui/reference/board-editor.png``, so the human-review step has a real
design target to compare the running app against. The gate FAILS if:

  1. ``docs/gui/reference/board-editor.png`` is missing,
  2. a ``docs/gui/reference/board-editor.png.PENDING`` placeholder shadows it, or
  3. the file is not a valid, non-trivial PNG.

This is now GREEN — the owner captured the reference. It was intentionally RED
until then (a ``.PENDING`` note is not a captured reference). It stays a live gate
so the reference can never silently disappear or be replaced by a placeholder.

Two different images — do NOT conflate them:
  - ``board-editor.png`` (this reference) is a BROWSER screenshot of
    ``board-editor.html`` — the human design target, reviewed BY EYE against the
    app. It is captured at the owner's browser viewport (its own dimensions, e.g.
    ~1920x953) and is NOT pixel-diffed against the app (cross-engine wgpu-vs-HTML
    never matches), so it need not match the app golden's 1680x1050.
  - The app golden
    (``crates/gui-render/testdata/golden/shell/datum-shell.golden.png``, captured
    at 1680x1050 by ``check_gui_visual_parity.py``) is a different artifact: the
    machine no-regression target for the current SINGLE-PANE INTERIM shell. The
    full split Board+Schematic composition is a Phase-2 target.

Usage::

    python3 scripts/check_gui_reference_capture.py
"""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

REFERENCE = ROOT / "docs/gui/reference/board-editor.png"
PENDING = ROOT / "docs/gui/reference/board-editor.png.PENDING"
MIN_DIM = 400  # a real prototype screenshot, not a blank/stub


def main() -> int:
    problems: list[str] = []

    if PENDING.is_file():
        problems.append(
            f"a PENDING placeholder still exists ({PENDING.relative_to(ROOT)}) — "
            "it is a note, not a captured reference."
        )
    if not REFERENCE.is_file():
        problems.append(f"the reference image is absent ({REFERENCE.relative_to(ROOT)}).")

    size = None
    if REFERENCE.is_file():
        try:
            from PIL import Image

            with Image.open(REFERENCE) as im:
                im.verify()
            with Image.open(REFERENCE) as im:
                size = im.size
            if min(size) < MIN_DIM:
                problems.append(
                    f"the reference image is implausibly small ({size[0]}x{size[1]}); "
                    "expected a real board-editor.html screenshot."
                )
        except Exception as exc:  # noqa: BLE001 - report any decode failure honestly
            problems.append(f"the reference image is not a valid PNG: {exc}")

    if problems:
        print(
            "GUI-REFERENCE-CAPTURE FAILED: the board-editor design reference is "
            "missing, shadowed, or invalid."
        )
        for problem in problems:
            print(f"  - {problem}")
        print(
            "  The board-editor human-review loop needs a real browser screenshot\n"
            "  of docs/gui/prototypes/board-editor.html at\n"
            "  docs/gui/reference/board-editor.png (no *.PENDING shadow). Do NOT\n"
            "  satisfy it with a fabricated or blank image, and do NOT substitute a\n"
            "  screenshot of the Datum app (that is the app golden, a DIFFERENT\n"
            "  artifact). See docs/gui/reference/README.md."
        )
        return 1

    print(
        f"GUI-REFERENCE-CAPTURE OK: board-editor.png committed "
        f"({size[0]}x{size[1]} browser capture of the prototype), no *.PENDING shadow."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
