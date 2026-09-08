"""Exact proposal inventory checks; no trust promotion or source authorization."""

import json
from pathlib import Path
import subprocess
import unittest

from workflow_delivery_authority import resolve_ref
from workflow_delivery_io import parse_json


ROOT = Path(__file__).resolve().parents[1]
PATH = "docs/reviews/workflow-delivery-rollout/coverage-proposal.json"


class RepositoryView:
    def read(self, path):
        return (ROOT / path).read_bytes()

    def json(self, path):
        return parse_json(self.read(path), path)


class CoverageProposalTest(unittest.TestCase):
    def setUp(self):
        self.tree = RepositoryView()
        self.coverage = self.tree.json(PATH)
        raw = subprocess.check_output([
            "git", "show", self.coverage["baseline_ref"] + ":specs/active_frontier.json"
        ], cwd=ROOT)
        self.items = {i["key"]: i for i in json.loads(raw)["frontier"]}
        self.rows = {r["frontier_key"]: r for r in self.coverage["rows"]}

    def test_all_baseline_frontier_identities_are_classified_once(self):
        self.assertEqual(set(self.items), set(self.rows))
        self.assertEqual(len(self.rows), len(self.coverage["rows"]))
        for key, row in self.rows.items():
            self.assertEqual(self.items[key]["issue_id"], row["issue_id"])
        self.assertEqual("classification_required", self.coverage["new_item_rule"])

    def test_classifications_preserve_approved_intent_and_explicit_landed_adjustment(self):
        original = self.tree.json("docs/reviews/workflow-delivery-rollout/proposal.json")
        expected = {r["frontier_key"]: r["category"] for r in original["classification_proposal"]}
        expected["WORKFLOW-DELIVERY-ROLLOUT"] = "historical"
        expected["WORKFLOW-DELIVERY-IMPLEMENTATION"] = "infrastructure"
        self.assertEqual(expected, {k: r["category"] for k, r in self.rows.items()})
        for key, row in self.rows.items():
            if row["category"] == "historical":
                self.assertEqual("landed", self.items[key]["state"])

    def test_every_boundary_resolves_and_specifications_use_their_own_inventory(self):
        for key, row in self.rows.items():
            resolve_ref(self.tree, row["boundary_ref"])
            if row["category"] == "specification":
                inventory = self.tree.json(row["boundary_ref"]["path"])
                self.assertEqual(key, inventory["frontier_key"])
                self.assertTrue(inventory["clauses"])
            if row["external_handoff_ref"] is not None:
                self.assertEqual("external_lane", row["category"])
                resolve_ref(self.tree, row["external_handoff_ref"])

    def test_only_infrastructure_has_proposed_source_permission(self):
        scopes = self.coverage["source_scopes"]
        self.assertEqual(1, len(scopes))
        scope = scopes[0]
        self.assertEqual("WORKFLOW-DELIVERY-IMPLEMENTATION", scope["frontier_key"])
        self.assertEqual(["WDQ-I03", "WDQ-REVIEW"], scope["step_ids"])
        contract = self.tree.json("specs/workflow_delivery/rollout.contract.json")
        self.assertEqual(contract["input_roots"], scope["paths"])
        self.assertNotIn("crates", scope["paths"])
        self.assertNotIn("scripts", scope["paths"])
        resolve_ref(self.tree, scope["boundary_ref"])

    def test_production_roots_cover_non_rust_code_without_git_metadata(self):
        roots = self.coverage["production_roots"]
        self.assertTrue({"crates", "mcp-server", "scripts", ".github"} <= set(roots))
        for scope in self.coverage["source_scopes"]:
            for path in scope["paths"]:
                self.assertTrue(any(path == r or path.startswith(r + "/") for r in roots))
                self.assertNotIn(".git", path.split("/"))

    def test_external_boundary_observation_does_not_authorize_a_writer(self):
        key = "GLOBAL-PREFERENCES-COMPLETION"
        self.assertEqual("external_lane", self.rows[key]["category"])
        self.assertIsNotNone(self.rows[key]["external_handoff_ref"])
        baseline = self.items[key]
        self.assertEqual("owner_decision", baseline["authorization"])
        self.assertEqual("GP-CM05V", baseline["completion"]["canonical_next_step_id"])
        self.assertIsNone(baseline.get("claim"))
        self.assertNotIn(key, {s["frontier_key"] for s in self.coverage["source_scopes"]})


if __name__ == "__main__":
    unittest.main()
