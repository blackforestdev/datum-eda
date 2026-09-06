#!/usr/bin/env python3
"""Hermetic N13-N18/P06 trust and receipt checks; no real approval is recorded."""

from copy import deepcopy
import unittest

from workflow_delivery_authority import authority_sha256
from workflow_delivery_checkpoints import validate_delivery, validate_transition
from workflow_delivery_io import DeliveryInputError, canonical_json
from workflow_delivery_native_test_support import environment
from workflow_delivery_review import receipt_section, validate_review
from workflow_delivery_test_support import Fixture
from workflow_delivery_tree import Tree
from workflow_delivery_trust import Trust
from workflow_delivery_trust_test_support import accepted_fixture


class TrustTests(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.close)
        self.item, self.proof, self.review, self.authority = accepted_fixture(self.f)

    def trust(self):
        return Trust(Tree(self.f.root), self.authority, self.authority)

    def check(self):
        tree = Tree(self.f.root)
        return validate_review(tree, self.f.contract, self.proof,
                               authority_sha256(tree, self.f.contract), self.trust(),
                               self.item, environment())

    def refuse(self, code):
        before = self.f.snapshot()
        with self.assertRaises(DeliveryInputError) as caught:
            self.check()
        self.assertEqual(caught.exception.code, code, str(caught.exception))
        self.assertEqual(before, self.f.snapshot())

    def test_p06_exact_trusted_receipt_and_independent_replay(self):
        self.assertEqual(self.check(), self.review)
        self.assertEqual(validate_delivery(Tree(self.f.root), self.item, phase="accept",
                         trust=self.trust(), environment=environment()), "accept")
        self.assertIsNone(self.item["completion"]["canonical_next_step_id"])

    def test_n13_self_review_and_rejected_review(self):
        original = deepcopy(self.review)
        for change in ({"reviewer_session": "writer"}, {"reviewer_session": "original"},
                       {"disposition": "revise"}, {"disposition": "reject"},
                       {"independent_of": ["original"]}):
            self.review = deepcopy(original)
            self.review.update(change)
            self.f.save(self.f.contract["review_path"], self.review)
            self.refuse("WDQ-REVIEW")

    def test_n13_copied_replay_is_not_independent(self):
        self.review["replay"] = self.f.blob("docs/reviews/replay.json", canonical_json(self.proof))
        self.f.save(self.f.contract["review_path"], self.review)
        self.refuse("WDQ-REVIEW")

    def test_n13_distinct_replay_identity_cannot_copy_event_blob(self):
        tree = Tree(self.f.root)
        replay = tree.json(self.review["replay"]["path"])
        roles = tree.json("evidence/reviewer-roles.json")
        roles["events"] = tree.json("evidence/original-roles.json")["events"]
        index = self.f.blob("evidence/reviewer-roles.json", canonical_json(roles))
        replay["results"][0]["artifacts"] = roles["events"] + roles["captures"] + roles["state"] + [roles["registry"], index]
        self.review["replay"] = self.f.blob(self.review["replay"]["path"], canonical_json(replay))
        self.f.save(self.f.contract["review_path"], self.review)
        self.f.stage()
        self.refuse("WDQ-REVIEW")

    def test_n14_candidate_only_or_changed_owner_receipt(self):
        self.f.write("docs/reviews/owner.md", b"<!-- RECEIPT -->\nOwner: somebody\nDate: 2026-09-06\n")
        self.refuse("WDQ-RECEIPT")

    def test_n14_wrong_packet_and_missing_receipt(self):
        self.review["packet_sha256"] = "0" * 64
        self.f.save(self.f.contract["review_path"], self.review)
        self.refuse("WDQ-STALE")
        from workflow_delivery_io import parse_json
        self.review = parse_json(self.f.git("show", self.authority + ":" + self.f.contract["review_path"]), "review")
        self.review["owner_receipt"] = None
        self.f.save(self.f.contract["review_path"], self.review)
        self.refuse("WDQ-RECEIPT")

    def test_n15_undisposed_defect(self):
        self.review["findings"] = [{"issue_id": "dat-test", "severity": "blocking", "disposition_ref": None}]
        self.f.save(self.f.contract["review_path"], self.review)
        self.refuse("WDQ-DEFECT")

    def test_n16_skip_verification_or_claim_owner_decision(self):
        candidate = deepcopy(self.item)
        candidate["completion"]["steps"][-1]["status"] = "complete"
        base = deepcopy(self.item)
        base["completion"]["steps"][-2]["status"] = "pending"
        base["completion"]["steps"][-1]["status"] = "pending"
        with self.assertRaises(DeliveryInputError) as caught:
            validate_transition(candidate, base)
        self.assertEqual(caught.exception.code, "WDQ-TRANSITION")
        candidate = deepcopy(self.item)
        candidate["completion"]["canonical_next_step_id"] = "A"
        candidate["completion"]["steps"][-1]["status"] = "pending"
        candidate["claim"] = {"agent": "writer"}
        with self.assertRaises(DeliveryInputError):
            validate_transition(candidate, self.item)

    def test_n17_policy_removal_and_gate_weakening(self):
        policy_path = "specs/workflow_delivery_policy.json"
        original = Tree(self.f.root).json(policy_path)
        altered = deepcopy(original)
        altered["enrolled"] = []
        self.f.save(policy_path, altered)
        with self.assertRaises(DeliveryInputError):
            self.trust()
        self.f.save(policy_path, original)
        self.f.write("scripts/check_workflow_delivery.py", b"raise SystemExit(0)\n")
        with self.assertRaises(DeliveryInputError) as caught:
            self.trust()
        self.assertEqual(caught.exception.code, "WDQ-POLICY")

    def test_n17_contract_roots_or_scenarios_cannot_self_promote(self):
        trust = self.trust()
        changed = deepcopy(self.f.contract)
        changed["input_roots"] = ["src/read.py"]
        with self.assertRaises(DeliveryInputError) as caught:
            trust.contract(Tree(self.f.root), self.item, changed)
        self.assertEqual(caught.exception.code, "WDQ-POLICY")

    def test_n18_missing_or_unresolved_trust(self):
        for authority, base in ((None, self.authority), (self.authority, None),
                                ("missing", self.authority), (self.authority, "missing")):
            with self.assertRaises(DeliveryInputError) as caught:
                Trust(Tree(self.f.root), authority, base)
            self.assertEqual(caught.exception.code, "WDQ-TRUST")

    def test_receipt_must_be_inside_selected_section(self):
        raw = b"<!-- RECEIPT -->\n## First\nSource: fixture\n## Other\nACCEPT elsewhere\n"
        self.assertNotIn("ACCEPT", receipt_section(raw, "<!-- RECEIPT -->", "receipt"))

    def test_completed_obligations_cannot_regress_to_skip_freshness(self):
        candidate = deepcopy(self.item)
        candidate["authorization"] = "planning"
        for step in candidate["completion"]["steps"]:
            step["status"] = "pending"
        with self.assertRaises(DeliveryInputError) as caught:
            validate_delivery(Tree(self.f.root), candidate, trust=self.trust())
        self.assertEqual(caught.exception.code, "WDQ-TRANSITION")

    def test_replay_cannot_use_different_fixture(self):
        replay = Tree(self.f.root).json(self.review["replay"]["path"])
        replay["fixture"] = self.f.blob("evidence/other-fixture.txt", b"different fixture")
        self.review["replay"] = self.f.blob(self.review["replay"]["path"], canonical_json(replay))
        self.f.save(self.f.contract["review_path"], self.review)
        self.f.stage()
        self.refuse("WDQ-REVIEW")

    def test_closed_defect_or_unrelated_heading_is_not_disposition(self):
        for severity in ("blocking", "nonblocking"):
            self.review["findings"] = [{"issue_id": "dat-test", "severity": severity,
                                        "disposition_ref": self.f.ref}]
            self.f.save(self.f.contract["review_path"], self.review)
            self.refuse("WDQ-DEFECT")

    def test_selector_retains_configured_trust_after_candidate_deletion(self):
        from workflow_delivery_selector import selector_failures
        self.f.git("config", "--local", "datum.workflowDeliveryAuthorityRef", self.authority)
        self.f.git("config", "--local", "datum.workflowDeliveryBaseRef", self.authority)
        (self.f.root / "specs/workflow_delivery_policy.json").unlink()
        failures = selector_failures(self.f.root, {"schema_version": 6, "frontier": []})
        self.assertTrue(any("WDQ-POLICY" in error for error in failures), failures)

    def test_older_base_cannot_erase_promoted_completed_obligation(self):
        candidate = deepcopy(self.item)
        candidate["authorization"] = "planning"
        for step in candidate["completion"]["steps"]:
            step["status"] = "pending"
        trust = Trust(Tree(self.f.root), self.authority, self.f.head)
        with self.assertRaises(DeliveryInputError) as caught:
            validate_delivery(Tree(self.f.root), candidate, trust=trust)
        self.assertEqual(caught.exception.code, "WDQ-TRANSITION")

    def test_replay_cannot_test_different_build(self):
        tree = Tree(self.f.root)
        replay = tree.json(self.review["replay"]["path"])
        receipt = tree.json(replay["build"]["receipt"]["path"])
        receipt["binary_sha256"] = "0" * 64
        replay["build"]["receipt"] = self.f.blob("evidence/replay-build.json", canonical_json(receipt))
        self.review["replay"] = self.f.blob(self.review["replay"]["path"], canonical_json(replay))
        self.f.save(self.f.contract["review_path"], self.review)
        self.f.stage()
        self.refuse("WDQ-REVIEW")

    def test_pending_labels_cannot_hide_changed_activation_inputs(self):
        manifest = Tree(self.f.root).json("specs/active_frontier.json")
        item = manifest["frontier"][0]
        item["authorization"] = "execution"
        item["completion"]["canonical_next_step_id"] = "I"
        for step in item["completion"]["steps"]:
            step["status"] = "complete" if step["id"] == "R" else "pending"
        self.f.save("specs/active_frontier.json", manifest)
        self.f.stage()
        self.f.git("commit", "-qm", "synthetic ready authority")
        authority = self.f.git("rev-parse", "HEAD").decode().strip()
        trust = Trust(Tree(self.f.root), authority, authority)
        (self.f.root / self.f.contract["proof_path"]).unlink()
        self.assertEqual(validate_delivery(Tree(self.f.root), item, trust=trust), "ready")
        self.f.write("src/read.py", b"changed activation source")
        with self.assertRaises(DeliveryInputError):
            validate_delivery(Tree(self.f.root), item, trust=trust)

    def test_owner_promoted_defect_disposition_binds_replay(self):
        from workflow_delivery_evidence_shapes import review_sha256
        reference = {"path": "docs/reviews/defect.md", "marker": "<!-- DEFECT -->"}
        self.review["findings"] = [{"issue_id": "dat-test", "severity": "blocking",
                                    "disposition_ref": reference}]
        self.f.write(reference["path"], ("<!-- DEFECT -->\nRESOLVED dat-test\n"
                     f"REPLAY {self.review['replay']['sha256']}\n").encode())
        governance = Tree(self.f.root).json("specs/spec_governance_manifest.json")
        governance["entries"][reference["path"]] = {"class": "governed"}
        self.f.save("specs/spec_governance_manifest.json", governance)
        self.f.save(self.f.contract["review_path"], self.review)
        self.f.write("docs/reviews/owner.md", (
            "<!-- RECEIPT -->\n"
            f"ACCEPT TASK/A {self.review['packet_sha256']} {review_sha256(self.review)}\n"
            "Source: synthetic fixture\nDate: 2026-09-06\n").encode())
        self.f.stage()
        self.f.git("commit", "-qm", "synthetic defect resolution promotion")
        self.authority = self.f.git("rev-parse", "HEAD").decode().strip()
        self.assertEqual(self.check(), self.review)


if __name__ == "__main__":
    unittest.main()
