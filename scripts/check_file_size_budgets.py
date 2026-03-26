#!/usr/bin/env python3
"""Fail when key source files exceed configured line-count budgets."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[1]


@dataclass(frozen=True)
class Budget:
    path: str
    max_lines: int
    mode: str = "total"  # "total" or "pre_test"


BUDGETS: tuple[Budget, ...] = (
    Budget("crates/engine/src/api/mod.rs", 1600, mode="pre_test"),
    Budget("crates/engine-daemon/src/main.rs", 350, mode="pre_test"),
    Budget("crates/cli/src/main.rs", 350, mode="pre_test"),
    Budget("mcp-server/server_runtime.py", 650, mode="total"),
    Budget("mcp-server/tools_catalog.py", 400, mode="total"),
    Budget("mcp-server/tools_catalog_data.py", 700, mode="total"),
    Budget("mcp-server/tool_dispatch.py", 180, mode="total"),
)


def pre_test_line_count(text: str) -> int:
    for line_no, line in enumerate(text.splitlines(), start=1):
        if line.strip().startswith("#[cfg(test)]"):
            return line_no - 1
    return len(text.splitlines())


def line_count(path: Path, mode: str) -> int:
    text = path.read_text(encoding="utf-8")
    if mode == "pre_test":
        return pre_test_line_count(text)
    if mode == "total":
        return len(text.splitlines())
    raise ValueError(f"unknown mode: {mode}")


def main() -> int:
    failures: list[str] = []
    checked = 0
    for budget in BUDGETS:
        target = ROOT / budget.path
        if not target.exists():
            failures.append(f"missing file: {budget.path}")
            continue
        checked += 1
        actual = line_count(target, budget.mode)
        if actual > budget.max_lines:
            failures.append(
                f"{budget.path}: {actual} lines ({budget.mode}) > budget {budget.max_lines}"
            )

    if failures:
        print("Source file size budget check failed:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print(f"Source file size budget check passed ({checked} files).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
