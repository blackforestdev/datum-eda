#!/usr/bin/env python3
"""Guard Datum's product identity against import/viewer scope drift."""

from __future__ import annotations

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]

REQUIRED_FILES = [
    ROOT / "docs/decisions/PRODUCT_MECHANICS_016_PRODUCT_NORTH_STAR.md",
    ROOT / "specs/PROGRESS.md",
    ROOT / "CLAUDE.md",
    ROOT / "README.md",
]

REQUIRED_PATTERNS = {
    ROOT / "docs/decisions/PRODUCT_MECHANICS_016_PRODUCT_NORTH_STAR.md": [
        r"professional native EDA system",
        r"governed native libraries",
        r"schematic capture",
        r"KiCad.*not Datum's\s+native architecture",
        r"library -> schematic -> PCB -> manufacturing",
    ],
    ROOT / "specs/PROGRESS.md": [
        r"library/schematic foundation",
        r"Import fidelity cannot be the active maturity metric",
    ],
    ROOT / "CLAUDE.md": [
        r"governed library plus schematic authority",
        r"library -> schematic -> PCB -> manufacturing",
    ],
    ROOT / "README.md": [
        r"native governed libraries",
        r"schematic capture",
        r"KiCad-first support.*not the long-term product boundary",
    ],
}

FORBIDDEN_UNQUALIFIED = [
    re.compile(r"Datum\s+is\s+(?:a|an)\s+KiCad\s+importer", re.IGNORECASE),
    re.compile(r"Datum\s+is\s+(?:a|an)\s+board\s+viewer", re.IGNORECASE),
    re.compile(r"Datum\s+is\s+(?:a|an)\s+AI-only", re.IGNORECASE),
    re.compile(r"KiCad\s+import\s+is\s+the\s+North\s+Star", re.IGNORECASE),
]


def main() -> int:
    failures: list[str] = []
    for path in REQUIRED_FILES:
        if not path.exists():
            failures.append(f"missing required North Star file: {path.relative_to(ROOT)}")
            continue
        text = path.read_text()
        for pattern in REQUIRED_PATTERNS.get(path, []):
            if not re.search(pattern, text, re.IGNORECASE | re.DOTALL):
                failures.append(
                    f"{path.relative_to(ROOT)} missing required North Star marker: {pattern}"
                )
        for forbidden in FORBIDDEN_UNQUALIFIED:
            if forbidden.search(text):
                failures.append(
                    f"{path.relative_to(ROOT)} contains unqualified forbidden product framing: "
                    f"{forbidden.pattern}"
                )

    if failures:
        print("Product North Star gate failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print("Product North Star gate passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
