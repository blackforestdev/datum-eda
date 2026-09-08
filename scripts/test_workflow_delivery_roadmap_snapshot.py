"""Snapshot mechanics on synthetic repositories; not real-roadmap gate proof."""

import json
import unittest

from workflow_delivery_capture_state import protected_state
from workflow_delivery_roadmap_snapshot import prepare_roadmap_snapshot
from workflow_delivery_test_support import Fixture
from workflow_delivery_trust_test_support import accepted_fixture


class RoadmapSnapshotTest(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.close)
        _, _, _, self.pin = accepted_fixture(self.f)
        self.ref = "refs/datum/workflow-delivery-candidates/" + self.pin
        self.f.git("update-ref", self.ref, self.pin)
        self.f.git("config", "datum.workflowDeliveryAuthorityRef", self.pin)
        self.destination = self.f.root / ".git/datum-wdq/proposals/snapshot-test"

    def prepare(self, **changes):
        args = dict(source_ref=self.ref, source_commit=self.pin, frontier_keys=["TASK", "NEXT"])
        args.update(changes)
        return prepare_roadmap_snapshot(self.f.root, self.destination, **args)

    def test_restore_preserves_all_committed_bytes_and_not_live_trust(self):
        before = protected_state(self.f.root, [])
        result = self.prepare()
        restored = self.destination / "snapshot"
        self.assertEqual(self.pin, result["source_commit"])
        self.assertFalse(result["activation_asserted"])
        self.assertFalse(result["restored_trust_installed"])
        self.assertEqual(before, protected_state(self.f.root, []))
        self.assertFalse(any(protected_state(restored, [])["local_trust"].values()))
        for name in ("specs/active_frontier.json", ".beads/issues.jsonl", "docs/reviews/review.json"):
            self.assertEqual((self.f.root / name).read_bytes(), (restored / name).read_bytes())
        self.assertEqual(result, json.loads((self.destination / "result.json").read_bytes()))

    def test_wrong_pin_identity_set_and_ref_refuse_without_output(self):
        for changes in ({"frontier_keys": ["TASK"]}, {"frontier_keys": ["TASK", "TASK"]},
                        {"source_ref": "HEAD"}, {"source_commit": self.f.head}):
            with self.subTest(changes=changes):
                with self.assertRaises(ValueError):
                    self.prepare(**changes)
                self.assertFalse(self.destination.exists())

    def test_dirty_source_and_occupied_destination_are_not_repaired(self):
        self.f.write("uncommitted-note.txt", b"owned dirty fixture input\n")
        with self.assertRaisesRegex(ValueError, "clean"):
            self.prepare()
        self.assertEqual(b"owned dirty fixture input\n", (self.f.root / "uncommitted-note.txt").read_bytes())
        self.assertFalse(self.destination.exists())
        self.destination.mkdir(parents=True)
        with self.assertRaisesRegex(ValueError, "fresh direct child"):
            self.prepare()

    def test_external_destination_refuses_before_any_creation(self):
        self.destination = self.f.root / "outside-store"
        with self.assertRaisesRegex(ValueError, "Git-common proposals"):
            self.prepare()
        self.assertFalse(self.destination.exists())


if __name__ == "__main__":
    unittest.main()
