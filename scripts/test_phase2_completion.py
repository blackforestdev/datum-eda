"""Structured P2.3/P2.4 handoffs, not native behavior or product acceptance."""

from copy import deepcopy
import json
from pathlib import Path
import unittest

from project_task_details import validate_completion


ROOT = Path(__file__).resolve().parents[1]
KEYS = ("GUI-P2-CROSSPROBE", "GUI-P2-INSPECTOR")


class Phase2CompletionTest(unittest.TestCase):
    def setUp(self):
        manifest = json.loads((ROOT / "specs/active_frontier.json").read_text())
        self.items = [i for i in manifest["frontier"] if i["key"] in KEYS]
        self.issues = {i["id"]: i for i in map(json.loads,
            (ROOT / ".beads/issues.jsonl").read_text().splitlines())}
        self.governed = json.loads((ROOT / "specs/spec_governance_manifest.json").read_text())["entries"]

    def validate(self, item):
        return validate_completion(ROOT, item, self.issues, self.governed)

    def test_both_real_plans_validate_with_their_tracker_and_requirements(self):
        self.assertEqual(set(KEYS), {i["key"] for i in self.items})
        for item in self.items:
            self.assertEqual([], self.validate(item), item["key"])

    def test_readiness_authorization_build_proof_review_acceptance_are_distinct(self):
        for item in self.items:
            steps = item["completion"]["steps"]
            self.assertEqual(["planning", "owner_decision", "execution", "execution",
                              "execution", "owner_decision"], [s["kind"] for s in steps])
            self.assertEqual(6, len({s["id"] for s in steps}))
            self.assertEqual([], steps[0]["depends_on"])
            for previous, current in zip(steps, steps[1:]):
                self.assertEqual([previous["id"]], current["depends_on"])

    def test_each_owner_boundary_has_its_own_decision_packet(self):
        for item in self.items:
            for step in item["completion"]["steps"]:
                if step["kind"] == "owner_decision":
                    self.assertTrue(step["owner_input"]["response_format"])
                    self.assertTrue(step["owner_input"]["requests"])
                else:
                    self.assertNotIn("owner_input", step)

    def test_no_implicit_successor_selection_or_parallel_execution(self):
        for item in self.items:
            completion = item["completion"]
            self.assertEqual({"max_in_progress_steps": 1,
                "dependency_independence_authorizes_parallelism": False}, completion["execution_policy"])
            post = completion["post_completion"]
            self.assertEqual("explicit_frontier_update", post["selection"])
            self.assertIs(False, post["authorizes_successor"])
            self.assertIs(False, post["selects_successor"])

    def test_removing_an_owner_packet_refuses(self):
        for original in self.items:
            item = deepcopy(original)
            del item["completion"]["steps"][1]["owner_input"]
            self.assertTrue(any("owner_input" in e for e in self.validate(item)))

    def test_requirement_marker_cannot_be_a_fabricated_reference(self):
        for original in self.items:
            item = deepcopy(original)
            item["completion"]["steps"][0]["requirement_refs"][0]["marker"] = "ABSENT"
            self.assertTrue(any("marker" in e for e in self.validate(item)))

    def test_skipping_review_breaks_the_canonical_acceptance_contract(self):
        for original in self.items:
            item = deepcopy(original)
            item["completion"]["steps"].pop(4)
            self.assertTrue(self.validate(item))


if __name__ == "__main__":
    unittest.main()
