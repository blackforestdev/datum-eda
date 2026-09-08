"""Specification review/ratification contracts, not native product proof."""

import copy
import json
from pathlib import Path
import unittest

from project_task_details import validate_completion
from workflow_delivery_authority import resolve_ref
from workflow_delivery_io import parse_json


ROOT = Path(__file__).resolve().parents[1]
SPECS = {
    "PROJECT-WRITER-OWNERSHIP": "project-writer-ownership",
    "NATIVE-CONNECTIVITY-CONTRACT": "native-connectivity",
    "NATIVE-WORKFLOW-BUDGETS": "native-workflow-budgets",
}


class RepositoryView:
    def read(self, path):
        return (ROOT / path).read_bytes()

    def json(self, path):
        return parse_json(self.read(path), path)


class FoundationSpecificationClausesTest(unittest.TestCase):
    def setUp(self):
        self.tree = RepositoryView()
        self.items = {i["key"]: i for i in self.tree.json("specs/active_frontier.json")["frontier"]
                      if i["key"] in SPECS}
        self.issues = {i["id"]: i for i in map(json.loads,
                       (ROOT / ".beads/issues.jsonl").read_text().splitlines())}
        self.governed = self.tree.json("specs/spec_governance_manifest.json")["entries"]

    def inventory(self, key):
        return self.tree.json("docs/reviews/workflow-delivery-rollout/clauses/"
                              + SPECS[key] + ".json")

    def test_actual_plans_match_tracker_and_governed_requirements(self):
        self.assertEqual(set(SPECS), set(self.items))
        for key, item in self.items.items():
            with self.subTest(key=key):
                self.assertEqual([], validate_completion(ROOT, item, self.issues, self.governed))

    def test_review_precedes_owner_without_authorizing_execution(self):
        for key, item in self.items.items():
            with self.subTest(key=key):
                steps = item["completion"]["steps"]
                self.assertEqual(["planning", "planning", "planning", "owner_decision"],
                                 [s["kind"] for s in steps])
                self.assertTrue(steps[2]["id"].endswith("-REVIEW"))
                for previous, current in zip(steps, steps[1:]):
                    self.assertEqual([previous["id"]], current["depends_on"])

    def test_owner_cannot_select_ratification_before_independent_review(self):
        for key, original in self.items.items():
            with self.subTest(key=key):
                item = copy.deepcopy(original)
                item["authorization"] = "owner_decision"
                item["completion"]["canonical_next_step_id"] = item["completion"]["steps"][-1]["id"]
                errors = validate_completion(ROOT, item, self.issues, self.governed)
                self.assertTrue(any("incomplete dependencies" in e for e in errors), errors)

    def test_ten_distinct_clauses_cover_all_requirements_and_resolve_authority(self):
        for key, item in self.items.items():
            with self.subTest(key=key):
                inventory = self.inventory(key)
                clauses = inventory["clauses"]
                self.assertEqual(key, inventory["frontier_key"])
                self.assertEqual(10, len({c["id"] for c in clauses}))
                self.assertEqual(10, len(clauses))
                expected = {(s["id"], r["path"], r["marker"])
                            for s in item["completion"]["steps"] for r in s["requirement_refs"]}
                covered = {(c["step_id"], c["requirement_ref"]["path"], c["requirement_ref"]["marker"])
                           for c in clauses}
                self.assertEqual(expected, covered)
                for clause in clauses:
                    for authority in clause["authority_refs"]:
                        resolve_ref(self.tree, authority)

    def test_owner_clauses_are_not_satisfied_by_independent_review(self):
        for key in self.items:
            with self.subTest(key=key):
                clauses = {c["id"]: c for c in self.inventory(key)["clauses"]}
                self.assertIs(True, clauses["OWNER-RATIFICATION"]["requires_owner_decision"])
                self.assertIs(False, clauses["INDEPENDENT-REVIEW"]["requires_owner_decision"])
                self.assertNotEqual(clauses["OWNER-RATIFICATION"]["step_id"],
                                    clauses["INDEPENDENT-REVIEW"]["step_id"])

    def test_specification_closure_never_selects_or_authorizes_successor(self):
        for item in self.items.values():
            post = item["completion"]["post_completion"]
            self.assertIs(False, post["authorizes_successor"])
            self.assertIs(False, post["selects_successor"])
            self.assertEqual("explicit_frontier_update", post["selection"])


if __name__ == "__main__":
    unittest.main()
