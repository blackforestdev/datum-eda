#!/usr/bin/env python3
"""Gate the custom Datum EDA glyph asset set."""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ICON_DIR = ROOT / "crates/engine/assets/icons/eda"

REQUIRED = [
    "route-track.svg",
    "via.svg",
    "zone-pour.svg",
    "pad.svg",
    "net-ratsnest.svg",
    "layer-stack.svg",
    "drc-marker.svg",
    "teardrop.svg",
    "keepout.svg",
    "diff-pair.svg",
    "length-tune.svg",
]


def fail(message: str) -> None:
    print(f"GUI icon asset gate failed: {message}", file=sys.stderr)
    sys.exit(1)


def main() -> None:
    if not ICON_DIR.is_dir():
        fail(f"missing icon directory {ICON_DIR}")

    missing = [name for name in REQUIRED if not (ICON_DIR / name).is_file()]
    if missing:
        fail(f"missing required icons: {', '.join(missing)}")

    failures: list[str] = []
    for name in REQUIRED:
        text = (ICON_DIR / name).read_text()
        if 'viewBox="0 0 24 24"' not in text:
            failures.append(f"{name}: missing 24px viewBox")
        if 'stroke="currentColor"' not in text:
            failures.append(f"{name}: stroke must use currentColor")
        if 'fill="none"' not in text:
            failures.append(f"{name}: root fill must be none")
        if "stroke-width=\"2\"" not in text:
            failures.append(f"{name}: expected 2px stroke")
        for forbidden in ["fill=\"#", "stroke=\"#", "style=\""]:
            if forbidden in text:
                failures.append(f"{name}: contains forbidden hardcoded style {forbidden}")

    if failures:
        fail("; ".join(failures))

    print(f"GUI icon asset gate passed ({len(REQUIRED)} custom EDA glyphs).")


if __name__ == "__main__":
    main()
