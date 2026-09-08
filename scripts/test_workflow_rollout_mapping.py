"""Roadmap/mapping consistency, not installed delivery enforcement or proof."""

import json
from pathlib import Path
import re
import subprocess
import unittest


ROOT = Path(__file__).resolve().parents[1]
PATH = "docs/reviews/workflow-delivery-rollout/rollout-mapping.json"


class RolloutMappingTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.proposal = json.loads((ROOT / PATH).read_text())
        cls.item = next(i for i in json.loads((ROOT / "specs/active_frontier.json").read_text())[
            "frontier"] if i["key"] == cls.proposal["frontier_key"])
        cls.steps = {s["id"]: s for s in cls.item["completion"]["steps"]}

    def ancestors(self, step_id):
        found, pending = set(), list(self.steps[step_id]["depends_on"])
        while pending:
            current = pending.pop()
            self.assertIn(current, self.steps)
            self.assertNotEqual(current, step_id)
            if current not in found:
                found.add(current)
                pending.extend(self.steps[current]["depends_on"])
        return found

    def test_closed_proposal_does_not_assert_activation_or_readiness(self):
        self.assertEqual({"schema_version", "record_type", "basis_ref", "frontier_key",
            "issue_id", "category", "delivery", "prior_preparation_steps", "owner_boundaries",
            "readiness_asserted", "enrollment_asserted", "acceptance_asserted"}, set(self.proposal))
        self.assertEqual(1, self.proposal["schema_version"])
        self.assertEqual("delivery_mapping_proposal", self.proposal["record_type"])
        for field in ("readiness_asserted", "enrollment_asserted", "acceptance_asserted"):
            self.assertIs(False, self.proposal[field])

    def test_mapping_uses_distinct_actual_infrastructure_step_kinds(self):
        delivery = self.proposal["delivery"]
        self.assertEqual({"schema_version", "contract_path", "checkpoints"}, set(delivery))
        self.assertEqual(2, delivery["schema_version"])
        self.assertEqual("infrastructure", self.proposal["category"])
        self.assertEqual(self.item["issue_id"], self.proposal["issue_id"])
        points = delivery["checkpoints"]
        self.assertEqual({"ready", "activate", "verify", "review", "accept"}, set(points))
        self.assertIsNone(points["activate"])
        self.assertIsNone(points["accept"])
        active = [points[p] for p in ("ready", "verify", "review")]
        self.assertEqual(3, len(set(active)))
        self.assertEqual(["governance", "execution", "execution"],
                         [self.steps[s]["kind"] for s in active])

    def test_readiness_verification_and_independent_review_precede_owner_activation(self):
        points = self.proposal["delivery"]["checkpoints"]
        self.assertIn(points["ready"], self.ancestors(points["verify"]))
        self.assertIn(points["verify"], self.ancestors(points["review"]))
        self.assertIn(points["review"], self.ancestors("WDQ-I04"))
        self.assertIn("WDQ-I04", self.ancestors("WDQ-I06"))

    def test_original_preparation_is_not_relabelled_readiness(self):
        revision = self.proposal["basis_ref"]
        self.assertIsNotNone(re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", revision))
        raw = subprocess.check_output(["git", "show", revision + ":specs/active_frontier.json"],
                                      cwd=ROOT)
        original = next(i for i in json.loads(raw)["frontier"] if i["key"] == self.item["key"])
        original_steps = {s["id"]: s for s in original["completion"]["steps"]}
        for sid in self.proposal["prior_preparation_steps"]:
            self.assertEqual("execution", original_steps[sid]["kind"])
            self.assertEqual("execution", self.steps[sid]["kind"])
            self.assertNotIn(sid, self.proposal["delivery"]["checkpoints"].values())

    def test_owner_boundaries_remain_separate_owner_decisions(self):
        self.assertEqual(["WDQ-I04", "WDQ-I06"], self.proposal["owner_boundaries"])
        for sid in self.proposal["owner_boundaries"]:
            self.assertEqual("owner_decision", self.steps[sid]["kind"])
            self.assertTrue(self.steps[sid]["owner_input"]["requests"])

    def test_mapping_targets_a_contract_not_the_live_policy(self):
        self.assertEqual("specs/workflow_delivery/rollout.contract.json",
                         self.proposal["delivery"]["contract_path"])
        self.assertNotEqual("specs/workflow_delivery_policy.json", PATH)


if __name__ == "__main__":
    unittest.main()
