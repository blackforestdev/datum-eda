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
    "INFRA-S04-07.owner-pending": (None, None, None),
    "INFRA-S04-12.infrastructure-review": (None, None, None),
}


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
    next(row for row in policy["coverage"]["rows"] if row["frontier_key"] == "TASK")["category"] = category
    f.save(f.contract["review_path"], review)
    f.save("specs/active_frontier.json", manifest)
    f.write("specs/PROGRESS.md", render_block(manifest).encode())
