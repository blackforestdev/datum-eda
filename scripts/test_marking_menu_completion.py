"""Pending inert-shell delivery boundaries, not native menu behavior proof."""

from copy import deepcopy
import json
from pathlib import Path
import unittest

from project_task_details import validate_completion


ROOT = Path(__file__).resolve().parents[1]


class MarkingMenuCompletionTest(unittest.TestCase):
    def setUp(self):
        manifest = json.loads((ROOT / "specs/active_frontier.json").read_text())
        self.item = next(i for i in manifest["frontier"] if i["key"] == "GUI-MARKING-MENU")
        self.issues = {i["id"]: i for i in map(json.loads,
            (ROOT / ".beads/issues.jsonl").read_text().splitlines())}
        self.governed = json.loads((ROOT / "specs/spec_governance_manifest.json").read_text())["entries"]

    def validate(self, item):
        return validate_completion(ROOT, item, self.issues, self.governed)

    def test_real_plan_resolves_and_matches_tracker(self):
        self.assertEqual([], self.validate(self.item))

    def test_six_distinct_ordered_delivery_boundaries(self):
        steps = self.item["completion"]["steps"]
        self.assertEqual(["planning", "owner_decision", "execution", "execution",
                          "execution", "owner_decision"], [s["kind"] for s in steps])
        self.assertEqual(6, len({s["id"] for s in steps}))
        self.assertEqual([], steps[0]["depends_on"])
        for previous, current in zip(steps, steps[1:]):
            self.assertEqual([previous["id"]], current["depends_on"])

    def test_owner_packets_cannot_be_removed(self):
        for index in (1, 5):
            item = deepcopy(self.item)
            del item["completion"]["steps"][index]["owner_input"]
            self.assertTrue(any("owner_input" in e for e in self.validate(item)))

    def test_handoff_does_not_authorize_write_path(self):
        post = self.item["completion"]["post_completion"]
        self.assertEqual("explicit_frontier_update", post["selection"])
        self.assertIs(False, post["authorizes_successor"])
        self.assertIs(False, post["selects_successor"])

    def test_removing_native_proof_or_independent_review_refuses(self):
        for index in (3, 4):
            item = deepcopy(self.item)
            item["completion"]["steps"].pop(index)
            self.assertTrue(self.validate(item))


if __name__ == "__main__":
    unittest.main()
