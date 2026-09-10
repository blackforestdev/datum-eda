"""Bounded evidence-import tests; no candidate publication or product proof."""

from pathlib import Path
import sys
import tempfile
import unittest

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(ROOT / ".git/datum-wdq/proposals/sequencing-repair-20260910/scripts"))

from prepare_sequencing_review_candidate import EVIDENCE, PRODUCER, main, reviewed_payloads
from workflow_delivery_io import canonical_json, sha256


class EvidenceImportTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="review-import-test-")
        self.addCleanup(self.temp.cleanup)
        self.overlay = Path(self.temp.name)
        self.review = EVIDENCE + "review.json"
        self.artifact = EVIDENCE + "workspace-renewal/independent-sequencing-typed/events.json"
        self.files = {self.review: b'{"test_only":true}\n', self.artifact: b"[]\n"}
        for path, raw in self.files.items():
            destination = self.overlay / path
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(raw)
        self.inventory = sha256(canonical_json([
            {"path": path, "sha256": sha256(raw), "size": len(raw)}
            for path, raw in sorted(self.files.items())]))
        self.calls = []

    def git(self, *args):
        self.calls.append(args)
        if args[0] == "ls-tree":
            self.assertEqual(args[2], PRODUCER)
            return b"100644 blob fixture-proof\t" + (EVIDENCE + "proof.json").encode() + b"\0"
        self.assertEqual(args, ("cat-file", "blob", "fixture-proof"))
        return b"synthetic producer proof\n"

    def load(self, **changes):
        args = dict(git=self.git, overlay=self.overlay,
                    review_digest=sha256(self.files[self.review]), inventory_digest=self.inventory)
        args.update(changes)
        return reviewed_payloads(**args)

    def test_exact_payload_import(self):
        payloads, inventory = self.load()
        self.assertEqual(set(payloads), {*self.files, EVIDENCE + "proof.json"})
        self.assertEqual(payloads[self.review], self.files[self.review])
        self.assertEqual(sha256(canonical_json(inventory)), self.inventory)
        self.assertTrue(all(call[0] in ("ls-tree", "cat-file") for call in self.calls))

    def test_wrong_review_digest_refuses(self):
        with self.assertRaises(AssertionError):
            self.load(review_digest="0" * 64)

    def test_changed_artifact_refuses(self):
        (self.overlay / self.artifact).write_bytes(b"changed artifact\n")
        with self.assertRaises(AssertionError):
            self.load()

    def test_out_of_lane_file_refuses(self):
        (self.overlay / "product.rs").write_bytes(b"not review evidence\n")
        with self.assertRaises(AssertionError):
            self.load()

    def test_symlink_refuses(self):
        (self.overlay / self.artifact).unlink()
        (self.overlay / self.artifact).symlink_to(self.overlay / self.review)
        with self.assertRaises(AssertionError):
            self.load()

    def test_incomplete_or_wrong_step_arguments_refuse_before_git(self):
        with self.assertRaises(ValueError):
            main(expected_step="WDQ-COMPAT", review_overlay=self.overlay,
                 review_digest="0" * 64, review_inventory_digest="0" * 64)
        with self.assertRaises(ValueError):
            main(expected_step="WDQ-RECHECK", review_overlay=self.overlay)


if __name__ == "__main__":
    unittest.main()
