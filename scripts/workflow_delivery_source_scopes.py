"""Authorize transaction paths against promoted scopes and PM025 claims.

The production caller supplies the exact transaction diff, promoted coverage
and enrollment, and the same candidate manifest/tracker view. This helper never
infers changed paths from candidate labels or treats a scope as product proof.
"""

from datetime import datetime, timezone

from workflow_delivery_coverage import index_items
from workflow_delivery_coverage_shapes import coverage_shape
from workflow_delivery_io import normalized_path
from workflow_delivery_shapes import require


def contains(roots, path):
    return any(path == root or path.startswith(root + "/") for root in roots)


def authorize_source_paths(changed_paths, coverage, manifest, issues, enrolled, *, now=None):
    # Reuse the controlling claim validator; do not create a second lease model.
    from project_status import validate_claim

    require(type(changed_paths) in (list, tuple), "changed_paths",
            "explicit transaction path sequence required", "WDQ-COVERAGE")
    for path in changed_paths:
        normalized_path(path)
    coverage_shape(coverage)
    items = index_items(manifest)
    rows = {r["frontier_key"]: r for r in coverage["rows"]}
    require(set(rows) == set(items), "coverage", "classification mismatch", "WDQ-COVERAGE")
    moment = now or datetime.now(timezone.utc)
    ttl = manifest.get("claim_ttl_hours")
    require(type(ttl) is int and ttl > 0, "claim_ttl_hours",
            "positive PM025 claim TTL required", "WDQ-COVERAGE")
    granted = []
    for scope in coverage["source_scopes"]:
        key = scope["frontier_key"]
        item = items[key]
        row = rows[key]
        issue = issues.get(item.get("issue_id"), {})
        selected = item.get("completion", {}).get("canonical_next_step_id")
        steps = item.get("completion", {}).get("steps", [])
        step = next((s for s in steps if s.get("id") == selected), {})
        if not (row["issue_id"] == item.get("issue_id") and
                row["category"] in ("product", "infrastructure", "external_lane") and
                item.get("state") == "in_progress" and
                item.get("authorization") == "execution" and
                issue.get("status") == "in_progress" and
                selected in scope["step_ids"] and step.get("kind") == "execution" and
                step.get("status") == "in_progress"):
            continue
        failures = []
        validate_claim(item, issue, ttl, moment, failures)
        if failures:
            continue
        if row["category"] != "external_lane":
            if key not in enrolled or "delivery" not in item.get("completion", {}):
                continue
        # External permission is bounded by its promoted scope/reference, not
        # an exemption for any code written by a named session.
        granted.append(scope)
    checked = []
    for path in sorted(set(changed_paths)):
        normalized_path(path)
        if not contains(coverage["production_roots"], path):
            continue
        permitted = [s["frontier_key"] for s in granted if contains(s["paths"], path)]
        require(bool(permitted), path,
                "production change has no promoted scope with synchronized live "
                "execution claim and required enrollment", "WDQ-COVERAGE")
        checked.append({"path": path, "authorized_lanes": sorted(set(permitted))})
    return checked
