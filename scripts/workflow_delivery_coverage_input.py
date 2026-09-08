"""Malformed synthetic coverage INPUT, never real roadmap reclassification."""

from copy import deepcopy

COVERAGE_INPUT_CASES = {
    "INFRA-S02-01.missing-classification": (
        "specs/active_frontier.json", "WDQ-COVERAGE", "classification mismatch: missing=['NEXT'], absent=[]"),
    "INFRA-S02-01.duplicate-classification": (
        "specs/workflow_delivery_policy.json", "WDQ-TRUST", "selected authority has no valid promoted policy"),
}


def coverage_expectation(variant, surface):
    if variant == "INFRA-S02-01.duplicate-classification" and surface.startswith("S-"):
        return "coverage.rows", "WDQ-POLICY", "duplicate identity"
    return COVERAGE_INPUT_CASES[variant]


def prepare_coverage_input(policy, variant):
    if variant not in COVERAGE_INPUT_CASES:
        raise ValueError("explicit implemented coverage-input variant required")
    rows = policy["coverage"]["rows"]
    if variant.endswith(".missing-classification"):
        policy["coverage"]["rows"] = [row for row in rows if row["frontier_key"] != "NEXT"]
    else:
        rows.append(deepcopy(rows[0]))
