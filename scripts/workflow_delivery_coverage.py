"""Coverage identity and baseline checks over exact candidate Git views.

This module is not a trust source. Its caller must supply owner-pinned coverage
and independently validate full PM025 state and delivery phase obligations.
"""

from workflow_delivery_authority import resolve_ref
from workflow_delivery_coverage_shapes import coverage_shape
from workflow_delivery_shapes import require
from workflow_delivery_tree import Tree


FRONTIER = "specs/active_frontier.json"


def index_items(manifest):
    items = manifest.get("frontier")
    require(type(items) is list and bool(items), FRONTIER,
            "nonempty Frontier required for coverage", "WDQ-COVERAGE")
    result = {}
    for item in items:
        require(type(item) is dict and type(item.get("key")) is str,
                FRONTIER, "Frontier object with key required", "WDQ-COVERAGE")
        key = item["key"]
        require(key not in result, key, "duplicate Frontier identity", "WDQ-COVERAGE")
        result[key] = item
    return result


def validate_classifications(tree, coverage, manifest, *, authority):
    """Reject omitted/foreign identities and historical reopening; read only."""
    coverage_shape(coverage)
    current = index_items(manifest)
    rows = {row["frontier_key"]: row for row in coverage["rows"]}
    require(set(rows) == set(current), FRONTIER,
            f"classification mismatch: missing={sorted(set(current) - set(rows))}, "
            f"absent={sorted(set(rows) - set(current))}", "WDQ-COVERAGE")
    tree.has_commit(coverage["baseline_ref"])
    baseline = Tree(tree.root, revision=coverage["baseline_ref"])
    previous = index_items(baseline.json(FRONTIER))
    for key, row in rows.items():
        item = current[key]
        require(item.get("issue_id") == row["issue_id"], key,
                "classified issue identity changed", "WDQ-COVERAGE")
        pinned_reference(tree, authority, row["boundary_ref"])
        if row["external_handoff_ref"] is not None:
            pinned_reference(tree, authority, row["external_handoff_ref"])
        if row["category"] == "historical":
            old = previous.get(key, {})
            require(old.get("state") == "landed" and
                    old.get("issue_id") == row["issue_id"], key,
                    "historical classification requires landed baseline identity",
                    "WDQ-COVERAGE")
            require(item.get("state") == "landed" and
                    item.get("authorization") == "none" and not item.get("claim"), key,
                    "historical reopening requires reclassification", "WDQ-COVERAGE")
            before, after = old.get("completion", {}), item.get("completion", {})
            # Live post-completion unblocks legitimately shrink as successors
            # close. Freeze obligations, not that current backlog projection.
            require(all(after.get(field) == before.get(field)
                        for field in ("steps", "delivery", "outcome")) and
                    item.get("landing_commit") == old.get("landing_commit"), key,
                    "historical completion/landing evidence changed", "WDQ-COVERAGE")
    for scope in coverage["source_scopes"]:
        key = scope["frontier_key"]
        steps = current[key].get("completion", {}).get("steps", [])
        kinds = {s["id"]: s["kind"] for s in steps}
        require(all(kinds.get(s) == "execution" for s in scope["step_ids"]), key,
                "source scopes require existing execution steps", "WDQ-COVERAGE")
        require(rows[key]["category"] in ("product", "infrastructure", "external_lane"),
                key, "non-executing classification cannot hold source scope", "WDQ-COVERAGE")
        pinned_reference(tree, authority, scope["boundary_ref"])
    return current


def pinned_reference(tree, authority, reference):
    require(resolve_ref(tree, reference) == resolve_ref(authority, reference),
            reference["path"], "coverage boundary changed after owner promotion",
            "WDQ-AUTHORITY")
