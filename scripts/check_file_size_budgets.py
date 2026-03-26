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
    Budget("crates/engine/src/api/mod.rs", 700, mode="pre_test"),
    Budget("crates/engine/src/api/ops_helpers.rs", 320, mode="total"),
    Budget("crates/engine/src/api/write_ops.rs", 700, mode="total"),
    Budget("crates/engine/src/api/project_surface.rs", 420, mode="total"),
    Budget(
        "crates/engine/src/api/project_surface/project_surface_replacements.rs",
        420,
        mode="total",
    ),
    Budget("crates/engine/src/api/save_kicad.rs", 220, mode="total"),
    Budget("crates/engine/src/api/save_kicad/transaction_state.rs", 180, mode="total"),
    Budget("crates/engine/src/api/save_kicad/kicad_text.rs", 520, mode="total"),
    Budget("crates/engine/src/api/write_ops/undo_redo.rs", 20, mode="total"),
    Budget("crates/engine/src/api/write_ops/undo_redo/undo.rs", 420, mode="total"),
    Budget("crates/engine/src/api/write_ops/undo_redo/redo.rs", 420, mode="total"),
    Budget("crates/engine-daemon/src/main.rs", 350, mode="pre_test"),
    Budget("crates/cli/src/main.rs", 350, mode="pre_test"),
    Budget("crates/engine/src/board/mod.rs", 670, mode="pre_test"),
    Budget("crates/engine/src/connectivity/mod.rs", 500, mode="pre_test"),
    Budget("crates/engine/src/erc/mod.rs", 500, mode="pre_test"),
    Budget("crates/engine/src/drc/mod.rs", 160, mode="pre_test"),
    Budget("crates/engine/src/import/kicad/mod.rs", 700, mode="pre_test"),
    Budget("crates/engine/src/import/mod.rs", 130, mode="pre_test"),
    Budget("crates/engine/src/import/ids_sidecar.rs", 160, mode="pre_test"),
    Budget("crates/engine/src/import/kicad/parser_helpers.rs", 550, mode="total"),
    Budget("crates/engine/src/import/kicad/symbol_helpers.rs", 300, mode="total"),
    Budget("crates/engine/src/import/eagle/mod.rs", 550, mode="pre_test"),
    Budget("crates/engine/src/import/eagle/pool_builder.rs", 320, mode="total"),
    Budget("crates/engine/src/import/eagle/xml_helpers.rs", 220, mode="total"),
    Budget("crates/engine/src/schematic/mod.rs", 580, mode="pre_test"),
    Budget("crates/engine/src/pool/mod.rs", 380, mode="pre_test"),
    Budget("mcp-server/server_runtime.py", 700, mode="total"),
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
