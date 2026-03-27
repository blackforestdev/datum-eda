#!/usr/bin/env python3
"""Static and executable alignment audit for Datum EDA docs/specs.

Default mode runs fast static checks for known parity contracts.
Optional flags run slower executable verification for milestone gates and
workspace health.
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent


@dataclass(frozen=True)
class TextCheck:
    path: str
    must_contain: tuple[str, ...] = ()
    must_not_contain: tuple[str, ...] = ()


TEXT_CHECKS = (
    TextCheck(
        path="specs/PROGRAM_SPEC.md",
        must_contain=(
            "## Scope Integrity Terms",
            "Status tracking rule:",
            "`specs/PROGRESS.md` is the sole source of truth for implementation status.",
            "**Product identity**",
            "**Implementation slice**",
            "**Execution strategy**",
            "**Non-goals**",
            "Do not infer product identity from implementation-slice limits.",
            "Do not infer permanent product limits from milestone non-goals.",
        ),
    ),
    TextCheck(
        path="README.md",
        must_contain=(
            "Canonical scope terminology is defined in",
            "Implementation slice",
            "Execution strategy",
        ),
    ),
    TextCheck(
        path="specs/PROGRESS.md",
        must_contain=(
            "**M2 overall**: [x] Complete for the current implementation slice",
            "Current repo health status (2026-03-25 audit):",
            "- `cargo test -q` currently passes.",
            "M3 determinism evidence hook behavioral",
            "M3 undo/redo evidence hook behavioral",
            "M3 write-surface parity hook behavioral",
        ),
    ),
    TextCheck(
        path="specs/INTEGRATED_PROGRAM_SPEC.md",
        must_contain=(
            "Status authority rule:",
            "`specs/PROGRESS.md` is the only source of truth for implementation status.",
            "Milestone completion state must be recorded in `specs/PROGRESS.md` only.",
            "| Workspace health | Automated test run | `cargo test -q`",
            "m3_op_determinism",
            "m3_undo_redo_roundtrip",
            "m3_write_surface_parity",
        ),
    ),
    TextCheck(
        path="specs/MCP_API_SPEC.md",
        must_contain=(
            "### Current Implemented Methods (2026-03-25)",
            "#### `get_design_rules`",
            "Current implementation note: implemented in the current daemon/stdio host",
        ),
        must_not_contain=(
            "Status: `Target M2+` (not implemented in current daemon/MCP slice).",
            "Current slice is read/check-focused. Write operations, save, and close-project\nlifecycle control are deferred.",
        ),
    ),
    TextCheck(
        path="specs/ENGINE_SPEC.md",
        must_contain=(
            "pub fn close_project(&mut self);",
            "pub fn get_part(&self, uuid: &Uuid) -> Result<PartDetail>;",
            "pub fn get_package(&self, uuid: &Uuid) -> Result<PackageDetail>;",
            "pub fn get_symbol_fields(&self, symbol_uuid: &Uuid) -> Result<Vec<SymbolFieldInfo>>;",
            "pub fn get_bus_entries(&self, sheet: Option<&Uuid>) -> Result<Vec<BusEntryInfo>>;",
            "pub fn get_netlist(&self) -> Result<Vec<NetlistNet>>;",
            "pub fn get_design_rules(&self) -> Result<Vec<Rule>>;",
            "pub fn explain_violation(",
        ),
        must_not_contain=(
            "Target parity note: full CLI/MCP method parity is required for the target M2+\nsurface at milestone closure. The current implementation exposes a strict\nsubset;",
        ),
    ),
    TextCheck(
        path="docs/IMPLEMENTATION_GUARDRAILS.md",
        must_contain=(
            "**Status**: Historical pre-`M2` implementation control snapshot.",
            "It no longer describes the current repository state.",
        ),
        must_not_contain=(
            "The repository has completed `M0` and is now entering `M1`:",
            "- `engine-daemon` and `mcp-server` are still stubs",
            "- process stub only",
        ),
    ),
    TextCheck(
        path="docs/M1_TASK_CHECKLIST.md",
        must_contain=(
            "**Status**: Historical `M1` implementation checklist.",
            "Current repo status lives in `specs/PROGRESS.md`.",
        ),
        must_not_contain=(
            "- [`crates/engine-daemon/src/main.rs`](../crates/engine-daemon/src/main.rs): still stub",
            "- [`mcp-server/server.py`](../mcp-server/server.py): still stub",
        ),
    ),
    TextCheck(
        path="docs/MCP_DESIGN.md",
        must_contain=(
            "Tool-availability notes below include historical planning text; current",
            "Methods that were previously staged later in planning but are now implemented",
            "Current `M2` availability is defined by `specs/MCP_API_SPEC.md` and",
        ),
        must_not_contain=(
            "#### `close_project`\nClose the currently open project.\nStatus: Target M2+.",
            "Status: Target M2+.\n\n#### `get_part`",
            "Specified for later slices, but not currently implemented in the daemon/stdio host:\n- `get_netlist`\n- `get_design_rules`\n- `explain_violation`",
            "#### `get_design_rules`\nAll configured rules with scopes and values.\nStatus: Target M2+.",
            "#### `explain_violation`\nNatural language explanation of a DRC violation with context.\n```json\nInput:  {\"violation_index\": 0}\nOutput: {\"explanation\": \"Two tracks on the Top layer are 0.08mm apart,\n          but the clearance rule for net class 'default' requires 0.1mm.\n          The tracks belong to nets VCC and GND near component U1.\",\n         \"suggestion\": \"Move track segment or increase spacing.\"}\n```\nStatus: Target M2+.",
        ),
    ),
    TextCheck(
        path="docs/USER_WORKFLOWS.md",
        must_contain=(
            "`explain_violation` is part of the current `M2` MCP surface.",
        ),
        must_not_contain=(
            "`explain_violation` is `Target M2+`; in the current slice, the agent explains",
        ),
    ),
    TextCheck(
        path="CLAUDE.md",
        must_contain=(
            "├── crates/",
            "│   ├── engine/",
            "│   ├── cli/",
            "│   └── test-harness/",
            "specs/",
        ),
        must_not_contain=(
            "├── engine/                 # Core engine (Rust library crate)",
            "├── cli/                    # CLI binary (Rust, depends on engine)",
        ),
    ),
)


def run_command(command: list[str]) -> tuple[int, str]:
    completed = subprocess.run(
        command,
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        check=False,
    )
    return completed.returncode, completed.stdout


def run_text_checks() -> list[str]:
    failures: list[str] = []
    for check in TEXT_CHECKS:
        text = (ROOT / check.path).read_text()
        for needle in check.must_contain:
            if needle not in text:
                failures.append(f"{check.path}: missing required text: {needle!r}")
        for needle in check.must_not_contain:
            if needle in text:
                failures.append(f"{check.path}: found stale text: {needle!r}")
    return failures


def run_exec_check(label: str, command: list[str], failures: list[str]) -> None:
    code, output = run_command(command)
    if code != 0:
        failures.append(
            f"{label}: command failed with exit code {code}\n"
            f"command: {' '.join(command)}\n"
            f"output:\n{output.strip()}"
        )


def run_m3_preflight_check(label: str, command: list[str], failures: list[str]) -> None:
    code, output = run_command(command)
    normalized = output.strip()
    if code not in (0, 3):
        failures.append(
            f"{label}: command failed with unexpected exit code {code}\n"
            f"command: {' '.join(command)}\n"
            f"output:\n{normalized}"
        )
        return

    if "\"schema_version\":1" not in normalized:
        failures.append(f"{label}: missing schema_version in JSON output")
    if "\"overall_status\":\"deferred\"" not in normalized and "\"overall_status\":\"passed\"" not in normalized:
        failures.append(f"{label}: output missing expected overall_status deferred/passed")
    if code == 3 and "\"overall_status\":\"deferred\"" not in normalized:
        failures.append(f"{label}: exit code 3 must correspond to deferred status")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--run-gates",
        action="store_true",
        help="Run current milestone executable gates in addition to static checks.",
    )
    parser.add_argument(
        "--run-health",
        action="store_true",
        help="Run broader workspace health checks in addition to static checks.",
    )
    parser.add_argument(
        "--run-m3-preflight",
        action="store_true",
        help="Run M3 preflight evidence hooks and require documented deferred/pass behavior.",
    )
    args = parser.parse_args()

    failures = run_text_checks()

    if args.run_gates:
        run_exec_check(
            "decomposition_coverage",
            ["python3", "scripts/check_decomposition_coverage.py"],
            failures,
        )
        run_exec_check(
            "touched_monolith_growth",
            ["python3", "scripts/check_touched_monolith_growth.py"],
            failures,
        )
        run_exec_check(
            "source_file_size_budgets",
            ["python3", "scripts/check_file_size_budgets.py"],
            failures,
        )
        run_exec_check(
            "m2_quality",
            ["cargo", "run", "-q", "-p", "eda-test-harness", "--bin", "m2_quality", "--", "--json"],
            failures,
        )
        run_exec_check(
            "m2_perf",
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "eda-test-harness",
                "--bin",
                "m2_perf",
                "--",
                "--iterations",
                "3",
                "--compare-baseline",
                "crates/test-harness/testdata/perf/m2_doa2526_baseline.json",
            ],
            failures,
        )
        run_exec_check(
            "mcp_self_test",
            ["python3", "mcp-server/server.py", "--self-test"],
            failures,
        )

    if args.run_health:
        run_exec_check(
            "workspace_health",
            ["cargo", "test", "-q"],
            failures,
        )

    if args.run_m3_preflight:
        run_m3_preflight_check(
            "m3_op_determinism",
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "eda-test-harness",
                "--bin",
                "m3_op_determinism",
                "--",
                "--json",
            ],
            failures,
        )
        run_m3_preflight_check(
            "m3_undo_redo_roundtrip",
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "eda-test-harness",
                "--bin",
                "m3_undo_redo_roundtrip",
                "--",
                "--json",
            ],
            failures,
        )
        run_m3_preflight_check(
            "m3_write_surface_parity",
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "eda-test-harness",
                "--bin",
                "m3_write_surface_parity",
                "--",
                "--json",
            ],
            failures,
        )

    if failures:
        print("Alignment audit failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print("Alignment audit passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
