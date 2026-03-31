#!/usr/bin/env python3
"""Validate checked-in native fixture projects with the product-facing `project validate` surface."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
MANIFEST_PATH = (
    ROOT / "crates/test-harness/testdata/quality/native_project_validation_manifest_v1.json"
)


def run_eda_validate(project_root: Path) -> tuple[dict, int]:
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
            "validate",
            str(project_root),
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if completed.returncode not in (0, 1):
        sys.stderr.write(completed.stderr)
        raise SystemExit(completed.returncode)
    return json.loads(completed.stdout), completed.returncode


def main() -> int:
    manifest = json.loads(MANIFEST_PATH.read_text())
    fixtures = manifest["fixtures"]
    results: list[dict] = []
    failures: list[dict] = []

    for fixture in fixtures:
        project_root = ROOT / fixture["project_root"]
        report, exit_code = run_eda_validate(project_root)
        passed = report["valid"] == fixture["expected_valid"]
        result = {
            "fixture_id": fixture["fixture_id"],
            "project_root": fixture["project_root"],
            "expected_valid": fixture["expected_valid"],
            "actual_valid": report["valid"],
            "exit_code": exit_code,
            "issue_count": report["issue_count"],
        }
        results.append(result)
        if not passed:
            failures.append(
                {
                    **result,
                    "issues": report["issues"],
                }
            )

    summary = {
        "action": "check_native_project_fixtures",
        "manifest_path": str(MANIFEST_PATH.relative_to(ROOT)),
        "manifest_kind": manifest["kind"],
        "manifest_version": manifest["version"],
        "total_fixtures": len(fixtures),
        "passed_fixtures": len(results) - len(failures),
        "failed_fixtures": len(failures),
    }
    payload = {
        "summary": summary,
        "results": results,
    }
    if failures:
        payload["failures"] = failures
    print(json.dumps(payload, indent=2, sort_keys=True))
    return 0 if not failures else 1


if __name__ == "__main__":
    raise SystemExit(main())
