"""Two-enrollment synthetic INPUT; no actual native run or owner acceptance."""

from copy import deepcopy

from project_status import render_block
from workflow_delivery_io import canonical_json, sha256
from workflow_delivery_native_test_support import typed_proof
from workflow_delivery_test_support import Fixture
from workflow_delivery_tree import Tree


def headless_environment(variant=None):
    value = {"schema_version": 2, "kind": "headless", "os": "fixture-linux",
            "toolchain": "fixture-python", "transport": "pipes", "terminal_size": None,
            "input_method": "synthetic fixture pipes", "reproduction_commands": ["synthetic fixture run"],
            "tools": [{"name": name, "path": "/nonexistent/fixture-" + name,
                       "sha256": sha256(b"fixture interpreter"), "version": "synthetic fixture version"}
                      for name in ("interpreter", "git")]}
    if variant == "INFRA-S06-12.environment-version":
        value["schema_version"] = True
    elif variant == "INFRA-S06-12.environment-kind":
        value["kind"] = "synthetic-invalid-kind"
    elif variant == "INFRA-S06-12.environment-fields":
        value["unexpected"] = "synthetic extra environment field"
    elif variant == "INFRA-S06-13.toolchain":
        value["toolchain"] = "synthetic mismatched toolchain"
    elif variant == "INFRA-S06-13.interpreter":
        value["tools"][0]["sha256"] = sha256(b"different synthetic interpreter")
    elif variant == "INFRA-S06-14.pipes-dimensions":
        value["terminal_size"] = [80, 24]
    elif variant == "INFRA-S06-14.pty-dimensions":
        value["transport"] = "pty"
    return value


class InfrastructureView:
    proof = Fixture.proof

    def __init__(self, fixture):
        self.fixture, self.root, self.head = fixture, fixture.root, fixture.head
        self.contract = deepcopy(fixture.contract)
        self.contract.update(id="infra-contract", frontier_key="INFRA", issue_id="dat-infra",
                             category="infrastructure", proof_path="docs/reviews/infra/proof.json",
                             review_path="docs/reviews/infra/review.json")
        self.contract["scenarios"][0]["method"] = "infrastructure"

    def blob(self, path, raw):
        return self.fixture.blob("infra-input/" + path, raw)

    def save(self, path, value):
        self.fixture.save(path, value)

    def stage(self):
        self.fixture.stage()


def add_infrastructure(f, variant=None):
    view = InfrastructureView(f)
    f.save("infra-contract.json", view.contract)
    proof, _, _, _ = typed_proof(view, "infra-producer")
    proof["environment"] = view.blob("headless.json", canonical_json(headless_environment(variant)))
    f.save(view.contract["proof_path"], proof)
    manifest = Tree(f.root).json("specs/active_frontier.json")
    item = deepcopy(manifest["frontier"][0])
    item.update(key="INFRA", issue_id="dat-infra", order=2, title="Synthetic headless infrastructure history")
    completion = item["completion"]
    completion["delivery"] = {"contract_path": "infra-contract.json", "checkpoints": {
        "ready": "R", "activate": None, "verify": "V", "accept": None}}
    completion["steps"] = [step for step in completion["steps"] if step["id"] in ("R", "V")]
    doc = item["governing_docs"][0]
    additions = []
    for step in completion["steps"]:
        sid = step["id"]
        step["depends_on"] = [] if sid == "R" else ["R"]
        step["requirement_refs"] = [{"path": doc, "marker": sid}]
        step["completion_evidence"] = [{"kind": "document", "path": doc, "marker": "EVIDENCE-INFRA-" + sid}]
        additions += [f"<!-- REQ:INFRA:{sid} -->", f"<!-- INFRA-{sid} -->", f"<!-- EVIDENCE-INFRA-{sid} -->"]
    f.write(doc, (f.root / doc).read_bytes() + ("\n" + "\n".join(additions) + "\n").encode())
    manifest["frontier"].append(item)
    f.save("specs/active_frontier.json", manifest)
    f.write("specs/PROGRESS.md", render_block(manifest).encode())
    issues = (f.root / ".beads/issues.jsonl").read_bytes()
    f.write(".beads/issues.jsonl", issues + canonical_json({"id": "dat-infra", "status": "closed",
        "labels": ["roadmap:frontier"], "acceptance_criteria": "R: Ready\nV: Verify"}))
    policy = Tree(f.root).json("specs/workflow_delivery_policy.json")
    policy["enrolled"].append({"frontier_key": "INFRA", "implementation_sessions": ["infra-writer"],
                              "activation_ref": f.ref})
    f.save("specs/workflow_delivery_policy.json", policy)
    f.stage()
    f.git("commit", "-qm", "test(workflow): retain synthetic mixed enrollment\n\nNo native proof or owner promotion.")
    return f.git("rev-parse", "HEAD").decode().strip()
