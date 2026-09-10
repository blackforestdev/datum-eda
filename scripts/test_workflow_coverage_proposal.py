"""Exact proposal inventory checks; no trust promotion or source authorization."""

import json
from pathlib import Path
import subprocess
import unittest

from workflow_delivery_authority import resolve_ref
from workflow_delivery_io import parse_json
from workflow_delivery_publication_delta import publication_delta


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
        self.assertTrue(set(scope["paths"]) <= set(contract["input_roots"]))
        self.assertNotIn("crates", scope["paths"])
        self.assertNotIn("scripts", scope["paths"])
        resolve_ref(self.tree, scope["boundary_ref"])

    def test_hook_read_dependencies_do_not_become_workflow_write_permissions(self):
        exemptions = self.tree.json("specs/rustfmt_exemption_manifest.json")["exemptions"]
        read_only = set(exemptions) | {"scripts/check_file_lane_ownership.py",
            "scripts/check_rustfmt.py", "specs/rustfmt_exemption_manifest.json"}
        for scope in self.coverage["source_scopes"]:
            for dependency in read_only:
                self.assertFalse(any(dependency == p or dependency.startswith(p + "/")
                                     for p in scope["paths"]))

    def test_reconciled_scope_covers_reviewed_history_not_only_final_diff(self):
        # Fixed preparation snapshot, never an authorization of subsequent history.
        delta = publication_delta(ROOT,
            base="7a77e40a850172b50e2251132d171d55a8adfb37",
            candidate="701c3b82363973857c47d16f46a8baa0a597c008")
        roots = self.coverage["production_roots"]
        required = {p for p in delta["touched_paths"]
                    if any(p == r or p.startswith(r + "/") for r in roots)}
        required.add("scripts/test_workflow_coverage_proposal.py")
        paths = self.coverage["source_scopes"][0]["paths"]
        self.assertEqual(sorted(set(paths)), paths)
        self.assertEqual([], sorted(required - set(paths)))
        # The mapping test changed in history but not in the final tree diff.
        self.assertIn("scripts/test_workflow_rollout_mapping.py", required)
        for path in paths:
            self.assertTrue((ROOT / path).is_file(), path)

    def test_assembled_policy_contains_exact_reviewed_scope_proposal(self):
        policy = self.tree.json("specs/workflow_delivery_policy.json")
        self.assertEqual(self.coverage, policy["coverage"])

    def test_compatibility_history_adds_only_eight_reviewed_source_paths(self):
        base = "2d60cd05dca71727ba28b1355b988de653eed82f"
        candidate = "afc3caa14ce253725c26fc9f8125f54eccdad309"
        old = json.loads(subprocess.check_output(
            ["git", "show", base + ":" + PATH], cwd=ROOT))
        expected = {"scripts/" + name + ".py" for name in (
            "workflow_delivery_source_only", "test_workflow_delivery_source_only",
            "workflow_delivery_workspace", "test_workflow_delivery_workspace",
            "test_workflow_delivery_workspace_consumers",
            "workflow_delivery_workspace_authority",
            "test_workflow_delivery_workspace_authority",
            "test_workflow_delivery_workspace_entrypoints")}
        historical = json.loads(subprocess.check_output(
            ["git", "show", "4e11d60b6f0ec50aa391c68ed39a0df138adf8cf:" + PATH], cwd=ROOT))
        paths = set(historical["source_scopes"][0]["paths"])
        self.assertEqual(set(old["source_scopes"][0]["paths"]) | expected, paths)
        self.assertEqual(old["production_roots"], self.coverage["production_roots"])
        delta = publication_delta(ROOT, base=base, candidate=candidate)
        required = {p for p in delta["touched_paths"] if any(
            p == r or p.startswith(r + "/") for r in self.coverage["production_roots"])}
        self.assertEqual([], sorted(required - paths))

    def test_sequencing_repair_adds_only_three_exact_source_paths(self):
        old = json.loads(subprocess.check_output(["git", "show",
            "f19391af59f623ae762d28566f8279232a75d6a2:" + PATH], cwd=ROOT))
        expected = {"scripts/" + name + ".py" for name in (
            "test_workflow_delivery_initial_migration", "test_workflow_delivery_review_inspection",
            "workflow_delivery_review_inspection")}
        paths = set(self.coverage["source_scopes"][0]["paths"])
        self.assertEqual(set(old["source_scopes"][0]["paths"]) | expected, paths)
        self.assertEqual(old["production_roots"], self.coverage["production_roots"])
        self.assertEqual(152, len(paths))
        self.assertEqual(164, len(self.tree.json("specs/workflow_delivery/rollout.contract.json")["input_roots"]))

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
