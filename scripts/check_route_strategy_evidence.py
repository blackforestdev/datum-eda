#!/usr/bin/env python3
"""Regenerate the curated M6 route-strategy evidence run and gate it to the checked-in baseline."""

from __future__ import annotations

import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
BASELINE_DIR = (
    ROOT / "crates/test-harness/testdata/quality/route_strategy_curated_baseline_v1"
)
BASELINE_ARTIFACT = BASELINE_DIR / "route-strategy-batch-result.json"


def run_eda(*args: str) -> dict:
    completed = subprocess.run(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "eda-cli",
            "--",
            "--format",
            "json",
            "project",
            *args,
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if completed.returncode not in (0, 2):
        sys.stderr.write(completed.stderr)
        raise SystemExit(completed.returncode)
    return json.loads(completed.stdout)


def main() -> int:
    if not BASELINE_ARTIFACT.is_file():
        raise SystemExit(
            f"missing checked-in route-strategy baseline artifact: {BASELINE_ARTIFACT}"
        )

    baseline_validation = run_eda(
        "validate-route-strategy-batch-result",
        str(BASELINE_ARTIFACT),
    )
    if not baseline_validation["structurally_valid"]:
        print(json.dumps({"action": "check_route_strategy_evidence", "status": "invalid_baseline", "baseline_validation": baseline_validation}, indent=2))
        return 1

    with tempfile.TemporaryDirectory(prefix="datum-route-strategy-evidence-") as temp_dir:
        out_dir = Path(temp_dir) / "curated-run"
        fresh_capture = run_eda(
            "capture-route-strategy-curated-baseline",
            "--out-dir",
            str(out_dir),
        )
        fresh_artifact = Path(fresh_capture["result_artifact_path"])
        comparison = run_eda(
            "compare-route-strategy-batch-result",
            str(BASELINE_ARTIFACT),
            str(fresh_artifact),
        )
        gate = run_eda(
            "gate-route-strategy-batch-result",
            str(BASELINE_ARTIFACT),
            str(fresh_artifact),
            "--policy",
            "strict_identical",
        )
        summary = run_eda(
            "summarize-route-strategy-batch-results",
            "--artifact",
            str(BASELINE_ARTIFACT),
            "--artifact",
            str(fresh_artifact),
            "--baseline",
            str(BASELINE_ARTIFACT),
            "--policy",
            "strict_identical",
        )

        report = {
            "action": "check_route_strategy_evidence",
            "status": "ok" if gate["passed"] else "gate_failed",
            "baseline_artifact": str(BASELINE_ARTIFACT),
            "fresh_artifact": str(fresh_artifact),
            "baseline_validation": baseline_validation,
            "fresh_capture": {
                "suite_id": fresh_capture["suite_id"],
                "summary": fresh_capture["summary"],
            },
            "comparison_classification": comparison["comparison_classification"],
            "gate": {
                "selected_gate_policy": gate["selected_gate_policy"],
                "passed": gate["passed"],
                "comparison_classification": gate["comparison_classification"],
                "pass_fail_reasons": gate["pass_fail_reasons"],
            },
            "summary": summary["summary"],
        }
        print(json.dumps(report, indent=2, sort_keys=True))
        return 0 if gate["passed"] else 2


if __name__ == "__main__":
    raise SystemExit(main())
