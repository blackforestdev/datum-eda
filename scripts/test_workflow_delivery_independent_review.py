"""Independent review is an evidenced checkpoint, never owner acceptance."""

from copy import deepcopy
import unittest

from workflow_delivery_authority import authority_sha256
from workflow_delivery_io import DeliveryInputError, canonical_json
from workflow_delivery_native_test_support import environment
from workflow_delivery_proof import packet_sha256
from workflow_delivery_review import validate_independent_review, validate_review
from workflow_delivery_test_support import Fixture
from workflow_delivery_tree import Tree
from workflow_delivery_trust import Trust
from workflow_delivery_trust_test_support import accepted_fixture


class IndependentReviewTest(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.close)
        self.item, self.proof, self.review, self.authority = accepted_fixture(self.f)
        self.review["owner_receipt"] = None
        self.save()

    def save(self):
        self.f.save(self.f.contract["review_path"], self.review)
        self.f.stage()

    def check(self, *, accept=False):
        before = self.f.snapshot()
        tree = Tree(self.f.root, staged=True)
        validator = validate_review if accept else validate_independent_review
        try:
            return validator(tree, self.f.contract, self.proof,
                authority_sha256(tree, self.f.contract),
                Trust(tree, self.authority, self.authority), self.item, environment())
        finally:
            self.assertEqual(before, self.f.snapshot())

    def refuse(self, code, *, accept=False):
        with self.assertRaises(DeliveryInputError) as error:
            self.check(accept=accept)
        self.assertEqual(code, error.exception.code, str(error.exception))

    def test_review_passes_without_owner_receipt_but_acceptance_refuses(self):
        self.assertEqual(self.review, self.check())
        self.refuse("WDQ-RECEIPT", accept=True)

    def test_known_findings_can_await_owner_disposition_not_acceptance(self):
        self.review["findings"] = [{"issue_id": "dat-next", "severity": "blocking",
                                    "disposition_ref": None}]
        self.save()
        self.assertEqual(self.review, self.check())
        self.refuse("WDQ-DEFECT", accept=True)

    def test_unknown_finding_refuses_even_before_owner_disposition(self):
        self.review["findings"] = [{"issue_id": "dat-unknown", "severity": "nonblocking",
                                    "disposition_ref": None}]
        self.save()
        self.refuse("WDQ-DEFECT")

    def test_unaccounted_producer_defect_refuses(self):
        self.proof["results"][0]["defects"] = ["dat-test"]
        self.review["packet_sha256"] = packet_sha256(
            self.f.contract, self.proof, authority_sha256(Tree(self.f.root), self.f.contract))
        self.save()
        self.refuse("WDQ-DEFECT")

    def test_self_or_unapproved_review_cannot_complete_review_phase(self):
        initial = deepcopy(self.review)
        for change in ({"reviewer_session": "writer"}, {"reviewer_session": "original"},
                       {"independent_of": ["original"]}, {"disposition": "revise"},
                       {"disposition": "reject"}):
            with self.subTest(change=change):
                self.review = deepcopy(initial)
                self.review.update(change)
                self.save()
                self.refuse("WDQ-REVIEW")

    def test_copied_replay_refuses_without_waiting_for_acceptance(self):
        self.review["replay"] = self.f.blob(
            self.review["replay"]["path"], canonical_json(self.proof))
        self.save()
        self.refuse("WDQ-REVIEW")

    def test_worktree_repair_cannot_replace_staged_review(self):
        valid = deepcopy(self.review)
        self.review["packet_sha256"] = "0" * 64
        self.save()
        self.f.save(self.f.contract["review_path"], valid)
        self.refuse("WDQ-STALE")


if __name__ == "__main__":
    unittest.main()
