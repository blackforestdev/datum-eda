"""Snapshot mechanics on synthetic repositories; not real-roadmap gate proof."""

import json
import unittest

from workflow_delivery_capture_state import git, protected_state
from workflow_delivery_roadmap_snapshot import prepare_roadmap_snapshot, capture_roadmap_snapshot
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

    def test_synthetic_snapshot_cannot_claim_real_accepted_pilot_capture(self):
        self.prepare()
        output = self.destination.with_name("not-real-pilot")
        for surface in ("C", "R", "S-check", "S-details", "H"):
            with self.subTest(surface=surface):
                with self.assertRaisesRegex(ValueError, "real accepted pilot"):
                    capture_roadmap_snapshot(self.destination, output, self.f.root, {}, surface=surface)
                self.assertFalse(output.exists())

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

    def test_nonancestor_evidence_requires_explicit_export_and_retains_failure(self):
        tree = self.f.git("rev-parse", "HEAD^{tree}").decode().strip()
        evidence = self.f.git("commit-tree", tree, "-m", "Detached fixture evidence").decode().strip()
        evidence_ref = "refs/datum/workflow-delivery-candidates/" + evidence
        self.f.git("update-ref", evidence_ref, evidence)
        path = "specs/active_frontier.json"
        frontier = json.loads((self.f.root / path).read_bytes())
        frontier["frontier"][0]["completion_evidence"] = [{"revision": evidence}]
        self.f.save(path, frontier)
        self.f.git("add", path)
        self.f.git("commit", "-qm", "Reference detached fixture evidence")
        self.pin = self.f.git("rev-parse", "HEAD").decode().strip()
        self.ref = "refs/datum/workflow-delivery-candidates/" + self.pin
        self.f.git("update-ref", self.ref, self.pin)
        before = protected_state(self.f.root, [])
        with self.assertRaisesRegex(ValueError, "referenced Frontier commit absent"):
            self.prepare()
        failed = self.destination
        failure = (failed / "failure.json").read_bytes()
        self.assertIn(evidence, json.loads(failure)["detail"])
        self.destination = failed.with_name("snapshot-with-evidence")
        with self.assertRaisesRegex(ValueError, "evidence export ref differs"):
            self.prepare(evidence_refs={evidence_ref: self.pin})
        self.assertFalse(self.destination.exists())
        result = self.prepare(evidence_refs={evidence_ref: evidence})
        self.assertIn(evidence, result["resolved_frontier_commits"])
        self.assertEqual(evidence.encode() + b"\n", git(
            self.destination / "snapshot", "rev-parse", "--verify", evidence + "^{commit}"))
        self.assertEqual(before, protected_state(self.f.root, []))
        self.assertEqual(failure, (failed / "failure.json").read_bytes())


if __name__ == "__main__":
    unittest.main()
