"""Synthetic typed evidence for refusal tests, not a production or native run."""

from copy import deepcopy

from workflow_delivery_authority import authority_sha256
from workflow_delivery_io import canonical_json, sha256
from workflow_delivery_native import ARTIFACT_KIND
from workflow_delivery_tree import Tree


def environment():
    return {"os": "fixture-linux", "backend": "fixture", "toolchain": "fixture-python",
            "scale": 1, "input_method": "fixture", "window_size": [800, 600],
            "reproduction_commands": ["Run the hermetic validator fixture"]}


def typed_proof(f, producer="original", *, disabled=False):
    proof = f.proof()
    proof["producer_session"] = producer
    proof["environment"] = f.blob("evidence/environment.json", canonical_json(environment()))
    consumer = f.contract["consumers"][0]
    handler = consumer["handler_ref"]
    binary = sha256(b"fixture interpreter")
    reason = "unavailable" if disabled else "implemented"
    registry = {"schema_version": 1, "binary_sha256": binary, "entries": [{
        "dispatch_key": consumer["dispatch_key"], "handler_ref": handler,
        "entry_surfaces": consumer["entry_surfaces"],
        "contexts": {"fixture": {"enabled": not disabled, "reason": reason}}}]}
    event = {"schema_version": 1, "scenario_id": "S01", "producer_session": producer,
             "method": f.contract["scenarios"][0]["method"], "binary_sha256": binary,
             "authority_sha256": authority_sha256(Tree(f.root), f.contract),
             "inputs": f.contract["scenarios"][0]["inputs"], "actual_visible": "value",
             "actual_state": "unchanged", "dispatches": [{
                 "dispatch_key": consumer["dispatch_key"], "entry_surface": "api",
                 "context": "fixture", "eligible": not disabled, "enabled": not disabled,
                 "invoked": not disabled, "handler_ref": handler,
                 "unavailable_reason": reason, "mutation_count": 0}]}
    roles = {"kind": ARTIFACT_KIND, "scenario_id": "S01",
             "events": [f.blob(f"evidence/{producer}-events.json", canonical_json(event))],
             "captures": [f.blob("evidence/capture.txt", b"synthetic capture, not native evidence\n")],
             "state": [f.blob("evidence/state.txt", b"unchanged fixture state\n")],
             "registry": f.blob("evidence/registry.json", canonical_json(registry))}
    index = f.blob(f"evidence/{producer}-roles.json", canonical_json(roles))
    proof["results"][0]["artifacts"] = roles["events"] + roles["captures"] + roles["state"] + [roles["registry"], index]
    f.save(f.contract["proof_path"], proof)
    f.stage()
    return proof, roles, event, registry


def replace_role(f, proof, roles, role, data):
    updated = deepcopy(roles)
    old = roles[role][0] if type(roles[role]) is list else roles[role]
    new = f.blob(old["path"], canonical_json(data))
    if type(roles[role]) is list:
        updated[role][0] = new
    else:
        updated[role] = new
    index_path = f"evidence/{proof['producer_session']}-roles.json"
    index = f.blob(index_path, canonical_json(updated))
    proof["results"][0]["artifacts"] = (updated["events"] + updated["captures"]
        + updated["state"] + [updated["registry"], index])
    f.save(f.contract["proof_path"], proof)
    f.stage()
    return updated
