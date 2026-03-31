#!/usr/bin/env python3
"""Run the checked-in KiCad import/query corpus repeatedly and fail on output drift."""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
MANIFEST_PATH = (
    ROOT
    / "crates/test-harness/testdata/quality/import_query_determinism_manifest_v1.json"
)
CLI_BIN = ROOT / "target/debug/eda"


def canonicalize_json(payload: str) -> str:
    return json.dumps(json.loads(payload), sort_keys=True, separators=(",", ":"))


def build_cli() -> None:
    completed = subprocess.run(
        ["cargo", "build", "-q", "-p", "eda-cli"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if completed.returncode != 0:
        sys.stderr.write(completed.stderr)
        raise SystemExit(completed.returncode)


def run_case(command: list[str]) -> tuple[str, int]:
    completed = subprocess.run(
        [str(CLI_BIN), "--format", "json", *command],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if completed.returncode not in (0, 1):
        sys.stderr.write(completed.stderr)
        raise SystemExit(completed.returncode)
    return canonicalize_json(completed.stdout), completed.returncode


def evaluate_query_case(case: dict, repeat_count: int) -> dict:
    fixture_path = str(ROOT / case["fixture_path"])
    runs = []
    for index in range(repeat_count):
        canonical, exit_code = run_case(["query", fixture_path, case["subcommand"]])
        runs.append(
            {
                "run": index + 1,
                "exit_code": exit_code,
                "sha256": hashlib.sha256(canonical.encode("utf-8")).hexdigest(),
                "canonical": canonical,
            }
        )
    return summarize_case("query", case["case_id"], runs, case["fixture_path"], case["subcommand"])


def evaluate_check_case(case: dict, repeat_count: int) -> dict:
    fixture_path = str(ROOT / case["fixture_path"])
    runs = []
    for index in range(repeat_count):
        canonical, exit_code = run_case(["check", fixture_path])
        runs.append(
            {
                "run": index + 1,
                "exit_code": exit_code,
                "sha256": hashlib.sha256(canonical.encode("utf-8")).hexdigest(),
                "canonical": canonical,
            }
        )
    return summarize_case("check", case["case_id"], runs, case["fixture_path"], None)


def summarize_case(
    kind: str,
    case_id: str,
    runs: list[dict],
    fixture_path: str,
    subcommand: str | None,
) -> dict:
    first = runs[0]
    drifted_runs = [
        run["run"]
        for run in runs[1:]
        if run["sha256"] != first["sha256"] or run["exit_code"] != first["exit_code"]
    ]
    result = {
        "case_id": case_id,
        "kind": kind,
        "fixture_path": fixture_path,
        "stable": not drifted_runs,
        "baseline_exit_code": first["exit_code"],
        "baseline_sha256": first["sha256"],
        "drifted_runs": drifted_runs,
    }
    if subcommand is not None:
        result["subcommand"] = subcommand
    if drifted_runs:
        result["run_fingerprints"] = [
            {
                "run": run["run"],
                "exit_code": run["exit_code"],
                "sha256": run["sha256"],
            }
            for run in runs
        ]
    return result


def main() -> int:
    manifest = json.loads(MANIFEST_PATH.read_text())
    repeat_count = manifest.get("repeat_count", 3)
    build_cli()

    results = []
    failures = []

    for case in manifest["query_cases"]:
        result = evaluate_query_case(case, repeat_count)
        results.append(result)
        if not result["stable"]:
            failures.append(result)

    for case in manifest["check_cases"]:
        result = evaluate_check_case(case, repeat_count)
        results.append(result)
        if not result["stable"]:
            failures.append(result)

    query_results = [result for result in results if result["kind"] == "query"]
    check_results = [result for result in results if result["kind"] == "check"]
    payload = {
        "summary": {
            "action": "check_import_query_determinism",
            "manifest_path": str(MANIFEST_PATH.relative_to(ROOT)),
            "manifest_kind": manifest["kind"],
            "manifest_version": manifest["version"],
            "repeat_count": repeat_count,
            "total_cases": len(results),
            "query_cases": len(query_results),
            "check_cases": len(check_results),
            "stable_cases": len(results) - len(failures),
            "drifted_cases": len(failures),
        },
        "results": results,
    }
    if failures:
        payload["failures"] = failures
    print(json.dumps(payload, indent=2, sort_keys=True))
    return 0 if not failures else 1


if __name__ == "__main__":
    raise SystemExit(main())
