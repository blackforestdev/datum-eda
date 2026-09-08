"""Pending collaboration research/decision boundaries, not implementation proof."""

import json
from pathlib import Path
import unittest

from project_task_details import validate_completion
from workflow_delivery_authority import resolve_ref
from workflow_delivery_io import parse_json


ROOT = Path(__file__).resolve().parents[1]
INVENTORY = "docs/reviews/workflow-delivery-rollout/clauses/distributed-collaboration.json"


class RepositoryView:
    def read(self, path):
        return (ROOT / path).read_bytes()

    def json(self, path):
        return parse_json(self.read(path), path)


class CollaborationCompletionTest(unittest.TestCase):
    def setUp(self):
        self.tree = RepositoryView()
        self.item = next(i for i in self.tree.json("specs/active_frontier.json")["frontier"]
                         if i["key"] == "DISTRIBUTED-COLLAB-SPEC")
        self.steps = self.item["completion"]["steps"]
        self.inventory = self.tree.json(INVENTORY)

    def test_actual_plan_matches_tracker_and_governed_requirements(self):
        issues = {i["id"]: i for i in map(json.loads,
                  (ROOT / ".beads/issues.jsonl").read_text().splitlines())}
        governed = self.tree.json("specs/spec_governance_manifest.json")["entries"]
        self.assertEqual([], validate_completion(ROOT, self.item, issues, governed))

    def test_research_authorship_review_ratification_handoff_never_execute(self):
        self.assertEqual(["planning", "planning", "planning", "owner_decision", "governance"],
                         [s["kind"] for s in self.steps])
        self.assertEqual([], self.steps[0]["depends_on"])
        for previous, current in zip(self.steps, self.steps[1:]):
            self.assertEqual([previous["id"]], current["depends_on"])
        self.assertTrue(self.steps[3]["owner_input"]["requests"])

    def test_inventory_covers_each_step_requirement_with_resolving_authority(self):
        expected = {(s["id"], r["path"], r["marker"])
                    for s in self.steps for r in s["requirement_refs"]}
        covered = set()
        for clause in self.inventory["clauses"]:
            r = clause["requirement_ref"]
            covered.add((clause["step_id"], r["path"], r["marker"]))
            # PM025 resolves requirement IDs as REQ:key:step markers; owner
            # packet markers may also contain the bare ID. validate_completion
            # above checks that namespace. WDQ authority refs are exact text.
            for authority in clause["authority_refs"]:
                resolve_ref(self.tree, authority)
        self.assertEqual(expected, covered)
        self.assertEqual(self.item["key"], self.inventory["frontier_key"])

    def test_unratified_mechanism_families_require_owner_disposition(self):
        required = {"DURABLE-MECHANISM", "LIVE-MECHANISM", "CONFLICT-MECHANISM",
                    "SECURITY-MECHANISM", "RELEASE-MECHANISM", "OWNER-RATIFICATION"}
        clauses = {c["id"]: c for c in self.inventory["clauses"]}
        self.assertTrue(required <= set(clauses))
        self.assertEqual(len(clauses), len(self.inventory["clauses"]))
        for cid in required:
            self.assertIs(True, clauses[cid]["requires_owner_decision"])

    def test_specification_closure_does_not_authorize_or_select_implementation(self):
        post = self.item["completion"]["post_completion"]
        self.assertIs(False, post["authorizes_successor"])
        self.assertIs(False, post["selects_successor"])
        self.assertEqual("explicit_frontier_update", post["selection"])
        self.assertEqual([], post["unblocks_issue_ids"])


if __name__ == "__main__":
    unittest.main()
