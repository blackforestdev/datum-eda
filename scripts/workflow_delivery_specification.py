"""Check completed specification outputs, not semantic completeness or approval."""

from workflow_delivery_authority import evidence_routes, resolve_ref, reviewed_route_members
from workflow_delivery_clause_inventory import clause_inventory
from workflow_delivery_clause_dispositions import validate_clause_dispositions
from workflow_delivery_shapes import require


def validate_specification_outputs(tree, items, coverage, authority):
    completed = []
    for row in coverage["rows"]:
        if row["category"] != "specification":
            continue
        item = items[row["frontier_key"]]
        clauses = clause_inventory(tree, item, row["boundary_ref"])
        completed.extend((item, step, clauses) for step in item["completion"]["steps"]
                         if step["status"] == "complete")
    if not completed:
        return  # Pending authorship does not require future outputs.
    routes = evidence_routes(tree)
    checked = set()
    def check_ref(reference):
        resolve_ref(tree, reference)
        owners = {name for name, route in routes.items()
                  if reference["path"] in route["sources"] + route["consumers"]}
        require(bool(owners), reference["path"],
                "completed specification output requires an owning evidence route",
                "WDQ-AUTHORITY")
        for name in sorted(owners - checked):
            reviewed_route_members(tree, name, routes[name])
            checked.add(name)

    for item, step, clauses in completed:
        key = item["key"]
        if step["kind"] == "owner_decision":
            promoted = next((i for i in authority.json("specs/active_frontier.json")["frontier"]
                             if i["key"] == key), {})
            approved = next((s for s in promoted.get("completion", {}).get("steps", [])
                             if s["id"] == step["id"]), {})
            require(approved == step, key,
                    "completed specification owner decision requires exact owner promotion",
                    "WDQ-AUTHORITY")
            for proof in step["completion_evidence"]:
                if proof["kind"] in ("document", "review", "decision"):
                    require(resolve_ref(tree, proof) == resolve_ref(authority, proof),
                            proof["path"], "owner decision evidence changed after promotion",
                            "WDQ-AUTHORITY")
        boundary = next(r["boundary_ref"] for r in coverage["rows"] if r["frontier_key"] == key)
        check_ref(boundary)
        refs = [proof for proof in step["completion_evidence"]
                if proof["kind"] in ("document", "review", "decision")]
        require(bool(refs), key,
                f"{step['id']}: completed specification requires governed output references",
                "WDQ-COVERAGE")
        for reference in refs:
            check_ref(reference)
        validate_clause_dispositions(tree, authority, item, step, clauses, refs, check_ref)
