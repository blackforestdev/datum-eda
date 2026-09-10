"""Hermetic clause inventories and matrices; never native or owner evidence."""

import hashlib

from workflow_delivery_tree import Tree


def govern(f, path):
    manifest = Tree(f.root).json("specs/spec_governance_manifest.json")
    manifest["entries"][path] = {"class": "governed"}
    f.save("specs/spec_governance_manifest.json", manifest)


def inventory(f, item, policy):
    path = "docs/specification-inventory.json"
    marker = "WDQ-CLAUSE-INVENTORY." + item["key"]
    value = {"schema_version": 1, "marker": marker, "frontier_key": item["key"],
        "clauses": [{"id": s["id"] + "." + str(n), "step_id": s["id"],
                    "requirement_ref": r, "authority_refs": [f.ref],
                    "requires_owner_decision": False}
                   for s in item["completion"]["steps"]
                   for n, r in enumerate(s["requirement_refs"])]}
    f.save(path, value)
    govern(f, path)
    next(r for r in policy["coverage"]["rows"] if r["frontier_key"] == item["key"])[
        "boundary_ref"] = {"path": path, "marker": marker}
    return value


def matrix(f, item, step, clauses):
    path = "docs/specification-matrix.json"
    marker = f"WDQ-CLAUSE-MATRIX.{item['key']}.{step['id']}"
    value = {"schema_version": 1, "marker": marker, "frontier_key": item["key"],
             "step_id": step["id"], "dispositions": [
                 {"clause_id": c["id"], "disposition": "documented",
                  "evidence_refs": [f.ref], "owner_step_id": None}
                 for c in clauses if c["step_id"] == step["id"]]}
    f.save(path, value)
    govern(f, path)
    refresh(f)
    return value, {"kind": "document", "path": path, "marker": marker}


def refresh(f):
    manifest = Tree(f.root).json("specs/evidence_traceability_manifest.json")
    manifest["routes"] = [r for r in manifest["routes"] if r["id"] != "specification-output"]
    members = ["docs/specification-inventory.json", "docs/specification-matrix.json"]
    digest = hashlib.sha256()
    for path in sorted(members):
        digest.update(path.encode() + b"\0" + f.root.joinpath(path).read_bytes() + b"\0")
    manifest["routes"].append({"id": "specification-output", "sources": members[:1],
                              "consumers": members[1:], "reviewed_digest": digest.hexdigest()})
    f.save("specs/evidence_traceability_manifest.json", manifest)
