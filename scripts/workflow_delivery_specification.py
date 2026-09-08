"""Check completed specification outputs, not semantic completeness or approval."""

from workflow_delivery_authority import evidence_routes, resolve_ref, reviewed_route_members
from workflow_delivery_shapes import require


def validate_specification_outputs(tree, items, coverage):
    completed = [
        (row["frontier_key"], step)
        for row in coverage["rows"] if row["category"] == "specification"
        for step in items[row["frontier_key"]]["completion"]["steps"]
        if step["status"] == "complete"
    ]
    if not completed:
        return  # Pending authorship does not require future outputs.
    routes = evidence_routes(tree)
    checked = set()
    for key, step in completed:
        refs = [proof for proof in step["completion_evidence"]
                if proof["kind"] in ("document", "review", "decision")]
        require(bool(refs), key,
                f"{step['id']}: completed specification requires governed output references",
                "WDQ-COVERAGE")
        for reference in refs:
            resolve_ref(tree, reference)
            owners = {name for name, route in routes.items()
                      if reference["path"] in route["sources"] + route["consumers"]}
            require(bool(owners), reference["path"],
                    "completed specification output requires an owning evidence route",
                    "WDQ-AUTHORITY")
            for name in sorted(owners - checked):
                reviewed_route_members(tree, name, routes[name])
                checked.add(name)
