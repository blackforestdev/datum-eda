"""Local Git authority fixtures; never owner promotion in the Datum repository."""

from copy import deepcopy
from pathlib import Path

from workflow_delivery_authority import authority_sha256
from workflow_delivery_evidence_shapes import review_sha256
from workflow_delivery_io import canonical_json
from workflow_delivery_native_test_support import typed_proof
from workflow_delivery_proof import packet_sha256
from workflow_delivery_tree import Tree
from workflow_delivery_trust import _gate_paths


def accepted_fixture(f):
    f.contract["category"] = "product"
    f.contract["scenarios"][0]["method"] = "native_input"
    f.save("contract.json", f.contract)
    proof, roles, events, registry = typed_proof(f)
    replay, replay_roles, replay_events, _ = typed_proof(f, "reviewer")
    replay_blob = f.blob("docs/reviews/replay.json", canonical_json(replay))
    f.save(f.contract["proof_path"], proof)
    item = {"key": "TASK", "issue_id": "dat-test", "authorization": "owner_decision",
            "state": "in_progress", "completion": {
                "canonical_next_step_id": "A", "delivery": {
                    "contract_path": "contract.json", "checkpoints": {
                        "ready": "R", "activate": "I", "verify": "V", "accept": "A"}},
                "steps": [
                    {"id": "R", "kind": "planning", "status": "complete", "depends_on": []},
                    {"id": "I", "kind": "execution", "status": "complete", "depends_on": ["R"]},
                    {"id": "V", "kind": "execution", "status": "complete", "depends_on": ["I"]},
                    {"id": "A", "kind": "owner_decision", "status": "pending", "depends_on": ["V"]}]}}
    item = _frontier(f, item)
    f.save("specs/workflow_delivery_policy.json", {
        "schema_version": 1, "decision_ref": f.ref, "legacy_baseline": f.head,
        "enrolled": [{"frontier_key": "TASK", "implementation_sessions": ["writer"],
                      "activation_ref": f.ref}]})
    tree = Tree(f.root)
    authority = authority_sha256(tree, f.contract)
    packet = packet_sha256(f.contract, proof, authority)
    review = {"schema_version": 1, "packet_sha256": packet, "reviewer_session": "reviewer",
              "independent_of": ["writer", "original"], "disposition": "approve",
              "replay": replay_blob, "findings": [], "owner_receipt": {
                  "path": "docs/reviews/owner.md", "marker": "<!-- RECEIPT -->"}}
    f.save(f.contract["review_path"], review)
    f.write("docs/reviews/owner.md", (
        "<!-- RECEIPT -->\n## Acceptance\n"
        f"ACCEPT TASK/A {packet} {review_sha256(review)}\n"
        "Source: hermetic fixture, not a real owner response\nDate: 2026-09-06\n").encode())
    governance = tree.json("specs/spec_governance_manifest.json")
    governance["entries"]["docs/reviews/owner.md"] = {"class": "governed"}
    f.save("specs/spec_governance_manifest.json", governance)
    source = Tree(Path(__file__).resolve().parents[1])
    # Include untracked implementation modules in this isolated authority fixture.
    paths = _gate_paths(source) | {
        p.relative_to(source.root).as_posix() for p in (source.root / "scripts").glob("workflow_delivery_*.py")
        if not p.name.endswith("_test_support.py")}
    for path in paths:
        f.write(path, (source.root / path).read_bytes())
    # Synthetic input only: real-roadmap captures must retain the actual
    # exemption manifest and its referenced source files.
    f.save("specs/rustfmt_exemption_manifest.json", {"schema_version": 1, "exemptions": {}})
    f.stage()
    f.git("commit", "-qm", "synthetic trusted authority")
    authority_ref = f.git("rev-parse", "HEAD").decode().strip()
    return item, proof, review, authority_ref


def _frontier(f, item):
    """A valid PM025 history plus one unrelated canonical task, not a rival roadmap."""
    from test_project_status import ProjectStatusTest
    from project_status import render_block
    doc = "docs/decisions/PRODUCT_MECHANICS_025_FIXTURE.md"
    factory = ProjectStatusTest()
    factory.doc = doc
    template = factory.item()
    completion = deepcopy(template["completion"])
    completion.update(item["completion"])
    completion["canonical_next_step_id"] = None
    item.update({"order": 0, "title": "Accepted fixture", "state": "landed", "authorization": "none",
                 "canonical_next": False, "parallel": False, "summary": "Synthetic accepted history",
                 "governing_docs": [doc], "dependencies": [], "unblocks": [],
                 "landing_commit": f.head, "completion": completion})
    completion["post_completion"]["effects"] = ["No task selected by delivery"]
    doc_lines = ["# Synthetic project-state doctrine\n"]
    for step in completion["steps"]:
        sid = step["id"]
        step["status"] = "complete"
        step["action"] = "Fixture " + sid
        step["requirement_refs"] = [{"path": doc, "marker": sid}]
        step["completion_evidence"] = [{"kind": "review" if sid == "A" else "document",
                                        "path": doc, "marker": "EVIDENCE-TASK-" + sid}]
        doc_lines += [f"<!-- REQ:TASK:{sid} -->\n", f"<!-- EVIDENCE-TASK-{sid} -->\n"]
        if sid == "A":
            step["owner_input"] = {"response_format": "Fixture ACCEPT", "requests": [{
                "id": "ACCEPT", "question": "Accept fixture?", "recommended_response": "Fixture only",
                "source_ref": {"path": doc, "marker": "A"}}]}
            doc_lines += ["<!-- OWNER:TASK:A:A -->\n"]
    next_item = deepcopy(template)
    next_item.update(key="NEXT", order=1, issue_id="dat-next")
    next_item["completion"]["canonical_next_step_id"] = "NEXT-C01"
    next_item["completion"]["steps"][0]["id"] = "NEXT-C01"
    next_item["completion"]["steps"][0]["requirement_refs"][0]["marker"] = "NEXT-C01"
    doc_lines += ["<!-- REQ:NEXT:NEXT-C01 -->\n"]
    f.write(doc, "".join(doc_lines).encode())
    governance = Tree(f.root).json("specs/spec_governance_manifest.json")
    governance["entries"][doc] = {"class": "doctrine", "controlling": True}
    f.save("specs/spec_governance_manifest.json", governance)
    manifest = {"schema_version": 6, "policy_decision": doc, "claim_ttl_hours": 8,
                "frontier": [item, next_item]}
    f.save("specs/active_frontier.json", manifest)
    f.write("specs/PROGRESS.md", render_block(manifest).encode())
    issue = {"id": "dat-test", "status": "closed", "labels": ["roadmap:frontier"],
             "acceptance_criteria": "R: Ready\nI: Activate\nV: Verify\nA: Accept"}
    next_issue = {"id": "dat-next", "status": "open", "labels": ["roadmap:frontier"],
                  "acceptance_criteria": "NEXT-C01: Next unrelated task"}
    f.write(".beads/issues.jsonl", canonical_json(issue) + canonical_json(next_issue))
    return item
