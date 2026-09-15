#!/usr/bin/env python3
"""Executable alignment audit for Datum EDA.

The historical static doc-string checks (pinned must_contain/must_not_contain
literals) were retired in the governance diet: behavioral enforcement lives in
the drift/proof gates, not in pinned prose. This script now only runs
executable verification modes:

  --run-gates         milestone executable gates (m2 quality/perf harnesses,
                      MCP self-test)
  --run-health        broader workspace health (cargo test -q)
  --run-m3-preflight  M3 preflight evidence hooks (documented deferred/pass)

The default no-flag invocation is a no-op that exits 0.
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent


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
        help="Run current milestone executable gates.",
    )
    parser.add_argument(
        "--run-health",
        action="store_true",
        help="Run broader workspace health checks.",
    )
    parser.add_argument(
        "--run-m3-preflight",
        action="store_true",
        help="Run M3 preflight evidence hooks and require documented deferred/pass behavior.",
    )
    args = parser.parse_args()

    if not (args.run_gates or args.run_health or args.run_m3_preflight):
        print(
            "Alignment audit: static doc-string checks were retired (governance diet); "
            "nothing to do without --run-gates / --run-health / --run-m3-preflight."
        )
        return 0

    failures: list[str] = []

    if args.run_gates:
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
