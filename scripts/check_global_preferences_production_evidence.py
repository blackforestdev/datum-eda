#!/usr/bin/env python3
"""Compare GP-CM05 measurements and proof status with the governed budgets."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MATRIX = ROOT / "specs/global_preferences_production_acceptance_matrix.json"
EVIDENCE = ROOT / "specs/evidence/preferences/gp-cm05-candidate-20260907.json"


def failures(require_acceptance: bool = False) -> list[str]:
    matrix = json.loads(MATRIX.read_text(encoding="utf-8"))
    evidence = json.loads(EVIDENCE.read_text(encoding="utf-8"))
    budgets = {entry["id"]: entry["limit"] for entry in matrix["candidate_resource_budgets"]}
    measured = evidence["measurement"]
    problems: list[str] = []

    def timed(name: str, budget: str) -> None:
        sample = measured[name]
        limit = budgets[budget]
        if sample["p95_ms"] > limit["p95_ms"]:
            problems.append(f"{name} p95 exceeds {limit['p95_ms']} ms")
        if sample["max_ms"] > limit["max_ms"]:
            problems.append(f"{name} maximum exceeds {limit['max_ms']} ms")
        if sample["max_rss_kib"] > limit["max_rss_mib"] * 1024:
            problems.append(f"{name} RSS exceeds {limit['max_rss_mib']} MiB")

    timed("release_query", "release-query")
    timed("durable_mutation", "durable-mutation")
    timed("project_genesis_factory", "project-genesis")
    timed("project_genesis_global_no_file", "project-genesis")
    timed("project_validation", "project-validation")

    storage = budgets["storage-growth"]
    project = measured["native_project"]
    if project["maximum_bytes"] > storage["empty_project_kib"] * 1024:
        problems.append("empty native Project exceeds its storage budget")
    repository = measured["preference_repository"]
    if repository["created_by_read_or_factory_paths"]:
        problems.append("a read-only or factory path created a preference repository")
    if repository["bytes_after_20_mutations"] / repository["immutable_generations"] > (
        storage["per_preference_generation_kib"] * 1024
    ):
        problems.append("preference generation growth exceeds its storage budget")
    if evidence["correctness_proof"]["status"] != "passed_outside_managed_sandbox":
        problems.append("bounded correctness proof is not passing")

    comparison = evidence["budget_comparison"]
    completed = {key for key, value in comparison.items() if value == "pass"}
    required_candidate = {
        "release_query",
        "durable_mutation",
        "project_genesis",
        "project_validation",
        "storage_growth",
    }
    if not required_candidate <= completed:
        problems.append("measured candidate budget results are incomplete")
    if require_acceptance:
        if set(comparison.values()) != {"pass"}:
            problems.append("one or more final GUI/window budgets are not passing")
        if not evidence["acceptance"]["production_accepted"]:
            problems.append("owner production acceptance is not recorded")
    elif evidence["acceptance"]["production_accepted"]:
        problems.append("candidate evidence must not claim owner production acceptance")
    return problems


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--acceptance", action="store_true")
    arguments = parser.parse_args()
    problems = failures(arguments.acceptance)
    if problems:
        for problem in problems:
            print(f"Global Preferences production evidence: {problem}")
        return 1
    state = "final acceptance" if arguments.acceptance else "candidate"
    print(f"Global Preferences production evidence {state} gate passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
