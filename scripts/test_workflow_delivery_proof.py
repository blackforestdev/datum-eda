#!/usr/bin/env python3
"""Evidence integrity and snapshot tests; native/review checks remain separate."""

from copy import deepcopy
import unittest

from workflow_delivery_evidence_shapes import proof_sha256, review_shape, review_sha256
from workflow_delivery_io import DeliveryInputError, canonical_json
from workflow_delivery_proof import validate_proof
from workflow_delivery_test_support import Fixture
from workflow_delivery_tree import Tree


class ProofTests(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.close)
        self.proof = self.f.proof()

    def validate(self, **kwargs):
        return validate_proof(Tree(self.f.root, **kwargs), self.f.contract)

    def refused(self, code, **kwargs):
        before = self.f.snapshot()
        with self.assertRaises(DeliveryInputError) as caught:
            self.validate(**kwargs)
        self.assertEqual(caught.exception.code, code, str(caught.exception))
        self.assertEqual(before, self.f.snapshot())

    def save(self):
        self.f.save(self.f.contract["proof_path"], self.proof)

    def test_valid_integrity_is_not_native_acceptance(self):
        self.assertEqual(self.validate(), self.proof)

    def test_fail_blocked_unverified_and_missing_dimension(self):
        original = deepcopy(self.proof)
        for state in ("fail", "blocked", "unverified"):
            self.proof = deepcopy(original)
            self.proof["results"][0]["outcome"] = state
            self.save()
            self.refused("WDQ-RESULT")
        self.proof = original
        self.proof["results"][0]["assertions"][0]["dimension"] = "cancel"
        self.save()
        self.refused("WDQ-RESULT")

    def test_duplicate_and_unknown_scenario_results(self):
        self.proof["results"].append(deepcopy(self.proof["results"][0]))
        self.save()
        self.refused("WDQ-RESULT")
        self.proof["results"].pop()
        self.proof["results"][0]["scenario_id"] = "UNKNOWN"
        self.save()
        self.refused("WDQ-RESULT")

    def test_relevant_dirty_new_deleted_source(self):
        self.f.write("src/new.py", b"new")
        self.refused("WDQ-STALE")
        (self.f.root / "src/new.py").unlink()
        self.f.write("src/read.py", b"changed")
        self.refused("WDQ-STALE")
        (self.f.root / "src/read.py").unlink()
        with self.assertRaises(DeliveryInputError):
            self.validate()

    def test_unrelated_changes_and_later_head_preserve_proof(self):
        self.f.write("unrelated.txt", b"other session")
        self.f.stage()
        self.f.git("commit", "-qm", "unrelated movement")
        self.assertEqual(self.validate(), self.proof)

    def test_artifact_missing_empty_altered(self):
        for raw in (b"", b"different"):
            self.f.write("evidence/events.txt", raw)
            self.refused("WDQ-ARTIFACT")
        (self.f.root / "evidence/events.txt").unlink()
        self.refused("WDQ-ARTIFACT")

    def test_environment_change_refuses(self):
        self.f.write("evidence/environment.txt", b"different scale/backend")
        self.refused("WDQ-ENVIRONMENT")

    def test_unknown_defect_refuses(self):
        self.proof["results"][0]["defects"] = ["dat-missing"]
        self.save()
        self.refused("WDQ-DEFECT")

    def test_build_digest_disagrees(self):
        path = "evidence/build.json"
        build = Tree(self.f.root).json(path)
        build["input_manifest_sha256"] = "0" * 64
        self.proof["build"]["receipt"] = self.f.blob(path, canonical_json(build))
        self.save()
        self.refused("WDQ-STALE")

    def test_source_clean_cannot_hide_dirty_build(self):
        self.f.write("src/read.py", b"changed before test")
        self.proof = self.f.proof()
        self.refused("WDQ-STALE")
        self.proof["build"]["source_clean"] = False
        self.save()
        self.assertEqual(self.validate(), self.proof)

    def test_index_never_reads_worktree_source_or_receipt(self):
        self.f.write("src/read.py", b"other session unstaged source")
        self.assertEqual(self.validate(staged=True), self.proof)
        self.f.git("add", "src/read.py")
        self.f.write("src/read.py", b"def read(path):\n    return path.read_bytes()\n")
        self.refused("WDQ-INDEX", staged=True)

    def test_untracked_receipt_cannot_mask_index_absence(self):
        self.f.git("rm", "--cached", self.f.contract["proof_path"])
        self.refused("WDQ-INDEX", staged=True)
        self.refused("WDQ-ARTIFACT")

    def test_hash_excludes_receipt_and_normalizes_review_sets(self):
        review = {"schema_version": 1, "packet_sha256": "a" * 64,
                  "reviewer_session": "reviewer", "independent_of": ["writer", "original"],
                  "disposition": "approve", "replay": {"path": "replay.json", "sha256": "b" * 64},
                  "findings": [], "owner_receipt": None}
        review_shape(review, "review.json")
        before = review_sha256(review)
        review["independent_of"].reverse()
        review["owner_receipt"] = self.f.ref
        self.assertEqual(before, review_sha256(review))
        self.assertEqual(len(proof_sha256(self.proof)), 64)


if __name__ == "__main__":
    unittest.main()
