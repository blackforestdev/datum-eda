"""PM059 feature-workflow and development-availability validation.

This checks task structure and evidence, not agent effort, independent approval,
or the truth of producer observations. Existing authority checks remain separate.
"""
from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any

from history_commit_locator import commit_exists

FIELDS = {
    "ordinary_workflow", "safety_requirements", "qualification_followups",
    "prerequisite_reasons", "development_ready", "ordinary_evidence",
}


def _strings(value: Any, *, required: bool = False) -> bool:
    return (
        isinstance(value, list) and (bool(value) or not required)
        and all(isinstance(x, str) and bool(x.strip()) for x in value)
        and len(value) == len(set(value))
    )


def _ordinary_record(root: Path, path: str, contract: dict) -> list[str]:
    rel = Path(path)
    if rel.is_absolute() or ".." in rel.parts:
        return ["ordinary evidence must be a repository-relative path"]
    resolved = (root / rel).resolve()
    if not resolved.is_relative_to(root.resolve()):
        return ["ordinary evidence cannot escape the repository through a symlink"]
    try:
        record = json.loads(resolved.read_text())
    except (OSError, ValueError):
        return [f"ordinary evidence is missing or invalid JSON: {path}"]
    if not isinstance(record, dict) or record.get("kind") != "ordinary_application":
        return [f"ordinary evidence must describe an ordinary application run: {path}"]
    failures = []
    source = record.get("source_commit", "")
    if not re.fullmatch(r"[0-9a-f]{40}", str(source)) or not commit_exists(root, str(source)):
        failures.append(f"ordinary evidence needs an exact source commit: {path}")
    if not isinstance(record.get("launch_command"), str) or not record["launch_command"].strip():
        failures.append(f"ordinary evidence needs its launch command: {path}")
    for field, expected in [
        ("workflow_results", contract["ordinary_workflow"]),
        ("safety_results", contract["safety_requirements"]),
    ]:
        results = record.get(field)
        if not isinstance(results, dict) or any(results.get(x) is not True for x in expected):
            failures.append(f"ordinary evidence lacks passing {field}: {path}")
    return failures


def validate_feature_delivery(root: Path, item: dict, issue: dict) -> list[str]:
    contract = item.get("feature_delivery")
    purpose_record = item.get("work_purpose")
    purpose = purpose_record.get("kind") if isinstance(purpose_record, dict) else None
    required = ((item.get("canonical_next") or item.get("state") == "in_progress")
                and (purpose == "product_delivery" or purpose is None and issue.get("issue_type") == "feature"))
    label = str(item.get("key", "?")) + ": feature delivery"
    if contract is None:
        return [f"{label} requires an ordinary-workflow contract"] if required else []
    if not isinstance(contract, dict) or set(contract) != FIELDS:
        return [f"{label} must have exactly {', '.join(sorted(FIELDS))}"]
    failures = []
    for field in ["ordinary_workflow", "safety_requirements", "qualification_followups", "ordinary_evidence"]:
        if not _strings(contract[field], required=field in {"ordinary_workflow", "safety_requirements"}):
            failures.append(f"{label}: {field} must contain unique non-empty strings")
    reasons = contract["prerequisite_reasons"]
    if not isinstance(reasons, dict) or any(
        not isinstance(k, str) or not k.strip() or not isinstance(v, str) or not v.strip()
        for k, v in reasons.items()
    ):
        failures.append(f"{label}: prerequisite_reasons must name concrete blockers")
        reasons = {}
    dependencies = item.get("dependencies", [])
    if not isinstance(dependencies, list):
        return failures + [f"{label}: dependencies must be a list"]
    for dependency in dependencies:
        if not isinstance(dependency, str):
            failures.append(f"{label}: dependency must be an issue ID")
            continue
        if not reasons.get("issue:" + dependency):
            failures.append(f"{label}: unexplained hard prerequisite {dependency}")
    completion = item.get("completion", {})
    if not isinstance(completion, dict):
        return failures + [f"{label}: completion must be an object"]
    selected = completion.get("canonical_next_step_id")
    steps = completion.get("steps", [])
    if not isinstance(steps, list):
        return failures + [f"{label}: completion steps must be a list"]
    step = next((s for s in steps if isinstance(s, dict) and s.get("id") == selected), {})
    if step.get("kind") in {"planning", "owner_decision", "governance"} and not reasons.get("step:" + str(selected)):
        failures.append(f"{label}: non-implementation step {selected} needs its concrete blocking reason")
    if type(contract["development_ready"]) is not bool:
        failures.append(f"{label}: development_ready must be boolean")
    elif contract["development_ready"]:
        if not contract["ordinary_evidence"]:
            failures.append(f"{label}: development-ready needs ordinary application evidence")
        elif not failures:
            for path in contract["ordinary_evidence"]:
                failures.extend(f"{label}: {failure}" for failure in _ordinary_record(root, path, contract))
    return failures


def delivery_lines(view: dict) -> list[str]:
    contract = view.get("feature_delivery")
    if not isinstance(contract, dict):
        if (view.get("work_purpose") or {}).get("kind") == "product_delivery":
            return ["Delivery contract: required before selection or work start; none recorded. No availability claimed."]
        return []
    lines = ["Ordinary user workflow: " + " -> ".join(contract["ordinary_workflow"])]
    status = "available; full qualification remains separate" if contract["development_ready"] else "not available"
    lines.append("Development build: " + status)
    lines.append("Safety requirements: " + "; ".join(contract["safety_requirements"]))
    lines.append("Qualification followups: " + ("; ".join(contract["qualification_followups"]) or "none recorded"))
    lines.append("Prerequisite reasons: " + ("; ".join(f"{key} — {value}" for key, value in contract["prerequisite_reasons"].items()) or "none"))
    lines.append("Ordinary application evidence: " + (", ".join(contract["ordinary_evidence"]) or "none recorded"))
    if (not view.get("decision_input_required", view.get("owner_input_required"))
            and not view.get("hold")
            and ("canonical_next" not in view or view["canonical_next"])
            and ("state" not in view or view["state"] not in {"blocked", "deferred", "landed"})):
        lines.append(
            "Delivery rule: continue within the recorded grant through implementation, focused safety checks, "
            "repairs and required review; stop only for delivery or a concrete authority, safety, review/transport "
            "or ownership blocker. Commits, documentation and lease updates are not delivery."
        )
    return lines
