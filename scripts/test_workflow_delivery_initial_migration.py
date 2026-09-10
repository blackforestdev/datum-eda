"""Full inspector migration tests using synthetic evidence, never owner proof."""

from copy import deepcopy
import json
import unittest

import test_workflow_delivery_activation_preflight as support
from project_status import render_block
from workflow_delivery_capture_state import protected_state
from workflow_delivery_io import canonical_json
from workflow_delivery_native_test_support import typed_proof
from workflow_delivery_tree import Tree


class InitialMigrationTest(unittest.TestCase):
    def setUp(self):
        self.case = support.PromotionCandidateTest()
        self.addCleanup(self.case.doCleanups)
        self.case.setUp()
        f = self.case.f
        manifest = Tree(f.root).json("specs/active_frontier.json")
        rollout = manifest["frontier"][0]
        rollout["completion"]["delivery"]["contract_path"] = "specs/workflow_delivery/rollout.contract.json"
        f.save(rollout["completion"]["delivery"]["contract_path"], f.contract)
        pilot = deepcopy(rollout)
        pilot.update(key="PILOT", issue_id="dat-pilot", order=len(manifest["frontier"]), canonical_next=False)
        doc = pilot["governing_docs"][0]
        pilot_doc = "docs/pilot-authority.md"
        # Copy the fixture's authority references into a distinct governed file.
        pilot = json.loads(json.dumps(pilot).replace(doc, pilot_doc))
        f.write(pilot_doc, (f.root / doc).read_bytes().replace(support.ROLLOUT.encode(), b"PILOT"))
        governance = Tree(f.root).json("specs/spec_governance_manifest.json")
        governance["entries"][pilot_doc] = deepcopy(governance["entries"][doc])
        f.save("specs/spec_governance_manifest.json", governance)
        pilot["completion"]["delivery"] = {"contract_path": "pilot.contract.json",
            "checkpoints": {"ready": "WDQ-READY", "activate": None,
                            "verify": "WDQ-COMPAT", "accept": None}}
        manifest["frontier"].append(pilot)
        issues = [json.loads(line) for line in (f.root / ".beads/issues.jsonl").read_bytes().splitlines()]
        issues.append(dict(issues[0], id="dat-pilot"))
        f.write(".beads/issues.jsonl", b"".join(canonical_json(row) for row in issues))
        contract = f.contract
        f.contract = dict(contract, frontier_key="PILOT", issue_id="dat-pilot",
                          proof_path="docs/reviews/pilot-proof.json", review_path="docs/reviews/pilot-review.json")
        f.save("pilot.contract.json", f.contract)
        typed_proof(f)
        f.contract = contract
        policy = Tree(f.root).json("specs/workflow_delivery_policy.json")
        self.pilot_enrollment = dict(policy["enrolled"][0], frontier_key="PILOT")
        policy["enrolled"].insert(0, self.pilot_enrollment)
        policy["coverage"]["rows"].append(dict(policy["coverage"]["rows"][0],
                                             frontier_key="PILOT", issue_id="dat-pilot"))
        legacy = {k: deepcopy(v) for k, v in policy.items() if k != "coverage"}
        legacy.update(schema_version=1, enrolled=[deepcopy(self.pilot_enrollment)])
        unmapped = deepcopy(manifest)
        del unmapped["frontier"][0]["completion"]["delivery"]
        self.save_manifest(unmapped)
        f.save("specs/workflow_delivery_policy.json", legacy)
        self.case.commit_candidate()
        self.case.base = self.case.candidate
        self.save_manifest(manifest)
        f.save("specs/workflow_delivery_policy.json", policy)
        self.case.make_proof()
        environments = Tree(f.root).json("requested-environment.json")
        environments["environments"].append(dict(environments["environments"][0], frontier_key="PILOT"))
        f.save("requested-environment.json", environments)
        self.case.commit_candidate()

    def save_manifest(self, manifest):
        self.case.f.save("specs/active_frontier.json", manifest)
        self.case.f.write("specs/PROGRESS.md", render_block(manifest).encode())

    def test_full_inspector_accepts_initial_mapping_and_preserves_state(self):
        before = self.case.f.snapshot()
        protected_before = protected_state(self.case.f.root, ["src"])
        result = self.case.inspect()
        self.assertEqual(["PILOT: verify", support.ROLLOUT + ": review"], result["checks"])
        self.assertFalse(result["activation_asserted"])
        self.assertFalse(result["publication_authorized"])
        self.assertEqual(before, self.case.f.snapshot())
        self.assertEqual(protected_before, protected_state(self.case.f.root, ["src"]))

    def test_full_inspector_refuses_changed_prior_enrollment(self):
        f = self.case.f
        policy = Tree(f.root).json("specs/workflow_delivery_policy.json")
        policy["enrolled"][0]["implementation_sessions"] = ["replacement"]
        f.save("specs/workflow_delivery_policy.json", policy)
        self.case.commit_candidate()
        with self.assertRaisesRegex(ValueError, "preserve prior enrollments"):
            self.case.inspect()


if __name__ == "__main__":
    unittest.main()
