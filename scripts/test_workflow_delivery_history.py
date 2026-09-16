"""Historical acceptance is immutable evidence, not a freeze on later source."""

from copy import deepcopy
import io
import json
from contextlib import redirect_stdout
import unittest
from unittest.mock import patch

from check_workflow_delivery import main
from workflow_delivery_checkpoints import validate_delivery
from workflow_delivery_io import DeliveryInputError
from workflow_delivery_native_test_support import environment
from workflow_delivery_test_support import Fixture
from workflow_delivery_tree import Tree
from workflow_delivery_trust import Trust, policy_shape
from workflow_delivery_trust_test_support import accepted_fixture


class HistoricalDeliveryTest(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.close)
        self.item, _, _, self.archive = accepted_fixture(self.f)
        policy = Tree(self.f.root).json("specs/workflow_delivery_policy.json")
        policy["enrolled"][0]["historical_ref"] = self.archive
        self.f.save("specs/workflow_delivery_policy.json", policy)
        self.f.save("requested-environment.json", environment())
        self.f.stage()
        self.f.git("commit", "-qm", "test(history): select immutable synthetic delivery\n\nFixture only.")
        self.authority = self.f.git("rev-parse", "HEAD").decode().strip()

    def check(self, item=None, *, phase=None, staged=False):
        tree = Tree(self.f.root, staged=staged)
        trust = Trust(tree, self.authority, self.authority)
        return validate_delivery(tree, item or self.item, phase=phase, trust=trust,
                                 environment=environment())

    def test_later_source_passes_without_reaccepting_original(self):
        self.f.write("src/read.py", b"changed implementation\n")
        self.assertEqual(self.check(), "historical:accept@" + self.archive)
        self.f.stage()
        self.assertEqual(self.check(staged=True), "historical:accept@" + self.archive)

    def test_explicit_current_acceptance_still_requires_current_proof(self):
        self.f.write("src/read.py", b"untested current candidate\n")
        with self.assertRaises(DeliveryInputError) as caught:
            self.check(phase="accept")
        self.assertEqual(caught.exception.code, "WDQ-STALE")

    def test_candidate_cannot_change_pin(self):
        policy = Tree(self.f.root).json("specs/workflow_delivery_policy.json")
        policy["enrolled"][0]["historical_ref"] = self.authority
        self.f.save("specs/workflow_delivery_policy.json", policy)
        with self.assertRaises(DeliveryInputError) as caught:
            self.check()
        self.assertEqual(caught.exception.code, "WDQ-POLICY")

    def test_history_cannot_reopen_or_change_completed_work(self):
        for mutate in (
            lambda i: i.update(state="in_progress", authorization="execution"),
            lambda i: i.update(claim={"agent": "writer"}),
            lambda i: i.update(landing_commit=self.authority),
            lambda i: i["completion"].update(outcome="new acceptance claim"),
            lambda i: i["completion"]["steps"][-1].update(status="pending"),
        ):
            with self.subTest(mutation=mutate):
                item = deepcopy(self.item)
                mutate(item)
                with self.assertRaises(DeliveryInputError) as caught:
                    self.check(item)
                self.assertEqual(caught.exception.code, "WDQ-TRANSITION")

    def test_archive_uses_original_evidence_not_rewritten_worktree(self):
        self.f.write(self.f.contract["proof_path"], b"not historical proof\n")
        self.assertEqual(self.check(), "historical:accept@" + self.archive)
        with self.assertRaises(DeliveryInputError):
            self.check(phase="accept")

    def test_cli_labels_historical_result_without_accepting_current_code(self):
        self.f.write("src/read.py", b"later source\n")
        output = io.StringIO()
        with redirect_stdout(output):
            code = main(["--root", str(self.f.root), "--enforce",
                         "--authority-ref", self.authority, "--base-ref", self.authority,
                         "--environment-path", "requested-environment.json"])
        report = json.loads(output.getvalue())
        self.assertEqual(code, 0, report)
        self.assertEqual(report["checks"], ["TASK: historical:accept@" + self.archive])
        self.assertFalse(report["acceptance_asserted"])
        self.assertFalse(report["readiness_asserted"])

    def test_history_pin_requires_full_commit_identity(self):
        policy = Tree(self.f.root).json("specs/workflow_delivery_policy.json")
        for invalid in (None, "HEAD", "", 42):
            policy["enrolled"][0]["historical_ref"] = invalid
            with self.assertRaises(DeliveryInputError):
                policy_shape(policy)

    def test_current_environment_cannot_invalidate_archived_acceptance(self):
        tree = Tree(self.f.root)
        trust = Trust(tree, self.authority, self.authority)
        changed = environment()
        changed["scale"] = 2
        self.assertEqual(validate_delivery(tree, self.item, trust=trust, environment=changed),
                         "historical:accept@" + self.archive)
        with self.assertRaises(DeliveryInputError) as caught:
            validate_delivery(tree, self.item, phase="accept", trust=trust, environment=changed)
        self.assertEqual(caught.exception.code, "WDQ-ENVIRONMENT")

    def test_explicit_verification_cannot_downgrade_to_structure(self):
        self.f.write("src/read.py", b"unverified implementation\n")
        with patch("workflow_delivery_checkpoints.preparation_bootstrap_structure",
                   return_value=True) as bootstrap:
            for phase in ("verify", "accept"):
                with self.subTest(phase=phase), self.assertRaises(DeliveryInputError) as caught:
                    self.check(phase=phase)
                self.assertEqual(caught.exception.code, "WDQ-STALE")
            bootstrap.assert_not_called()


if __name__ == "__main__":
    unittest.main()
