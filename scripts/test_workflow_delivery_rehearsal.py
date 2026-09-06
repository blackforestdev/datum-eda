#!/usr/bin/env python3
"""G05 completion/freshness rehearsals in disposable synthetic repositories.

These exercise the full validator, never native behavior or real owner approval.
"""

from copy import deepcopy
import unittest

from workflow_delivery_checkpoints import validate_delivery
from workflow_delivery_io import DeliveryInputError
from workflow_delivery_native_test_support import environment, typed_proof
from workflow_delivery_test_support import Fixture
from workflow_delivery_tree import Tree
from workflow_delivery_trust import Trust
from workflow_delivery_trust_test_support import accepted_fixture


class CompletionRehearsal(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.close)

    def test_p04_infrastructure_finishes_without_product_acceptance(self):
        typed_proof(self.f)
        item = {
            "key": "TASK", "issue_id": "dat-test", "authorization": "none",
            "completion": {
                "canonical_next_step_id": None,
                "delivery": {"contract_path": "contract.json", "checkpoints": {
                    "ready": "R", "activate": None, "verify": "V", "accept": None}},
                "steps": [
                    {"id": "R", "kind": "planning", "status": "complete", "depends_on": []},
                    {"id": "V", "kind": "execution", "status": "complete", "depends_on": ["R"]},
                ],
            },
        }
        before, original = self.f.snapshot(), deepcopy(item)
        self.assertEqual(validate_delivery(Tree(self.f.root), item,
                                          environment=environment()), "verify")
        self.assertEqual(item, original)
        self.assertEqual(self.f.snapshot(), before)
        self.assertFalse((self.f.root / self.f.contract["review_path"]).exists())
        self.assertNotIn("claim", item)
        self.assertEqual(item["completion"]["canonical_next_step_id"], None)

    def test_n10_p05_accepted_packet_freshness_preserves_history_and_selection(self):
        item, _, _, authority = accepted_fixture(self.f)
        trust = Trust(Tree(self.f.root), authority, authority)

        def check():
            before, original = self.f.snapshot(), deepcopy(item)
            try:
                return validate_delivery(Tree(self.f.root), item, trust=trust,
                                         environment=environment())
            finally:
                self.assertEqual(item, original)
                self.assertEqual(self.f.snapshot(), before)

        history = self.f.snapshot()
        self.assertEqual(check(), "accept")
        self.f.write("notes/unrelated.md", b"Unrelated synthetic work.\n")
        self.f.stage()
        self.f.git("commit", "-qm", "synthetic unrelated history movement")
        self.assertNotEqual(self.f.git("rev-parse", "HEAD").decode().strip(), authority)
        self.assertEqual(check(), "accept")

        original_source = history["src/read.py"]
        for mutation in ("changed", "added", "deleted"):
            with self.subTest(mutation=mutation):
                if mutation == "changed":
                    self.f.write("src/read.py", b"untested relevant handler\n")
                elif mutation == "added":
                    self.f.write("src/new.py", b"untested new dependency\n")
                else:
                    (self.f.root / "src/read.py").unlink()
                with self.assertRaises(DeliveryInputError) as caught:
                    check()
                # A deleted registered handler fails reference resolution before
                # the manifest freshness check; both paths refuse acceptance.
                expected = "WDQ-ARTIFACT" if mutation == "deleted" else "WDQ-STALE"
                self.assertEqual(caught.exception.code, expected)
                self.f.write("src/read.py", original_source)
                if mutation == "added":
                    (self.f.root / "src/new.py").unlink()
                self.assertEqual(check(), "accept")

        # The validator neither rewrites the receipt nor picks another task.
        for path in ("docs/reviews/owner.md", "specs/active_frontier.json",
                     "specs/PROGRESS.md", ".beads/issues.jsonl"):
            self.assertEqual((self.f.root / path).read_bytes(), history[path])


if __name__ == "__main__":
    unittest.main()
