"""Pending surface specification coverage without taking product ownership."""

import copy
import json
from pathlib import Path
import unittest

from project_task_details import validate_completion
from workflow_delivery_authority import resolve_ref
from workflow_delivery_io import parse_json


ROOT = Path(__file__).resolve().parents[1]
SPECS = {
    "GUI-SURFACE-SPECS": ("gui-surface-specs", 14),
    "PROJECT-PREFERENCES-SPEC": ("project-preferences-spec", 21),
}


class RepositoryView:
    def read(self, path):
        return (ROOT / path).read_bytes()

    def json(self, path):
        return parse_json(self.read(path), path)


class SurfaceSpecificationClausesTest(unittest.TestCase):
    def setUp(self):
        self.tree = RepositoryView()
        self.items = {i["key"]: i for i in self.tree.json("specs/active_frontier.json")["frontier"]
                      if i["key"] in SPECS}
        self.issues = {i["id"]: i for i in map(json.loads,
                       (ROOT / ".beads/issues.jsonl").read_text().splitlines())}
        self.governed = self.tree.json("specs/spec_governance_manifest.json")["entries"]

    def test_actual_plans_match_tracker_and_governed_requirements(self):
        self.assertEqual(set(SPECS), set(self.items))
        for item in self.items.values():
            self.assertEqual([], validate_completion(ROOT, item, self.issues, self.governed))

    def test_review_is_distinct_from_ratification_and_does_not_execute(self):
        for item in self.items.values():
            steps = item["completion"]["steps"]
            self.assertEqual("planning", steps[-2]["kind"])
            self.assertTrue(steps[-2]["id"].endswith("-REVIEW"))
            self.assertEqual("owner_decision", steps[-1]["kind"])
            self.assertEqual([steps[-2]["id"]], steps[-1]["depends_on"])
            self.assertNotIn("execution", [s["kind"] for s in steps])

    def test_pending_review_prevents_owner_selection(self):
        for original in self.items.values():
            item = copy.deepcopy(original)
            item["authorization"] = "owner_decision"
            item["completion"]["canonical_next_step_id"] = item["completion"]["steps"][-1]["id"]
            errors = validate_completion(ROOT, item, self.issues, self.governed)
            self.assertTrue(any("incomplete dependencies" in e for e in errors), errors)

    def test_complete_inventory_requirements_and_exact_sources_resolve(self):
        for key, (slug, count) in SPECS.items():
            with self.subTest(key=key):
                inventory = self.tree.json("docs/reviews/workflow-delivery-rollout/clauses/" + slug + ".json")
                clauses = inventory["clauses"]
                self.assertEqual(key, inventory["frontier_key"])
                self.assertEqual(count, len(clauses))
                self.assertEqual(count, len({c["id"] for c in clauses}))
                expected = {(s["id"], r["path"], r["marker"])
                            for s in self.items[key]["completion"]["steps"] for r in s["requirement_refs"]}
                covered = {(c["step_id"], c["requirement_ref"]["path"], c["requirement_ref"]["marker"])
                           for c in clauses}
                self.assertEqual(expected, covered)
                for clause in clauses:
                    for authority in clause["authority_refs"]:
                        resolve_ref(self.tree, authority)
                indexed = {c["id"]: c for c in clauses}
                self.assertIs(True, indexed["OWNER-RATIFICATION"]["requires_owner_decision"])
                self.assertIs(False, indexed["INDEPENDENT-REVIEW"]["requires_owner_decision"])

    def test_specification_closure_cannot_implicitly_select_or_authorize_build(self):
        for item in self.items.values():
            post = item["completion"]["post_completion"]
            self.assertIs(False, post["authorizes_successor"])
            self.assertIs(False, post["selects_successor"])
            self.assertEqual("explicit_frontier_update", post["selection"])


if __name__ == "__main__":
    unittest.main()
