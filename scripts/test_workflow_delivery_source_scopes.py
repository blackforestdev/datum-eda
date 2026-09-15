"""Permission tests; transaction diff capture and entry-point wiring are separate."""

from copy import deepcopy
from datetime import datetime, timezone
from types import SimpleNamespace
import unittest

from workflow_delivery_io import DeliveryInputError
from workflow_delivery_source_scopes import authorize_source_paths
from workflow_delivery_checkpoints import preparation_bootstrap_structure, same_claim_identity


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

    def test_renewed_lease_preserves_scope_permission_and_claim_identity(self):
        approved = deepcopy(self.item["claim"])
        self.item["claim"].update(heartbeat_at="2026-09-08T02:00:00Z",
                                  expires_at="2026-09-08T10:00:00Z")
        self.assertTrue(same_claim_identity(self.item["claim"], approved))
        self.assertEqual([{"path": "src/owned/a.py", "authorized_lanes": ["TASK"]}], self.check())

    def test_renewal_does_not_allow_expired_future_or_overlong_leases(self):
        approved = deepcopy(self.item["claim"])
        for heartbeat, expiry in (
            ("2026-09-08T00:00:00Z", "2026-09-08T01:00:00Z"),
            ("2026-09-08T03:00:00Z", "2026-09-08T04:00:00Z"),
            ("2026-09-08T01:00:00Z", "2026-09-08T10:00:00Z"),
            ("invalid", "2026-09-08T03:00:00Z"),
        ):
            with self.subTest(heartbeat=heartbeat, expiry=expiry):
                self.item["claim"].update(heartbeat_at=heartbeat, expires_at=expiry)
                self.assertTrue(same_claim_identity(self.item["claim"], approved))
                with self.assertRaises(DeliveryInputError):
                    self.check()

    def test_renewal_identity_rejects_every_nonlease_field_change(self):
        approved = deepcopy(self.item["claim"])
        for key in set(approved) - {"heartbeat_at", "expires_at"}:
            with self.subTest(field=key):
                changed = deepcopy(approved)
                changed[key] = ["src/other"] if key == "scope" else "changed"
                self.assertFalse(same_claim_identity(changed, approved))
        for invalid in (None, {}, {**approved, "unknown": True},
                        {k: v for k, v in approved.items() if k != "expires_at"}):
            self.assertFalse(same_claim_identity(invalid, approved))
            self.assertFalse(same_claim_identity(approved, invalid))

    def test_installed_bootstrap_accepts_renewal_but_still_rejects_input_change(self):
        from workflow_delivery_activation_preflight import S5A_PREPARATION_SCOPE

        item = deepcopy(self.item)
        item["key"] = "WORKFLOW-DELIVERY-IMPLEMENTATION"
        item["completion"]["canonical_next_step_id"] = "WDQ-I05"
        approved = deepcopy(item)
        authority = SimpleNamespace(
            json=lambda path: {"frontier": [approved]},
            manifest=lambda roots: {"scripts/gate.py": "unchanged"})
        trust = SimpleNamespace(authority=authority, policy={"coverage": {
            "preparation_scopes": [deepcopy(S5A_PREPARATION_SCOPE)],
            "source_scopes": [{"frontier_key": item["key"], "step_ids": ["WDQ-I05"]}]}})
        tree = SimpleNamespace(manifest=lambda roots: {"scripts/gate.py": "unchanged"})
        contract = {"input_roots": ["scripts/gate.py"]}
        item["claim"].update(heartbeat_at="2026-09-08T02:00:00Z",
                             expires_at="2026-09-08T10:00:00Z")
        self.assertTrue(preparation_bootstrap_structure(tree, item, trust, contract))
        tree.manifest = lambda roots: {"scripts/gate.py": "changed"}
        self.assertFalse(preparation_bootstrap_structure(tree, item, trust, contract))
        tree.manifest = authority.manifest
        item["claim"]["session"] = "different-session"
        self.assertFalse(preparation_bootstrap_structure(tree, item, trust, contract))

    def test_tracker_mismatch_refuses(self):
        self.issues["dat-task"]["assignee"] = "different-session"
        with self.assertRaises(DeliveryInputError):
            self.check()

    def test_lease_repair_promotion_preserves_other_lanes_and_owner_policy(self):
        from workflow_delivery_activation_preflight import installed_lease_repair, LEASE_REPAIR_PATHS

        manifest = deepcopy(self.manifest)
        item = manifest["frontier"][0]
        item["key"] = "WORKFLOW-DELIVERY-IMPLEMENTATION"
        item["completion"]["canonical_next_step_id"] = "WDQ-I05"
        item["claim"]["scope"] = sorted(LEASE_REPAIR_PATHS)
        manifest["frontier"].append({"key": "OTHER", "state": "specified"})
        policy = {"coverage": {"production_roots": ["scripts", "src"],
            "source_scopes": [{"frontier_key": item["key"], "paths": sorted(LEASE_REPAIR_PATHS)}]}}
        baseline, prior = deepcopy(manifest), deepcopy(policy)
        item["claim"].update(heartbeat_at="2026-09-08T02:00:00Z",
                             expires_at="2026-09-08T10:00:00Z")
        changed = sorted(LEASE_REPAIR_PATHS) + ["specs/active_frontier.json"]
        def check():
            return installed_lease_repair(manifest, baseline, policy, prior, changed)
        self.assertEqual(sorted(LEASE_REPAIR_PATHS), check())
        snapshot = deepcopy((manifest, policy, changed))
        mutations = [
            lambda: manifest["frontier"][1].update(state="landed"),
            lambda: manifest["frontier"][0]["claim"].update(session="other"),
            lambda: manifest["frontier"][0]["claim"].update(scope=["scripts"]),
            lambda: manifest["frontier"][0].update(authorization="owner_decision"),
            lambda: manifest["frontier"][0]["completion"].update(canonical_next_step_id="WDQ-I06"),
            lambda: policy.update(unreviewed=True),
            lambda: changed.append("src/product.py"),
            lambda: changed.append("docs/gui/prototypes/preferences-window.html"),
            lambda: changed.append("docs/unrelated.md"),
            lambda: changed.remove(sorted(LEASE_REPAIR_PATHS)[0]),
        ]
        for index, mutate in enumerate(mutations):
            with self.subTest(case=index):
                manifest, policy, changed = deepcopy(snapshot)
                mutate()
                with self.assertRaises(ValueError):
                    check()

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
