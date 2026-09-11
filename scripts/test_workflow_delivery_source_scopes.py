"""Permission tests; transaction diff capture and entry-point wiring are separate."""

from datetime import datetime, timezone
import unittest

from workflow_delivery_io import DeliveryInputError
from workflow_delivery_source_scopes import authorize_source_paths


class SourceScopesTest(unittest.TestCase):
    def setUp(self):
        ref = {"path": "docs/authority.md", "marker": "scope"}
        self.coverage = {"baseline_ref": "a" * 40, "production_roots": ["src"],
                         "new_item_rule": "classification_required",
                         "rows": [{"frontier_key": "TASK", "issue_id": "dat-task",
                                   "category": "product", "boundary_ref": ref,
                                   "external_handoff_ref": None}],
                         "source_scopes": [{"frontier_key": "TASK", "step_ids": ["I01"],
                                            "paths": ["src/owned"], "boundary_ref": ref}]}
        self.item = {"key": "TASK", "issue_id": "dat-task", "state": "in_progress",
                     "authorization": "execution", "claim": {
                         "agent": "codex", "harness": "codex-cli", "session": "test",
                         "worktree": "fixture", "head": "a" * 40, "scope": ["src/owned"],
                         "claimed_at": "2026-09-08T01:00:00Z",
                         "heartbeat_at": "2026-09-08T01:00:00Z",
                         "expires_at": "2026-09-08T09:00:00Z"},
                     "completion": {"canonical_next_step_id": "I01", "delivery": {},
                                    "steps": [{"id": "I01", "kind": "execution",
                                               "status": "in_progress"}]}}
        self.manifest = {"frontier": [self.item], "claim_ttl_hours": 8}
        self.issues = {"dat-task": {"status": "in_progress", "assignee": "codex"}}
        self.enrolled = {"TASK": {}}
        self.now = datetime(2026, 9, 8, 2, tzinfo=timezone.utc)

    def check(self, paths=("src/owned/a.py",)):
        return authorize_source_paths(paths, self.coverage, self.manifest,
                                      self.issues, self.enrolled, now=self.now)

    def test_exact_active_scope_allows_and_unrelated_docs_need_no_permission(self):
        self.assertEqual([{"path": "src/owned/a.py", "authorized_lanes": ["TASK"]}], self.check())
        self.assertEqual([], self.check(["docs/note.md"]))

    def test_promoted_preparation_scope_allows_only_planning_claim(self):
        ref = self.coverage["rows"][0]["boundary_ref"]
        self.coverage["preparation_scopes"] = [{"frontier_key": "TASK",
            "step_ids": ["I01"], "paths": ["src/owned"],
            "boundary_ref": ref, "approval_ref": ref}]
        self.coverage["source_scopes"] = []
        self.item["authorization"] = "planning"
        self.item["completion"].pop("delivery")
        self.item["completion"]["steps"][0]["kind"] = "planning"
        self.enrolled.clear()
        self.assertEqual([{"path": "src/owned/a.py", "authorized_lanes": ["TASK"]}], self.check())

    def test_preparation_scope_refuses_execution_or_missing_approval_shape(self):
        ref = self.coverage["rows"][0]["boundary_ref"]
        scope = {"frontier_key": "TASK", "step_ids": ["I01"],
                 "paths": ["src/owned"], "boundary_ref": ref, "approval_ref": ref}
        self.coverage.update(source_scopes=[], preparation_scopes=[scope])
        self.item["authorization"] = "planning"
        self.item["completion"]["steps"][0]["kind"] = "planning"
        self.enrolled.clear()
        self.item["authorization"] = "execution"
        with self.assertRaises(DeliveryInputError):
            self.check()
        self.item["authorization"] = "planning"
        del scope["approval_ref"]
        with self.assertRaises(DeliveryInputError):
            self.check()

    def test_promoted_scope_cannot_exceed_frontier_claim_scope(self):
        self.item["claim"]["scope"] = ["src/other"]
        with self.assertRaisesRegex(DeliveryInputError, "no promoted scope"):
            self.check()

    def test_prefix_lookalike_is_not_owned(self):
        with self.assertRaises(DeliveryInputError):
            self.check(["src/owned_elsewhere/a.py"])

    def test_missing_or_expired_claim_refuses(self):
        self.item["claim"]["expires_at"] = "2026-09-08T01:30:00Z"
        with self.assertRaises(DeliveryInputError):
            self.check()
        del self.item["claim"]
        with self.assertRaises(DeliveryInputError):
            self.check()

    def test_tracker_mismatch_refuses(self):
        self.issues["dat-task"]["assignee"] = "different-session"
        with self.assertRaises(DeliveryInputError):
            self.check()

    def test_pending_label_and_planning_authorization_refuse(self):
        self.item["completion"]["steps"][0]["status"] = "pending"
        with self.assertRaises(DeliveryInputError):
            self.check()
        self.item["completion"]["steps"][0]["status"] = "in_progress"
        self.item["authorization"] = "planning"
        with self.assertRaises(DeliveryInputError):
            self.check()

    def test_unenrolled_product_or_missing_declaration_refuses(self):
        self.enrolled.clear()
        with self.assertRaises(DeliveryInputError):
            self.check()
        self.enrolled["TASK"] = {}
        del self.item["completion"]["delivery"]
        with self.assertRaises(DeliveryInputError):
            self.check()

    def test_external_handoff_does_not_grant_paths_outside_scope(self):
        row = self.coverage["rows"][0]
        row.update(category="external_lane", external_handoff_ref=row["boundary_ref"])
        self.enrolled.clear()
        self.check()
        with self.assertRaises(DeliveryInputError):
            self.check(["src/somebody_else.py"])


if __name__ == "__main__":
    unittest.main()
