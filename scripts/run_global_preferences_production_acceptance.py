#!/usr/bin/env python3
"""Run the exact GP-CM05 correctness, governance, and release proof."""

from __future__ import annotations

import argparse
from pathlib import Path
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[1]
PYTHON = sys.executable
GUARD = [PYTHON, str(ROOT / "scripts/run_cargo_guarded.py"), "--workload", "proof", "--"]


def run(command: list[str]) -> None:
    subprocess.run(command, cwd=ROOT, check=True)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--full-workspace",
        action="store_true",
        help="also run the serial full-workspace test and strict-Clippy closure gates",
    )
    parser.add_argument(
        "--measure",
        action="store_true",
        help="also rebuild and measure the optimized CLI product paths",
    )
    arguments = parser.parse_args()

    for checker in (
        "check_global_preferences_production_matrix.py",
        "check_global_preferences_production_evidence.py",
        "check_global_preferences_boundary.py",
        "check_menu_model.py",
        "check_resolver_raw_loads.py",
        "check_daemon_write_parity.py",
        "check_mcp_public_taxonomy.py",
        "check_dependency_authority.py",
        "check_cargo_resource_policy.py",
        "check_source_health.py",
        "check_spec_governance.py",
        "check_spec_parity.py",
        "check_evidence_traceability.py",
        "check_progress_coverage.py",
        "check_alignment.py",
    ):
        run([PYTHON, str(ROOT / "scripts" / checker)])
    run([PYTHON, str(ROOT / "scripts/project_status.py"), "check"])
    run([PYTHON, str(ROOT / "scripts/run_global_preferences_proof.py")])

    if arguments.full_workspace:
        run([*GUARD, "cargo", "test", "--workspace", "--all-targets", "--locked", "--offline"])
        run(
            [
                *GUARD,
                "cargo",
                "clippy",
                "--workspace",
                "--all-targets",
                "--locked",
                "--offline",
                "--",
                "-D",
                "warnings",
            ]
        )
    if arguments.measure:
        run([*GUARD, "cargo", "build", "--release", "-p", "datum-eda-cli", "--locked", "--offline"])
        run([PYTHON, str(ROOT / "scripts/measure_global_preferences_release.py"), "--samples", "20"])

    print("GP-CM05 production-candidate proof: PASS")
    print("Boundary: 11 active Global settings, 45 reserved candidates, eight Project Units seeds.")
    print("Exclusions: no live Project following, reserved activation, private writer, direct MCP Set/Reset, Publish, or Revision.")
    print("Disposition: evidence only; GP-CM05V owner budget and production-acceptance decision remains required.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
