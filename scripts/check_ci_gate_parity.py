#!/usr/bin/env python3
"""Tripwire for the tripwires.

The comprehensive gate set lives in scripts/run_drift_gates.sh (governance,
spec-parity, contract locks, private-writer/daemon-write bans, MCP taxonomy, and
all PG-* proof gates). Those enforce nothing during development unless CI runs
them. This meta-gate asserts that CI (.github/workflows/alignment.yml) invokes
scripts/run_drift_gates.sh, so the full set actually fires on every push/PR and
cannot silently diverge from the manual runner. It also asserts the runner still
invokes the governance and spec-parity gates.

Without this, "enforced" degrades to "enforced only when someone remembers to
run the gates by hand" -- the exact failure this gate exists to prevent.
"""

from __future__ import annotations

import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
CI = ROOT / ".github/workflows/alignment.yml"
RUNNER = ROOT / "scripts/run_drift_gates.sh"

# Gates that MUST fire automatically in CI (via the runner) -- the enforcement
# that was previously manual-only.
REQUIRED_IN_RUNNER = [
    "check_spec_governance.py",
    "check_spec_parity.py",
    "check_product_north_star.py",
    "check_library_foundation_contract.py",
    "check_schematic_private_writers.py",
    "check_daemon_write_parity.py",
    "check_mcp_public_taxonomy.py",
    "run_migration_proof_gates.sh",
]


def main() -> int:
    failures: list[str] = []

    ci = CI.read_text(encoding="utf-8") if CI.exists() else ""
    runner = RUNNER.read_text(encoding="utf-8") if RUNNER.exists() else ""

    # Require an actual invocation, not a mere mention in a comment or step name.
    invoked = False
    for line in ci.splitlines():
        stripped = line.strip()
        if stripped.startswith("#") or stripped.startswith("- name:") or stripped.startswith("name:"):
            continue
        if "run_drift_gates.sh" in stripped:
            invoked = True
            break
    if not invoked:
        failures.append(
            ".github/workflows/alignment.yml must actually invoke scripts/run_drift_gates.sh "
            "in a run step (not just mention it) so the full drift-gate set runs in CI on "
            "every push/PR — not only on manual runs"
        )

    for required in REQUIRED_IN_RUNNER:
        if required not in runner:
            failures.append(
                f"scripts/run_drift_gates.sh must invoke {required} so it is CI-enforced"
            )

    if failures:
        print("CI gate-parity check failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print(
        "CI gate-parity check passed (CI invokes run_drift_gates.sh; governance, "
        "spec-parity, contract, write-ban, taxonomy, and proof gates are CI-enforced)."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
