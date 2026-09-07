#!/usr/bin/env python3
"""Validate the bounded GP-CM05 production-acceptance corpus and budgets."""

from __future__ import annotations

import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MATRIX = ROOT / "specs/global_preferences_production_acceptance_matrix.json"

EXPECTED_CORPUS = {
    "clean-install",
    "upgrade",
    "downgrade-preservation",
    "corrupt-store",
    "portable-collision",
    "backup-restore",
    "project-genesis-crash",
    "project-genesis-replay",
    "existing-project-stability",
    "managed-offline",
    "surface-parity-negative",
    "human-agent-authority",
    "accessibility-interaction",
    "responsive-render",
    "noncolor-render",
}
EXPECTED_BUDGETS = {
    "release-query",
    "durable-mutation",
    "project-genesis",
    "project-validation",
    "storage-growth",
    "gui-feedback",
    "native-window-open",
    "window-lifecycle",
}


def failures(root: Path = ROOT) -> list[str]:
    try:
        matrix = json.loads((root / MATRIX.relative_to(ROOT)).read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        return [f"cannot read GP-CM05 matrix: {error}"]
    problems: list[str] = []
    if matrix.get("schema") != "datum-global-preferences-production-acceptance-matrix-v1":
        problems.append("matrix schema is missing or changed")
    if matrix.get("frontier_task") != "GLOBAL-PREFERENCES-COMPLETION":
        problems.append("matrix is not bound to GLOBAL-PREFERENCES-COMPLETION")
    if matrix.get("execution_step") != "GP-CM05":
        problems.append("matrix is not bound to GP-CM05")

    boundary = matrix.get("product_boundary", {})
    expected_boundary = {
        "active_global_descriptors": 11,
        "active_sections": ["appearance", "units"],
        "reserved_descriptors": 45,
        "project_seed_keys": 8,
        "project_seed_modes": ["global", "factory"],
        "live_project_following": False,
        "mcp_direct_set_or_reset": False,
    }
    for key, expected in expected_boundary.items():
        if boundary.get(key) != expected:
            problems.append(f"product boundary {key} must remain {expected!r}")

    corpus = matrix.get("durable_corpus", [])
    corpus_ids = {entry.get("id") for entry in corpus}
    if corpus_ids != EXPECTED_CORPUS or len(corpus) != len(EXPECTED_CORPUS):
        problems.append("durable corpus inventory is incomplete or duplicated")
    for entry in corpus:
        source = root / entry.get("source", "")
        test = entry.get("test", "")
        if not source.is_file():
            problems.append(f"corpus {entry.get('id')} source is missing")
            continue
        text = source.read_text(encoding="utf-8")
        if f"fn {test}(" not in text:
            problems.append(f"corpus {entry.get('id')} test {test!r} is missing")
        if not entry.get("authority"):
            problems.append(f"corpus {entry.get('id')} lacks an authority assertion")

    budgets = matrix.get("candidate_resource_budgets", [])
    budget_ids = {entry.get("id") for entry in budgets}
    if budget_ids != EXPECTED_BUDGETS or len(budgets) != len(EXPECTED_BUDGETS):
        problems.append("candidate resource-budget inventory is incomplete or duplicated")
    for entry in budgets:
        limits = entry.get("limit", {})
        if (
            not limits
            or any(not isinstance(value, (int, float)) or value < 0 for value in limits.values())
            or not any(value > 0 for value in limits.values())
        ):
            problems.append(f"budget {entry.get('id')} lacks bounded numeric limits")
    if "owner-dispositioned resource budget" not in matrix.get("acceptance_rule", ""):
        problems.append("acceptance rule does not preserve the owner budget boundary")
    return problems


def main() -> int:
    problems = failures()
    if problems:
        for problem in problems:
            print(f"Global Preferences production matrix: {problem}")
        return 1
    print("Global Preferences production matrix passed (15 corpus cases; 8 candidate budgets).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
