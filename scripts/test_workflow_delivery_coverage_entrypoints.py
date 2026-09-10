"""Schema-2 checks through the real staged CLI and selector on hermetic Git."""

from contextlib import redirect_stdout
import io
import json
import unittest

from check_workflow_delivery import main
from project_status import render_block
from workflow_delivery_selector import selector_failures
from workflow_delivery_io import canonical_json
from workflow_delivery_native_test_support import environment
from workflow_delivery_test_support import Fixture
from workflow_delivery_tree import Tree
from workflow_delivery_trust_test_support import accepted_fixture
from workflow_delivery_clause_test_support import inventory


class CoverageEntrypointsTest(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.close)
        _, _, _, baseline = accepted_fixture(self.f)
        self.manifest = Tree(self.f.root).json("specs/active_frontier.json")
        next_item = self.manifest["frontier"][1]
        next_item.update(state="specified", authorization="planning")
        next_item["completion"]["steps"][0]["kind"] = "planning"
        self.f.save("specs/active_frontier.json", self.manifest)
        self.f.write("specs/PROGRESS.md", render_block(self.manifest).encode())
        policy = Tree(self.f.root).json("specs/workflow_delivery_policy.json")
        policy.update(schema_version=2, coverage={
            "baseline_ref": baseline, "new_item_rule": "classification_required",
            "production_roots": ["src"], "source_scopes": [],
            "rows": [{"frontier_key": i["key"], "issue_id": i["issue_id"],
                      "category": "historical" if i["state"] == "landed" else "specification",
                      "boundary_ref": self.f.ref, "external_handoff_ref": None}
                     for i in self.manifest["frontier"]]})
        inventory(self.f, next_item, policy)
        self.f.save("specs/workflow_delivery_policy.json", policy)
        selected = self.f.blob("selected-environment.json", canonical_json(environment()))
        self.f.save("requested-environment.json", {
            "schema_version": 2, "kind": "datum.workflow-delivery.environments",
            "environments": [{"frontier_key": row["frontier_key"], "environment": selected}
                             for row in policy["enrolled"]]})
        self.f.stage()
        self.f.git("commit", "-qm", "test(coverage): pin synthetic schema-2 authority\n\nTest fixture only, not Datum promotion.")
        self.authority = self.f.git("rev-parse", "HEAD").decode().strip()
        self.f.git("config", "datum.workflowDeliveryAuthorityRef", self.authority)
        self.f.git("config", "datum.workflowDeliveryBaseRef", self.authority)
        self.f.git("config", "datum.workflowDeliveryEnvironmentPath", "requested-environment.json")

    def cli(self, candidate_ref=None):
        before = self.f.snapshot()
        output = io.StringIO()
        with redirect_stdout(output):
            view = ["--candidate-ref", candidate_ref] if candidate_ref else ["--staged"]
            result = main(["--root", str(self.f.root), "--enforce", *view,
                           "--authority-ref", self.authority, "--base-ref", self.authority,
                           "--environment-path", "requested-environment.json"])
        self.assertEqual(before, self.f.snapshot())
        return result, json.loads(output.getvalue())

    def test_valid_schema2_passes_both_actual_entrypoints(self):
        code, report = self.cli()
        self.assertEqual(0, code, report)
        self.assertEqual([], selector_failures(self.f.root, self.manifest))

    def test_candidate_environment_selection_change_refuses_cli_and_selector(self):
        selection = Tree(self.f.root).json("requested-environment.json")
        selection["environments"] = []
        self.f.save("requested-environment.json", selection)
        self.f.stage()
        code, report = self.cli()
        self.assertEqual(1, code, report)
        self.assertEqual("WDQ-ENVIRONMENT", report["findings"][0]["code"])
        self.assertTrue(any("environment selection change" in e
                            for e in selector_failures(self.f.root, self.manifest)))

    def test_changed_environment_blob_refuses_cli_and_selector(self):
        changed = environment()
        changed["toolchain"] = "candidate invented toolchain"
        self.f.save("selected-environment.json", changed)
        self.f.stage()
        code, report = self.cli()
        self.assertEqual(1, code, report)
        self.assertEqual("WDQ-ENVIRONMENT", report["findings"][0]["code"])
        self.assertTrue(any("artifact hash mismatch" in e
                            for e in selector_failures(self.f.root, self.manifest)))

    def test_staged_unscoped_source_refuses_before_native_proof(self):
        self.f.write("src/unknown.py", b"unapproved new implementation\n")
        self.f.stage()
        code, report = self.cli()
        self.assertEqual(1, code, report)
        self.assertEqual("WDQ-COVERAGE", report["findings"][0]["code"])
        self.assertEqual("src/unknown.py", report["findings"][0]["path"])
        self.assertTrue(any("no promoted scope" in e
                            for e in selector_failures(self.f.root, self.manifest)))

    def test_candidate_sequence_cannot_hide_reverted_unscoped_source(self):
        original = self.f.root.joinpath("src/read.py").read_bytes()
        self.f.write("src/read.py", b"Unscoped intermediate implementation.\n")
        self.f.stage()
        self.f.git("commit", "-qm", "fixture unscoped intermediate source")
        self.f.write("src/read.py", original)
        self.f.stage()
        self.f.git("commit", "-qm", "fixture restore final source")
        candidate = self.f.git("rev-parse", "HEAD").decode().strip()
        code, report = self.cli(candidate_ref=candidate)
        self.assertEqual(1, code, report)
        self.assertEqual("WDQ-COVERAGE", report["findings"][0]["code"])
        self.assertEqual("src/read.py", report["findings"][0]["path"])
        self.assertIn("no promoted scope", report["findings"][0]["detail"])
        code, report = self.cli()
        self.assertEqual(0, code, report)  # Empty new transaction, not history approval.

    def test_candidate_cannot_delete_coverage_or_self_grant_scope(self):
        policy = Tree(self.f.root).json("specs/workflow_delivery_policy.json")
        policy["coverage"]["source_scopes"] = [{"frontier_key": "NEXT", "step_ids": ["NEXT-C01"],
            "paths": ["src"], "boundary_ref": self.f.ref}]
        self.f.save("specs/workflow_delivery_policy.json", policy)
        self.f.stage()
        code, report = self.cli()
        self.assertNotEqual(0, code)
        self.assertEqual("WDQ-POLICY", report["findings"][0]["code"])

    def test_untracked_worktree_source_does_not_leak_into_staged_cli(self):
        self.f.write("src/untracked.py", b"uncommitted input\n")
        code, report = self.cli()
        self.assertEqual(0, code, report)
        self.assertTrue(any("no promoted scope" in e
                            for e in selector_failures(self.f.root, self.manifest)))

    def test_specification_cannot_turn_itself_into_execution(self):
        item = self.manifest["frontier"][1]
        item.update(state="ready", authorization="execution")
        item["completion"]["steps"][0]["kind"] = "execution"
        self.f.save("specs/active_frontier.json", self.manifest)
        self.f.write("specs/PROGRESS.md", render_block(self.manifest).encode())
        self.f.stage()
        code, report = self.cli()
        self.assertEqual(1, code, report)
        self.assertEqual("WDQ-COVERAGE", report["findings"][0]["code"])
        self.assertIn("cannot authorize implementation", report["findings"][0]["detail"])


if __name__ == "__main__":
    unittest.main()
