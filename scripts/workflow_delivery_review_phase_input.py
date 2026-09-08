"""Synthetic review-only INPUT; never independent review of rollout code."""

from copy import deepcopy
import json

from project_status import render_block
from workflow_delivery_authority import authority_sha256
from workflow_delivery_io import canonical_json
from workflow_delivery_native_test_support import typed_proof
from workflow_delivery_proof import packet_sha256
from workflow_delivery_tree import Tree

REVIEW_PHASE_CASES = {
    "INFRA-S04-06.producer-unavailable": ("reader", "WDQ-CONSUMER", "required enabled behavior lacks invocation on every reviewed surface"),
    "INFRA-S04-06.reviewer-unavailable": ("reader", "WDQ-CONSUMER", "required enabled behavior lacks invocation on every reviewed surface"),
    "INFRA-S04-01.skipped-review": ("TASK", "WDQ-TRANSITION", "checkpoints must follow dependency order"),
    "INFRA-S04-01.missing-record": ("docs/reviews/review.json", "WDQ-ARTIFACT", "required artifact is not in candidate Git tree"),
    "INFRA-S04-07.owner-pending": (None, None, None),
    "INFRA-S04-12.infrastructure-review": (None, None, None),
}


def review_phase_expectation(variant, surface):
    path, code, detail = REVIEW_PHASE_CASES[variant]
    if variant.endswith(".missing-record") and surface in ("C", "H"):
        code = "WDQ-INDEX"
    return path, code, detail


def prepare_review_phase(f, manifest, policy, variant):
    if variant not in REVIEW_PHASE_CASES:
        raise ValueError("explicit implemented review-only fixture required")
    item = manifest["frontier"][0]
    completion = item["completion"]
    doc = item["governing_docs"][0]
    item.update(state="specified", authorization="owner_decision")
    del item["landing_commit"]
    completion["delivery"]["schema_version"] = 2
    completion["delivery"]["checkpoints"]["review"] = "D"
    completion["canonical_next_step_id"] = "A"
    review_step = deepcopy(completion["steps"][2])
    review_step.update(id="D", depends_on=["V"], requirement_refs=[{"path": doc, "marker": "D"}],
                       completion_evidence=[{"kind": "review", "path": doc, "marker": "REVIEW-TASK-D"}])
    completion["steps"].insert(3, review_step)
    completion["steps"][-1].update(status="pending", depends_on=["D"], completion_evidence=[])
    f.write(doc, (f.root / doc).read_bytes() + b"\n<!-- REQ:TASK:D -->\n<!-- REVIEW-TASK-D -->\n")
    issues = [json.loads(line) for line in (f.root / ".beads/issues.jsonl").read_bytes().splitlines()]
    next(row for row in issues if row["id"] == "dat-test").update(
        status="open", acceptance_criteria="R: Ready\nI: Activate\nV: Verify\nD: Review\nA: Accept")
    f.write(".beads/issues.jsonl", b"".join(canonical_json(row) for row in issues))
    tree = Tree(f.root)
    review = tree.json(f.contract["review_path"])
    review["owner_receipt"] = None
    category = "product"
    if variant.endswith(".infrastructure-review"):
        category = "infrastructure"
        f.contract["category"] = category
        f.contract["scenarios"][0]["method"] = "infrastructure"
        f.save("contract.json", f.contract)
        completion["delivery"]["checkpoints"].update(activate=None, accept=None)
        proof, _, _, _ = typed_proof(f)
        replay, _, _, _ = typed_proof(f, "reviewer")
        review["replay"] = f.blob("docs/reviews/replay.json", canonical_json(replay))
        f.save(f.contract["proof_path"], proof)
        review["packet_sha256"] = packet_sha256(f.contract, proof, authority_sha256(tree, f.contract))
    else:
        review["findings"] = [{"issue_id": "dat-next", "severity": "blocking", "disposition_ref": None}]
    if variant.endswith(".skipped-review"):
        review_step.update(status="pending", completion_evidence=[])
        completion["steps"][-1]["depends_on"] = ["V"]
    if variant.startswith("INFRA-S04-06."):
        unavailable_run(f, tree, review, replay=variant.endswith(".reviewer-unavailable"))
    next(row for row in policy["coverage"]["rows"] if row["frontier_key"] == "TASK")["category"] = category
    f.save(f.contract["review_path"], review)
    if variant.endswith(".missing-record"):
        # Omit only this just-created synthetic input. Its older fixture history
        # remains retained, but cannot substitute for the selected tree's record.
        (f.root / f.contract["review_path"]).unlink()
        f.git("add", "--", f.contract["review_path"])
    f.save("specs/active_frontier.json", manifest)
    f.write("specs/PROGRESS.md", render_block(manifest).encode())


def unavailable_run(f, tree, review, *, replay):
    """Disable only the named synthetic run; preserve the other run's registry."""
    path = review["replay"]["path"] if replay else f.contract["proof_path"]
    proof = tree.json(path)
    producer = proof["producer_session"]
    index_path = f"evidence/{producer}-roles.json"
    roles = tree.json(index_path)
    registry = tree.json(roles["registry"]["path"])
    registry["entries"][0]["contexts"]["fixture"].update(enabled=False, reason="unavailable")
    roles["registry"] = f.blob(f"evidence/{producer}-unavailable-registry.json", canonical_json(registry))
    event = tree.json(roles["events"][0]["path"])
    event["dispatches"][0].update(eligible=False, enabled=False, invoked=False, unavailable_reason="unavailable")
    roles["events"][0] = f.blob(roles["events"][0]["path"], canonical_json(event))
    index = f.blob(index_path, canonical_json(roles))
    proof["results"][0]["artifacts"] = (roles["events"] + roles["captures"] + roles["state"]
                                         + [roles["registry"], index])
    if replay:
        review["replay"] = f.blob(path, canonical_json(proof))
    else:
        f.save(path, proof)
        review["packet_sha256"] = packet_sha256(f.contract, proof, authority_sha256(tree, f.contract))
