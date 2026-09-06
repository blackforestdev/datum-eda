"""PM041 artifact integrity, input/build freshness and scenario assertion checks.

Opaque native artifacts still require consumer validation and independent review;
passing this module alone is not a native verification verdict.
"""

from workflow_delivery_contract import contract_sha256
from workflow_delivery_evidence_shapes import build_receipt_shape, proof_shape, proof_sha256
from workflow_delivery_io import canonical_json, parse_json, sha256
from workflow_delivery_shapes import array, blob, require, unique


def read_blob(tree, value, *, code="WDQ-ARTIFACT"):
    raw = tree.read(value["path"], committed=True)
    require(bool(raw), value["path"], "empty evidence artifact", code)
    require(sha256(raw) == value["sha256"], value["path"], "artifact hash mismatch", code)
    return raw


def validate_proof(tree, contract, path=None):
    path = path or contract["proof_path"]
    proof = proof_shape(tree.json(path, committed=True), path)
    require(proof["contract_sha256"] == contract_sha256(contract), path,
            "contract changed since proof", "WDQ-STALE")
    tree.has_commit(proof["source_commit"])
    manifest_blob = proof["input_manifest"]
    manifest = parse_json(read_blob(tree, manifest_blob), manifest_blob["path"])
    array(manifest, manifest_blob["path"])
    for member in manifest:
        blob(member, manifest_blob["path"])
    unique([m["path"] for m in manifest], manifest_blob["path"])
    actual = tree.manifest(contract["input_roots"])
    expected = sorted(manifest, key=lambda x: x["path"])
    if actual != expected:
        old = {m["path"]: m["sha256"] for m in expected}
        new = {m["path"]: m["sha256"] for m in actual}
        changed = sorted(p for p in old.keys() | new.keys() if old.get(p) != new.get(p))
        require(False, manifest_blob["path"], f"changed relevant inputs: {changed}",
                "WDQ-INDEX" if tree.staged else "WDQ-STALE")
    build = proof["build"]
    receipt = parse_json(read_blob(tree, build["receipt"]), build["receipt"]["path"])
    build_receipt_shape(receipt, build["receipt"]["path"])
    require(receipt["input_manifest_sha256"] == manifest_blob["sha256"]
            and receipt["toolchain"] == build["toolchain"]
            and receipt["build_command"] == build["command"], build["receipt"]["path"],
            "build receipt disagrees with proof inputs/command/toolchain", "WDQ-STALE")
    if build["source_clean"]:
        from workflow_delivery_tree import Tree
        historical = Tree(tree.root, revision=proof["source_commit"])
        require(historical.manifest(contract["input_roots"]) == actual, path,
                "source_clean contradicts recorded commit inputs", "WDQ-STALE")
    read_blob(tree, proof["fixture"], code="WDQ-STALE")
    read_blob(tree, proof["environment"], code="WDQ-ENVIRONMENT")
    scenarios = {s["id"]: s for s in contract["scenarios"]}
    require({r["scenario_id"] for r in proof["results"]} == set(scenarios), path,
            "exact scenario result coverage required", "WDQ-RESULT")
    issues = _issues(tree)
    for result in proof["results"]:
        sid = result["scenario_id"]
        label = path + ".results." + sid
        require(result["outcome"] == "pass", label, "required scenario not passing", "WDQ-RESULT")
        assertions = result["assertions"]
        require(all(a["outcome"] == "pass" for a in assertions), label,
                "failed/unverified assertion", "WDQ-RESULT")
        required = {k for k, v in scenarios[sid]["dimensions"].items()
                    if v["disposition"] == "required"}
        require(required <= {a["dimension"] for a in assertions}, label,
                "missing required dimension assertion", "WDQ-RESULT")
        for artifact in result["artifacts"]:
            read_blob(tree, artifact)
        require(set(result["defects"]) <= set(issues), label,
                "defect must exist in beads", "WDQ-DEFECT")
    return proof


def _issues(tree):
    result = {}
    for line in tree.read(".beads/issues.jsonl").splitlines():
        if line.strip():
            issue = parse_json(line, ".beads/issues.jsonl")
            require(type(issue) is dict and type(issue.get("id")) is str,
                    ".beads/issues.jsonl", "issue identity missing", "WDQ-DEFECT")
            require(issue["id"] not in result, ".beads/issues.jsonl",
                    "duplicate issue identity", "WDQ-DEFECT")
            result[issue["id"]] = issue
    return result


def packet_sha256(contract, proof, authority_digest):
    return sha256(canonical_json({
        "contract_sha256": contract_sha256(contract),
        "input_manifest_sha256": proof["input_manifest"]["sha256"],
        "proof_sha256": proof_sha256(proof),
        "authority_sha256": authority_digest,
    }))
