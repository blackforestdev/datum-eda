"""Owner-pinned clause inventory carried by a specification boundary reference."""

from workflow_delivery_shapes import (
    array, closed, identifier, ref, refs, require, unique, version,
)


def clause_inventory(tree, item, boundary):
    path = boundary["path"]
    value = tree.json(path)
    closed(value, "schema_version marker frontier_key clauses", path)
    version(value["schema_version"], path)
    require(value["marker"] == boundary["marker"] and value["frontier_key"] == item["key"],
            path, "clause inventory marker/Frontier identity mismatch", "WDQ-COVERAGE")
    array(value["clauses"], path)
    steps = {s["id"]: s for s in item["completion"]["steps"]}
    expected = {(s["id"], r["path"], r["marker"])
                for s in steps.values() for r in s["requirement_refs"]}
    covered = set()
    for clause in value["clauses"]:
        closed(clause, "id step_id requirement_ref authority_refs requires_owner_decision", path)
        identifier(clause["id"], path)
        identifier(clause["step_id"], path)
        ref(clause["requirement_ref"], path)
        refs(clause["authority_refs"], path)
        require(type(clause["requires_owner_decision"]) is bool, path,
                "requires_owner_decision must be boolean", "WDQ-COVERAGE")
        r = clause["requirement_ref"]
        identity = (clause["step_id"], r["path"], r["marker"])
        require(identity in expected, path,
                "clause must map an existing completion-step requirement", "WDQ-COVERAGE")
        covered.add(identity)
    unique([c["id"] for c in value["clauses"]], path, "WDQ-COVERAGE")
    require(covered == expected, path,
            "clause inventory must cover every completion-step requirement", "WDQ-COVERAGE")
    return value["clauses"]
