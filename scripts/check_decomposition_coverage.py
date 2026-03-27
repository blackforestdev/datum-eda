#!/usr/bin/env python3
"""Ensure large source modules are tracked by explicit size budgets.

This is a structural governance check. Any oversized source file must be
explicitly listed in `scripts/check_file_size_budgets.py` so decomposition work
cannot silently drift.
"""

from __future__ import annotations

from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parents[1]
SIZE_BUDGET_SCRIPT = ROOT / "scripts/check_file_size_budgets.py"
OVERSIZE_PRE_TEST_THRESHOLD = 700


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def parse_budget_paths() -> set[str]:
    text = read_text(SIZE_BUDGET_SCRIPT)
    return set(re.findall(r'Budget\("([^"]+)"', text))


def pre_test_line_count(text: str) -> int:
    for line_no, line in enumerate(text.splitlines(), start=1):
        if line.strip().startswith("#[cfg(test)]"):
            return line_no - 1
    return len(text.splitlines())


def is_test_module(path: Path) -> bool:
    rel = str(path.relative_to(ROOT))
    if "/tests/" in rel:
        return True
    name = path.name.lower()
    if "test" in name:
        return True
    return False


def find_oversized_source_modules() -> list[tuple[str, int]]:
    rows: list[tuple[str, int]] = []
    for path in (ROOT / "crates").rglob("*.rs"):
        text = read_text(path)
        if is_test_module(path):
            continue
        pre_test = pre_test_line_count(text)
        if pre_test > OVERSIZE_PRE_TEST_THRESHOLD:
            rows.append((str(path.relative_to(ROOT)), pre_test))
    return sorted(rows, key=lambda item: item[1], reverse=True)


def main() -> int:
    budgeted = parse_budget_paths()
    oversized = find_oversized_source_modules()
    missing = [(path, lines) for path, lines in oversized if path not in budgeted]

    if missing:
        print("Decomposition coverage check failed:", file=sys.stderr)
        print(
            "  Oversized source modules must be explicitly budgeted in "
            "`scripts/check_file_size_budgets.py`.",
            file=sys.stderr,
        )
        for path, lines in missing:
            print(
                f"  - {path}: {lines} pre-test lines > {OVERSIZE_PRE_TEST_THRESHOLD}",
                file=sys.stderr,
            )
        return 1

    print(
        "Decomposition coverage check passed "
        f"({len(oversized)} oversized source modules are explicitly budgeted)."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
