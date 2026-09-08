"""Pre-publication refusal tests on isolated owned repositories only."""

import os
from copy import deepcopy
import unittest
from unittest.mock import patch

from workflow_delivery_activation_preflight import preflight, promotion_boundary, ROLLOUT
from workflow_delivery_capture_state import protected_state
from workflow_delivery_publication_delta import publication_delta
from workflow_delivery_io import canonical_json, sha256
from workflow_delivery_test_support import Fixture


class ActivationPreflightTest(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.close)
        self.f.git("branch", "-M", "main")
        self.base = self.f.head
        tree = self.f.git("rev-parse", "HEAD^{tree}").decode().strip()
        self.candidate = self.f.git("commit-tree", tree, "-p", self.base,
                                   "-m", "synthetic candidate").decode().strip()
        self.trust = protected_state(self.f.root, ["src"])["local_trust"]
        delta = publication_delta(self.f.root, base=self.base, candidate=self.candidate)
        self.review = {"delta_sha256": sha256(canonical_json(delta)), "paths": delta["touched_paths"]}

    def run_preflight(self, **overrides):
        options = dict(base=self.base, candidate=self.candidate, input_roots=["src"],
                       expected_local_trust=self.trust, publication_review=self.review)
        options.update(overrides)
        return preflight(self.f.root, **options)

    def test_clean_preparation_is_nonmutating_and_not_permission(self):
        before = self.f.snapshot()
        result = self.run_preflight()
        self.assertFalse(result["activation_asserted"])
        self.assertFalse(result["publication_authorized"])
        self.assertEqual(before, self.f.snapshot())

    def test_stale_base_and_prior_trust_refuse(self):
        with self.assertRaisesRegex(ValueError, "HEAD differs"):
            self.run_preflight(base=self.candidate)
        self.f.git("config", "--local", "datum.workflowDeliveryAuthorityRef", self.base)
        with self.assertRaisesRegex(ValueError, "local trust differs"):
            self.run_preflight()

    def test_unreviewed_candidate_history_refuses_without_publication(self):
        tree = self.f.git("rev-parse", "HEAD^{tree}").decode().strip()
        later = self.f.git("commit-tree", tree, "-p", self.candidate,
                          "-m", "additional unreviewed history").decode().strip()
        before = protected_state(self.f.root, ["src"])
        with self.assertRaisesRegex(ValueError, "review digest differs"):
            self.run_preflight(candidate=later)
        self.assertEqual(before, protected_state(self.f.root, ["src"]))

    def test_detached_checkout_refuses_without_switching_branch(self):
        self.f.git("checkout", "--detach", "-q", self.base)
        before = protected_state(self.f.root, ["src"])
        with self.assertRaisesRegex(ValueError, "attached main checkout"):
            self.run_preflight()
        self.assertEqual(before, protected_state(self.f.root, ["src"]))

    def test_other_branch_refuses_even_with_exact_base_and_clean_files(self):
        self.f.git("branch", "-m", "fixture-other")
        before = protected_state(self.f.root, ["src"])
        with self.assertRaisesRegex(ValueError, "attached main checkout"):
            self.run_preflight()
        self.assertEqual(before, protected_state(self.f.root, ["src"]))

    def test_dirty_tracked_index_and_worktree_refuse(self):
        self.f.write("src/read.py", b"uncommitted\n")
        with self.assertRaisesRegex(ValueError, "dirty"):
            self.run_preflight()
        self.f.stage()
        with self.assertRaisesRegex(ValueError, "dirty"):
            self.run_preflight()

    def test_untracked_file_outside_reviewed_roots_still_refuses(self):
        self.f.write("unexpected-note.txt", b"untracked\n")
        with self.assertRaisesRegex(ValueError, "dirty"):
            self.run_preflight()

    def test_filemode_setting_cannot_hide_executable_change(self):
        self.f.git("config", "core.filemode", "false")
        (self.f.root / "src/read.py").chmod(0o755)
        with self.assertRaisesRegex(ValueError, "executable mode changed"):
            self.run_preflight()

    def test_assume_unchanged_cannot_hide_tracked_byte_changes(self):
        self.f.git("update-index", "--assume-unchanged", "src/read.py")
        self.f.write("src/read.py", b"hidden change\n")
        with self.assertRaisesRegex(ValueError, "tracked bytes differ"):
            self.run_preflight()

    def test_ignored_source_input_refuses(self):
        self.f.write(".git/info/exclude", b"src/ignored.py\n")
        self.f.write("src/ignored.py", b"ignored build input\n")
        with self.assertRaisesRegex(ValueError, "ignored or missing"):
            self.run_preflight()

    def test_missing_reviewed_input_and_alternate_index_refuse(self):
        with self.assertRaisesRegex(ValueError, "missing reviewed input"):
            self.run_preflight(input_roots=["src", "absent-input"])
        with patch.dict(os.environ, {"GIT_INDEX_FILE": "/nonexistent-alternate-index"}):
            with self.assertRaisesRegex(ValueError, "overrides"):
                self.run_preflight()

    def test_changes_during_preparation_are_not_silently_accepted(self):
        def changed(*args, **kwargs):
            result = publication_delta(*args, **kwargs)
            self.f.write("src/read.py", b"concurrent input\n")
            return result
        with patch("workflow_delivery_activation_preflight.publication_delta", side_effect=changed):
            with self.assertRaisesRegex(ValueError, "state changed"):
                self.run_preflight()


class PromotionBoundaryTest(unittest.TestCase):
    """Synthetic boundary inputs only; passing does not establish real review."""

    def setUp(self):
        self.item = {"key": ROLLOUT, "issue_id": "dat-wdq-rollout-implementation-ffy",
            "authorization": "owner_decision", "claim": None, "completion": {
                "canonical_next_step_id": "WDQ-I04", "steps": [
                    {"id": sid, "kind": "execution", "status": "complete"}
                    for sid in ("WDQ-TOOLS", "WDQ-READY", "WDQ-I03", "WDQ-REVIEW")] + [
                    {"id": "WDQ-I04", "kind": "owner_decision", "status": "pending"}]}}
        self.manifest = {"frontier": [self.item, {"key": "EXTERNAL", "claim": "preserved"}]}
        self.baseline = deepcopy(self.manifest)
        ref = {"path": "docs/scope.md", "marker": "scope"}
        self.coverage = {"baseline_ref": "1" * 40, "new_item_rule": "classification_required",
            "production_roots": ["scripts", "crates"], "rows": [
                {"frontier_key": ROLLOUT, "issue_id": self.item["issue_id"],
                 "category": "infrastructure", "boundary_ref": ref, "external_handoff_ref": None}],
            "source_scopes": [{"frontier_key": ROLLOUT, "step_ids": ["WDQ-I03", "WDQ-REVIEW"],
                               "paths": ["scripts/owned.py"], "boundary_ref": ref}]}

    def check(self, paths=None):
        return promotion_boundary(self.manifest, self.baseline, self.coverage,
                                  paths or ["scripts/owned.py"])

    def test_complete_boundary_has_exact_paths_without_synthetic_execution_claim(self):
        before = deepcopy(self.manifest)
        self.assertEqual(["scripts/owned.py"], self.check())
        self.assertEqual(before, self.manifest)

    def test_each_unfinished_predecessor_refuses(self):
        for sid in ("WDQ-TOOLS", "WDQ-READY", "WDQ-I03", "WDQ-REVIEW"):
            with self.subTest(step=sid):
                step = next(s for s in self.item["completion"]["steps"] if s["id"] == sid)
                step["status"] = "pending"
                with self.assertRaisesRegex(ValueError, "requires completed " + sid):
                    self.check()
                step["status"] = "complete"

    def test_execution_claim_or_wrong_owner_selection_refuses_in_either_view(self):
        for manifest in (self.manifest, self.baseline):
            item = manifest["frontier"][0]
            for field, value in (("claim", {"session": "writer"}), ("authorization", "execution")):
                with self.subTest(view=manifest is self.baseline, field=field):
                    original = item[field]
                    item[field] = value
                    with self.assertRaisesRegex(ValueError, "claim-free WDQ-I04"):
                        self.check()
                    item[field] = original

    def test_owner_completion_cannot_be_manufactured_before_publication(self):
        self.item["completion"]["steps"][-1]["status"] = "complete"
        with self.assertRaisesRegex(ValueError, "await exact owner activation"):
            self.check()

    def test_other_lane_changes_and_identity_removal_refuse(self):
        self.manifest["frontier"][1]["claim"] = None
        with self.assertRaisesRegex(ValueError, "another Frontier lane"):
            self.check()
        self.manifest["frontier"].pop()
        with self.assertRaisesRegex(ValueError, "add or remove"):
            self.check()

    def test_source_prefixes_and_product_or_prototype_paths_refuse(self):
        for path in ("scripts/owned.py/child", "scripts/unreviewed.py", "crates/preferences.rs",
                     "docs/gui/prototypes/preferences-window.html"):
            with self.subTest(path=path), self.assertRaises(ValueError):
                self.check([path])

    def test_extra_lane_scope_and_owner_step_scope_refuse(self):
        self.coverage["source_scopes"].append(deepcopy(self.coverage["source_scopes"][0]))
        with self.assertRaises(ValueError):
            self.check()
        self.coverage["source_scopes"].pop()
        self.coverage["source_scopes"][0]["step_ids"] = ["WDQ-I04"]
        with self.assertRaisesRegex(ValueError, "sole bounded"):
            self.check()


if __name__ == "__main__":
    unittest.main()
