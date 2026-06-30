#!/usr/bin/env python3
"""Guard the native library foundation contract against stale scope drift."""

from __future__ import annotations

from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[1]


FORBIDDEN = {
    "docs/contracts/LIBRARY_AUTHORING_TOOL_CONTRACT.md": [
        "READ + board-consume ONLY",
        "grep returned 0 hits",
        "The entire authoring substrate",
        "SUBSTRATE SEQUENCING",
    ],
    "docs/LIBRARY_ARCHITECTURE.md": [
        "Package`: physical footprint",
        "Package (physical footprint)",
        "The physical footprint. Defines pads",
    ],
    "docs/POOL_ARCHITECTURE.md": [
        "Package (physical footprint)",
        "The physical footprint. Defines pads",
        "| .kicad_mod footprint | Package |",
    ],
    "docs/ENGINE_DESIGN.md": [
        "Package (physical footprint)",
        "pad_map: Map<PadUUID",
    ],
}


REQUIRED = {
    "docs/contracts/LIBRARY_AUTHORING_TOOL_CONTRACT.md": [
        "The remaining blocker is not \"no substrate.\"",
        "`Package` is not a `Footprint`.",
        "`PinPadMap` is first-class library data.",
        "Engine-owned `LibraryGraph` authority.",
    ],
    "docs/LIBRARY_ARCHITECTURE.md": [
        "Horizon's library architecture is useful prior art, not the target ceiling",
        "Package\n  -> Footprint",
        "### Footprint",
        "### PinPadMap",
    ],
    "docs/POOL_ARCHITECTURE.md": [
        "Footprint (PCB land pattern)",
        "### Footprint",
        "pin_pad_maps/",
        "not the product-level revision",
    ],
    "specs/NATIVE_FORMAT_SPEC.md": [
        "### Library foundation target schema",
        "`Package`: component body/package-family data.",
        "`Footprint`: PCB land pattern.",
        "`PinPadMap`: first-class logical-to-physical binding.",
        "Validation tiers:",
    ],
    "specs/PROGRESS.md": [
        "Native library foundation",
        "This is the next implementation axis before board-editor expansion.",
    ],
    "docs/ENGINE_DESIGN.md": [
        "Package (physical component body / terminal family)",
        "Footprint (PCB land pattern)",
        "pin_pad_map: UUID",
    ],
}


def main() -> int:
    failures: list[str] = []

    for rel, needles in FORBIDDEN.items():
        text = (ROOT / rel).read_text(encoding="utf-8")
        for needle in needles:
            if needle in text:
                failures.append(f"{rel}: forbidden stale phrase remains: {needle!r}")

    for rel, needles in REQUIRED.items():
        text = (ROOT / rel).read_text(encoding="utf-8")
        for needle in needles:
            if needle not in text:
                failures.append(f"{rel}: required contract phrase missing: {needle!r}")

    if failures:
        print("library foundation contract gate failed:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print("library foundation contract gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
