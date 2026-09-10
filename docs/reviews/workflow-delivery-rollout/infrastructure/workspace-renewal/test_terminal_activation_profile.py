"""Evidence-lane import regressions; no publication or acceptance."""

from pathlib import Path
import sys
import unittest

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from test_prepare_sequencing_review_candidate import EvidenceImportTest
from terminal_activation_profile import EVIDENCE, PRODUCER, REVIEW_PREFIX, reviewed_payloads
from workflow_delivery_io import sha256


class TerminalEvidenceImportTest(EvidenceImportTest):
    def setUp(self):
        super().setUp()
        old = self.artifact
        self.artifact = REVIEW_PREFIX + "events.json"
        raw = self.files.pop(old)
        (self.overlay / old).unlink()
        destination = self.overlay / self.artifact
        destination.parent.mkdir(parents=True)
        destination.write_bytes(raw)
        self.files[self.artifact] = raw
        from workflow_delivery_io import canonical_json
        self.inventory = sha256(canonical_json([
            {"path": path, "sha256": sha256(raw), "size": len(raw)}
            for path, raw in sorted(self.files.items())]))
        self.producer_paths = [EVIDENCE + "proof.json", EVIDENCE + "terminal-renewal/raw.tar.xz",
                              EVIDENCE + "terminal-renewal/inventory.json",
                              EVIDENCE + "terminal-owner/inventory.json"]

    def git(self, *args):
        self.calls.append(args)
        if args[0] == "ls-tree":
            self.assertEqual(args[2], PRODUCER)
            return b"".join(b"100644 blob fixture-proof\t" + path.encode() + b"\0"
                            for path in self.producer_paths)
        self.assertEqual(args, ("cat-file", "blob", "fixture-proof"))
        return b"synthetic producer artifact\n"

    def load(self, **changes):
        args = dict(git=self.git, overlay=self.overlay,
                    review_digest=sha256(self.files[self.review]), inventory_digest=self.inventory)
        args.update(changes)
        return reviewed_payloads(**args)

    def test_exact_payload_import(self):
        payloads, _ = self.load()
        self.assertEqual(set(payloads), {*self.files, *self.producer_paths})
        self.assertNotIn(".beads/issues.jsonl", payloads)

    def test_producer_cannot_supply_review(self):
        self.producer_paths.append(REVIEW_PREFIX + "forged.json")
        with self.assertRaises(AssertionError):
            self.load()

    def test_missing_raw_custody_refuses(self):
        self.producer_paths.remove(EVIDENCE + "terminal-renewal/raw.tar.xz")
        with self.assertRaises(AssertionError):
            self.load()


if __name__ == "__main__":
    unittest.main()
