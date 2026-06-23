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
    # CLI decomposition freeze caps (non-growth guardrails).
    Budget("crates/cli/src/command_project.rs", 9400, mode="total"),
    Budget("crates/cli/src/command_exec.rs", 2700, mode="total"),
    Budget("crates/cli/src/command_exec_project_command.rs", 804, mode="pre_test"),
    Budget("crates/cli/src/command_exec_project_library.rs", 910, mode="pre_test"),
    Budget("crates/cli/src/cli_args.rs", 2350, mode="total"),
    Budget("crates/cli/src/cli_args_project_command_args_artifacts.rs", 788, mode="pre_test"),
    Budget("crates/cli/src/cli_args_project_command_args_board.rs", 738, mode="pre_test"),
    Budget("crates/cli/src/cli_args_project_query_plan.rs", 716, mode="pre_test"),
    Budget("crates/cli/src/command_exec_project_query.rs", 1423, mode="pre_test"),
    Budget("crates/cli/src/command_project_forward_annotation_artifact.rs", 702, mode="pre_test"),
    Budget("crates/cli/src/command_exec_project_board_surface.rs", 751, mode="pre_test"),
    Budget("crates/cli/src/command_project_board_routing_net.rs", 933, mode="pre_test"),
    Budget("crates/cli/src/command_project_gerber_layers.rs", 704, mode="pre_test"),
    Budget("crates/cli/src/cli_args_project_library.rs", 994, mode="pre_test"),
    Budget("crates/cli/src/command_project_library.rs", 1797, mode="pre_test"),
    Budget("crates/cli/src/command_project_library_package_geometry.rs", 744, mode="pre_test"),
    Budget("crates/cli/src/command_project_native_inspect.rs", 734, mode="pre_test"),
    Budget("crates/cli/src/command_project_route_proposal.rs", 6284, mode="pre_test"),
    Budget("crates/cli/src/command_project_validate.rs", 1042, mode="pre_test"),
    Budget("crates/cli/src/command_exec_dispatch.rs", 802, mode="pre_test"),
    Budget("crates/cli/src/command_project_artifacts.rs", 764, mode="pre_test"),
    Budget("crates/cli/src/main_route_proposal.rs", 1917, mode="pre_test"),
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
    Budget("crates/engine/src/substrate/mod.rs", 706, mode="pre_test"),
    Budget("crates/engine/src/substrate/board_journal_ops.rs", 707, mode="pre_test"),
    Budget("crates/engine/src/substrate/operation_application.rs", 714, mode="pre_test"),
    Budget("crates/engine/src/substrate/zone_fill.rs", 770, mode="pre_test"),
    Budget("crates/engine/src/substrate/replay.rs", 920, mode="pre_test"),
    Budget("crates/engine/src/substrate/journal.rs", 820, mode="pre_test"),
    Budget("crates/cli/src/command_project_schematic_connectivity_mutations.rs", 780, mode="pre_test"),
    Budget("crates/engine/src/api/write_ops/undo_redo.rs", 20, mode="total"),
    Budget("crates/engine/src/api/write_ops/undo_redo/undo.rs", 420, mode="total"),
    Budget("crates/engine/src/api/write_ops/undo_redo/redo.rs", 420, mode="total"),
    Budget("crates/engine-daemon/src/main.rs", 420, mode="pre_test"),
    Budget("crates/cli/src/main.rs", 3050, mode="pre_test"),
    Budget("crates/engine/src/board/mod.rs", 730, mode="pre_test"),
    Budget("crates/engine/src/connectivity/mod.rs", 790, mode="pre_test"),
    Budget("crates/engine/src/erc/mod.rs", 500, mode="pre_test"),
    Budget("crates/engine/src/drc/mod.rs", 160, mode="pre_test"),
    Budget("crates/engine/src/import/kicad/mod.rs", 805, mode="pre_test"),
    Budget("crates/engine/src/import/kicad/skeleton.rs", 1339, mode="pre_test"),
    Budget("crates/engine/src/import/mod.rs", 130, mode="pre_test"),
    Budget("crates/engine/src/import/ids_sidecar.rs", 160, mode="pre_test"),
    Budget("crates/engine/src/import/kicad/parser_helpers.rs", 991, mode="total"),
    Budget("crates/engine/src/import/kicad/symbol_helpers.rs", 300, mode="total"),
    Budget("crates/engine/src/import/eagle/mod.rs", 550, mode="pre_test"),
    Budget("crates/engine/src/import/eagle/pool_builder.rs", 320, mode="total"),
    Budget("crates/engine/src/import/eagle/xml_helpers.rs", 220, mode="total"),
    Budget("crates/engine/src/schematic/mod.rs", 580, mode="pre_test"),
    Budget("crates/engine/src/export/mod.rs", 950, mode="pre_test"),
    Budget("crates/engine/src/pool/mod.rs", 586, mode="pre_test"),
    Budget("crates/gui-app/src/main.rs", 3229, mode="pre_test"),
    # +17 for the Decision-013 read-only supervision-reflection crate-root
    # surface (the `supervision` workspace field + `with_supervision_from_model`
    # builder + `DockTab::Supervision` arm + supervision command dispatch +
    # re-export); the larger fixture/golden builders live in supervision.rs.
    Budget("crates/gui-protocol/src/lib.rs", 7227, mode="pre_test"),
    Budget("crates/gui-render/src/outputs_lane.rs", 1067, mode="pre_test"),
    Budget("crates/cli/src/command_project_check_run_view.rs", 862, mode="pre_test"),
    Budget("crates/gui-render/src/lib.rs", 7853, mode="pre_test"),
    Budget("crates/test-harness/src/bin/m3_sidecar_roundtrip_fidelity.rs", 950, mode="total"),
    Budget("mcp-server/server_runtime.py", 994, mode="total"),
    Budget("mcp-server/tools_catalog.py", 400, mode="total"),
    Budget("mcp-server/tools_catalog_data.py", 1062, mode="total"),
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
