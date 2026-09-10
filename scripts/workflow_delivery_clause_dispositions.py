"""Exact clause accounting; no document can promote its own ratification."""

from workflow_delivery_authority import resolve_ref
from workflow_delivery_shapes import array, closed, enum, identifier, refs, require, unique, version


def validate_clause_dispositions(tree, authority, item, step, clauses, evidence, check_ref):
    marker = f"WDQ-CLAUSE-MATRIX.{item['key']}.{step['id']}"
    matrices = [r for r in evidence if r["marker"] == marker]
    require(len(matrices) == 1, item["key"],
            f"{step['id']}: exactly one clause/disposition matrix reference required",
            "WDQ-COVERAGE")
    path = matrices[0]["path"]
    value = tree.json(path)
    closed(value, "schema_version marker frontier_key step_id dispositions", path)
    version(value["schema_version"], path)
    require(value["marker"] == marker and value["frontier_key"] == item["key"] and
            value["step_id"] == step["id"], path, "clause matrix identity mismatch",
            "WDQ-COVERAGE")
    array(value["dispositions"], path)
    selected = {c["id"]: c for c in clauses if c["step_id"] == step["id"]}
    ids = []
    steps = {s["id"]: s for s in item["completion"]["steps"]}
    for row in value["dispositions"]:
        closed(row, "clause_id disposition evidence_refs owner_step_id", path)
        identifier(row["clause_id"], path)
        ids.append(row["clause_id"])
        require(row["clause_id"] in selected, path, "unknown clause disposition", "WDQ-COVERAGE")
        clause = selected[row["clause_id"]]
        enum(row["disposition"], ("documented", "pending_owner", "ratified"), path)
        refs(row["evidence_refs"], path)
        for reference in clause["authority_refs"] + row["evidence_refs"]:
            check_ref(reference)
        if row["disposition"] == "pending_owner":
            target = steps.get(row["owner_step_id"]) if isinstance(row["owner_step_id"], str) else None
            require(target is not None and target["kind"] == "owner_decision" and
                    target["status"] == "pending" and item["state"] != "landed", path,
                    "open clause requires a pending explicit owner-decision step", "WDQ-COVERAGE")
            # PM025 already validates acyclic dependencies. Walk them to ensure
            # this owner decision actually follows the output, not another lane.
            ancestors, pending = set(), list(target["depends_on"])
            while pending:
                current = pending.pop()
                if current not in ancestors:
                    ancestors.add(current)
                    pending.extend(steps[current]["depends_on"])
            require(step["id"] in ancestors, path,
                    "owner decision must depend on the authored step", "WDQ-COVERAGE")
        else:
            require(row["owner_step_id"] is None, path,
                    "resolved clauses cannot retain an open owner step", "WDQ-COVERAGE")
            require(not clause["requires_owner_decision"] or row["disposition"] == "ratified",
                    path, "mechanism clause requires explicit owner ratification", "WDQ-COVERAGE")
        if row["disposition"] == "ratified":
            decisions = [r for r in row["evidence_refs"]
                         if r["path"].startswith("docs/decisions/PRODUCT_MECHANICS_")]
            require(bool(decisions), path, "ratified clause requires numbered decision evidence",
                    "WDQ-COVERAGE")
            for reference in decisions:
                entry = authority.json("specs/spec_governance_manifest.json")["entries"].get(
                    reference["path"], {})
                require(entry.get("class") == "doctrine" and entry.get("controlling") is True,
                        reference["path"], "ratification requires owner-promoted controlling doctrine",
                        "WDQ-AUTHORITY")
                require(resolve_ref(tree, reference) == resolve_ref(authority, reference),
                        reference["path"], "ratification must exist in owner-promoted authority",
                        "WDQ-AUTHORITY")
    unique(ids, path, "WDQ-COVERAGE")
    require(set(ids) == set(selected), path,
            "matrix must dispose every clause for the completed step", "WDQ-COVERAGE")
