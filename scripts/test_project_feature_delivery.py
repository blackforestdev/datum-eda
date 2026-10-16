"""Reject paper-only feature availability and unjustified work blockers."""
from __future__ import annotations

import copy
import json
import subprocess
import unittest

import project_feature_delivery as delivery
import project_status as status
import test_project_status as fixtures


class FeatureDeliveryTests(unittest.TestCase):
    def setUp(self):
        self.fixture = fixtures.ProjectStatusTest()
        self.fixture.setUp()
        self.root = self.fixture.root
        self.item = self.fixture.manifest["frontier"][0]
        self.issue = self.fixture.issues[0]
        self.issue["issue_type"] = "feature"
        self.item["work_purpose"] = {"kind": "product_delivery", "rationale": "Deliver footprint editing."}
        self.contract = {
            "ordinary_workflow": ["Move footprint", "Undo and reopen"],
            "safety_requirements": ["Original import unchanged", "Cancel writes nothing"],
            "qualification_followups": ["Full accessibility qualification"],
            "prerequisite_reasons": {},
            "development_ready": False,
            "ordinary_evidence": [],
        }
        self.item["feature_delivery"] = self.contract

    def tearDown(self):
        self.fixture.tearDown()

    def failures(self):
        return delivery.validate_feature_delivery(self.root, self.item, self.issue)

    def evidence(self):
        source = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=self.root, text=True).strip()
        record = {
            "kind": "ordinary_application", "source_commit": source,
            "launch_command": "datum-gui --project-root saved-project",
            "workflow_results": {x: True for x in self.contract["ordinary_workflow"]},
            "safety_results": {x: True for x in self.contract["safety_requirements"]},
        }
        self.contract.update(development_ready=True, ordinary_evidence=["ordinary.json"])
        (self.root / "ordinary.json").write_text(json.dumps(record))
        return record

    def test_selected_feature_requires_workflow_contract(self):
        del self.item["feature_delivery"]
        self.assertIn("requires an ordinary-workflow contract", " ".join(self.failures()))

    def test_explicit_maintenance_is_distinct_from_product_delivery(self):
        del self.item["feature_delivery"]
        self.issue["issue_type"] = "task"
        self.item["work_purpose"]["kind"] = "maintenance"
        self.assertEqual(self.failures(), [])

    def test_tracker_type_cannot_evade_product_delivery(self):
        del self.item["feature_delivery"]
        for kind in ["task", "epic", "chore", "feature"]:
            with self.subTest(kind=kind):
                self.issue["issue_type"] = kind
                self.assertIn("requires an ordinary-workflow contract", " ".join(self.failures()))

    def test_unselected_product_contract_is_due_before_selection_not_new_work(self):
        del self.item["feature_delivery"]
        self.item["canonical_next"] = False
        self.assertEqual(self.failures(), [])
        self.item["state"] = "in_progress"
        self.assertIn("requires an ordinary-workflow contract", " ".join(self.failures()))

    def test_qualification_does_not_fabricate_product_availability(self):
        del self.item["feature_delivery"]
        self.item["work_purpose"]["kind"] = "qualification"
        self.assertEqual(self.failures(), [])
        self.assertEqual(delivery.delivery_lines(self.item), [])

    def test_empty_workflow_and_safety_are_rejected(self):
        self.contract["ordinary_workflow"] = []
        self.contract["safety_requirements"] = []
        self.assertEqual(len(self.failures()), 2)

    def test_hard_blocker_requires_failure_reason(self):
        self.item["dependencies"] = ["dat-writer"]
        self.assertIn("unexplained hard prerequisite", " ".join(self.failures()))
        self.contract["prerequisite_reasons"]["issue:dat-writer"] = "Concurrent writers can overwrite persisted state."
        self.assertEqual(self.failures(), [])

    def test_planning_needs_concrete_blocker_but_does_not_authorize_execution(self):
        self.item["completion"]["steps"][0]["kind"] = "planning"
        self.assertIn("needs its concrete blocking reason", " ".join(self.failures()))
        self.contract["prerequisite_reasons"]["step:TEST-C01"] = "Writer serialization authority is unresolved."
        self.assertEqual(self.failures(), [])
        self.assertEqual(self.item["completion"]["steps"][0]["kind"], "planning")

    def test_document_or_offline_fixture_cannot_make_build_available(self):
        self.evidence()
        (self.root / "ordinary.json").write_text(json.dumps({"kind": "offline_fixture"}))
        self.assertIn("ordinary application run", " ".join(self.failures()))

    def test_development_ready_needs_all_native_workflow_and_safety_results(self):
        record = self.evidence()
        self.assertEqual(self.failures(), [])
        record["safety_results"]["Cancel writes nothing"] = False
        (self.root / "ordinary.json").write_text(json.dumps(record))
        self.assertIn("passing safety_results", " ".join(self.failures()))

    def test_missing_evidence_and_unresolvable_source_are_rejected(self):
        self.contract["development_ready"] = True
        self.assertIn("needs ordinary application evidence", " ".join(self.failures()))
        record = self.evidence()
        record["source_commit"] = "1" * 40
        (self.root / "ordinary.json").write_text(json.dumps(record))
        self.assertIn("exact source commit", " ".join(self.failures()))

    def test_evidence_cannot_escape_repository(self):
        self.contract.update(development_ready=True, ordinary_evidence=["../outside.json"])
        self.assertIn("repository-relative", " ".join(self.failures()))
        (self.root / "outside.json").symlink_to(self.root.parent / "outside.json")
        self.contract["ordinary_evidence"] = ["outside.json"]
        self.assertIn("symlink", " ".join(self.failures()))

    def test_selector_enforces_policy_and_prints_continuation(self):
        self.fixture.write_fixture()
        errors, _ = status.validate(self.root)
        self.assertEqual(errors, [])
        view = {"feature_delivery": copy.deepcopy(self.contract), "owner_input_required": False}
        self.assertIn("Commits, documentation and lease updates are not delivery", " ".join(delivery.delivery_lines(view)))
        self.item.pop("feature_delivery")
        self.fixture.write_fixture()
        errors, _ = status.validate(self.root)
        self.assertIn("ordinary-workflow contract", " ".join(errors))

    def test_details_show_same_delivery_contract_and_stop_unselected_guidance(self):
        self.fixture.write_fixture()
        next_view = status.next_view(self.item, self.issue)
        details = status.completion_view(self.item, self.issue)
        self.assertEqual(next_view["feature_delivery"], details["feature_delivery"])
        rendered = status.render_completion(details)
        self.assertIn("Move footprint -> Undo and reopen", rendered)
        self.item["canonical_next"] = False
        rendered = status.render_completion(status.completion_view(self.item, self.issue))
        self.assertIn("STOP: unselected", rendered)
        self.assertNotIn("Delivery rule: continue", rendered)

    def test_owner_boundary_never_prints_continue_instruction(self):
        view = {"feature_delivery": self.contract, "owner_input_required": True}
        self.assertNotIn("continue", " ".join(delivery.delivery_lines(view)))


if __name__ == "__main__":
    unittest.main()
