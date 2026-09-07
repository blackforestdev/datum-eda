"""Preparation-only defect accounting; all authority here is synthetic."""

from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from workflow_delivery_io import DeliveryInputError
from workflow_delivery_prepare import prepare, live_state
from workflow_delivery_prepare_review import check_defect_accounting, check_review
from workflow_delivery_test_support import Fixture
from workflow_delivery_tree import Tree


class DefectPreparationTests(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.close)
        self.proof = {"results": [{"defects": ["dat-test"]}]}
        self.replay = {"results": [{"defects": []}]}
        self.ref = {"path": "docs/reviews/resolutions.md", "marker": "<!-- FIXED -->"}
        self.review = {"replay": {"sha256": "a" * 64}, "findings": [
            {"issue_id": "dat-test", "severity": "blocking", "disposition_ref": self.ref}]}
        self.f.write(".beads/issues.jsonl", b'{"id":"dat-test","status":"closed"}\n')
        self.f.write(self.ref["path"], ("<!-- FIXED -->\n## Resolution\nRESOLVED dat-test\n"
                     "REPLAY " + "a" * 64 + "\n").encode())
        governance = Tree(self.f.root).json("specs/spec_governance_manifest.json")
        governance["entries"][self.ref["path"]] = {"class": "governed"}
        self.f.save("specs/spec_governance_manifest.json", governance)

    def check(self):
        before = self.f.snapshot()
        try:
            return check_defect_accounting(Tree(self.f.root), self.proof, self.replay,
                                           self.review, "review.json")
        finally:
            self.assertEqual(before, self.f.snapshot())

    def test_closed_producer_defect_still_requires_review_finding(self):
        self.review["findings"] = []
        with self.assertRaisesRegex(DeliveryInputError, "missing review disposition.*dat-test"):
            self.check()

    def test_replay_only_defect_also_requires_finding(self):
        self.proof, self.replay = self.replay, self.proof
        self.review["findings"] = []
        with self.assertRaisesRegex(DeliveryInputError, "missing review disposition.*dat-test"):
            self.check()

    def test_closed_defect_with_exact_replay_resolution_passes_readonly(self):
        self.assertEqual(self.check(), ["dat-test"])

    def test_missing_reference_wrong_replay_and_open_blocker_refuse(self):
        for change in ("reference", "replay", "open"):
            with self.subTest(change=change):
                if change == "reference":
                    self.review["findings"][0]["disposition_ref"] = None
                elif change == "replay":
                    self.review["findings"][0]["disposition_ref"] = self.ref
                    self.review["replay"]["sha256"] = "b" * 64
                else:
                    self.review["replay"]["sha256"] = "a" * 64
                    self.f.write(".beads/issues.jsonl", b'{"id":"dat-test","status":"open"}\n')
                with self.assertRaises(DeliveryInputError):
                    self.check()

    def test_resolution_outside_selected_section_refuses(self):
        self.f.write(self.ref["path"], ("<!-- FIXED -->\n## First\nNo resolution\n"
                     "## Elsewhere\nRESOLVED dat-test\nREPLAY " + "a" * 64 + "\n").encode())
        with self.assertRaisesRegex(DeliveryInputError, "missing exact"):
            self.check()

    def test_unclassified_resolution_refuses(self):
        self.f.save("specs/spec_governance_manifest.json", {"entries": {}})
        with self.assertRaises(DeliveryInputError):
            self.check()

    def test_nonblocking_deferral_requires_explicit_line(self):
        self.review["findings"][0]["severity"] = "nonblocking"
        with self.assertRaisesRegex(DeliveryInputError, "missing exact"):
            self.check()
        self.f.write(self.ref["path"], b"<!-- FIXED -->\nDEFER dat-test\n")
        self.assertEqual(self.check(), ["dat-test"])


class PreparationIntegrationTests(unittest.TestCase):
    def test_preflight_failure_prevents_output_and_ref_creation(self):
        f = Fixture()
        self.addCleanup(f.close)
        before = live_state(f.root), f.git("show-ref"), f.snapshot()
        with tempfile.TemporaryDirectory() as parent:
            output = Path(parent) / "not-created"
            with patch("workflow_delivery_prepare.validate_frontier"), patch(
                    "workflow_delivery_prepare.check_review", side_effect=DeliveryInputError(
                        "WDQ-DEFECT", "review.json", "missing producer defect")):
                with self.assertRaisesRegex(DeliveryInputError, "missing producer defect"):
                    prepare(f.root, output)
            self.assertFalse(output.exists())
        self.assertEqual(before, (live_state(f.root), f.git("show-ref"), f.snapshot()))

    def test_review_preflight_never_claims_owner_acceptance(self):
        from workflow_delivery_native_test_support import environment
        from workflow_delivery_trust_test_support import accepted_fixture
        f = Fixture()
        self.addCleanup(f.close)
        _, _, review, _ = accepted_fixture(f)
        review["owner_receipt"] = None
        f.save(f.contract["review_path"], review)
        f.save("requested-environment.json", environment())
        f.stage()
        before = f.snapshot()
        result = check_review(Tree(f.root), "contract.json", "requested-environment.json")
        self.assertEqual(result["mode"], "preparation-review-only")
        for field in ("owner_acceptance_checked", "trust_checked", "promotion_performed"):
            self.assertIs(result[field], False)
        self.assertEqual(before, f.snapshot())


if __name__ == "__main__":
    unittest.main()
