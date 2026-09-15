#!/usr/bin/env python3
"""Reject rival authored/import length and angle conversion paths for UNIT-I03A."""

from __future__ import annotations

import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SOURCE_ROOTS = (
    ROOT / "crates" / "engine" / "src",
    ROOT / "crates" / "cli" / "src",
    ROOT / "crates" / "gui-protocol" / "src",
)

FORBIDDEN = {
    "retired floating conversion helper": re.compile(
        r"(?<![A-Za-z0-9_])(?:mm_to_nm|mil_to_nm|inch_to_nm)\s*\("
    ),
    "direct millimeter-to-nanometer rounding": re.compile(
        r"\b(?:mm|x_mm|y_mm|parsed)\s*\*\s*1_000_000\.0\)\.round\(\)"
    ),
    "direct inch-to-nanometer rounding": re.compile(
        r"\b(?:inch|parsed)\s*\*\s*25_400_000\.0\)\.round\(\)"
    ),
    "rounded authored/import rotation": re.compile(
        r"\b(?:rotation|rot)\.round\(\)\s+as\s+i32"
    ),
}


def main() -> int:
    failures: list[str] = []
    checked_adapters = 0
    for source_root in SOURCE_ROOTS:
        for path in source_root.rglob("*.rs"):
            text = path.read_text(encoding="utf-8")
            relative = path.relative_to(ROOT)
            for label, pattern in FORBIDDEN.items():
                for match in pattern.finditer(text):
                    line = text.count("\n", 0, match.start()) + 1
                    failures.append(f"{relative}:{line}: {label}")
            checked_adapters += text.count("checked_f64_length(")
    if checked_adapters < 5:
        failures.append(
            "conversion inventory lost expected checked compatibility adapters "
            f"(found {checked_adapters}, expected at least 5)"
        )
    if failures:
        print("Units conversion inventory failed:")
        for failure in failures:
            print(f"- {failure}")
        return 1
    print(
        "Units conversion inventory passed: retired floating helpers and direct "
        "length/rotation rounding are absent; checked adapters delegate to the exact service."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
