"""Pre-publication refusal tests on isolated owned repositories only."""

import os
import json
from pathlib import Path
import subprocess
import sys
from copy import deepcopy
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from workflow_delivery_activation_preflight import (
    preflight, promotion_boundary, initial_mapping_migration,
    preparation_scope_migration, preparation_upgrade_boundary,
    validate_preparation_upgrade_review, PREPARATION_REPLAY_COMMANDS, PREPARATION_REVIEW,
    installed_preparation_recovery, validate_installed_recovery_review,
    RECOVERY_IMPLEMENTATION_PATHS, RECOVERY_REPLAY_COMMANDS, RECOVERY_REVIEW,
    ROLLOUT, S5A, S5A_PREPARATION_SCOPE,
)
from workflow_delivery_capture_state import protected_state
from workflow_delivery_checkpoints import preparation_bootstrap_structure
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
                    for sid in ("WDQ-TOOLS", "WDQ-READY", "WDQ-I03", "WDQ-REVIEW",
                                "WDQ-COMPAT", "WDQ-RECHECK")] + [
                    {"id": "WDQ-I04", "kind": "owner_decision", "status": "pending"}]}}
        steps = self.item["completion"]["steps"]
        for step, dependency in zip(steps[-3:], ("WDQ-REVIEW", "WDQ-COMPAT", "WDQ-RECHECK")):
            step["depends_on"] = [dependency]
        self.item["completion"]["delivery"] = {"schema_version": 2, "checkpoints": {
            "ready": "WDQ-READY", "activate": None, "verify": "WDQ-COMPAT",
            "review": "WDQ-RECHECK", "accept": None}}
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

    def test_renewal_missing_pending_mapping_and_dependencies_refuse_in_both_views(self):
        for view in (self.manifest, self.baseline):
            for sid in ("WDQ-COMPAT", "WDQ-RECHECK"):
                for missing in (False, True):
                    with self.subTest(baseline=view is self.baseline, step=sid, missing=missing):
                        original = deepcopy(view["frontier"][0])
                        completion = view["frontier"][0]["completion"]
                        if missing:
                            completion["steps"] = [s for s in completion["steps"] if s["id"] != sid]
                        else:
                            next(s for s in completion["steps"] if s["id"] == sid)["status"] = "pending"
                        with self.assertRaisesRegex(ValueError, "requires completed " + sid):
                            self.check()
                        view["frontier"][0] = original
            completion = view["frontier"][0]["completion"]
            for step in completion["steps"][-3:]:
                original = step["depends_on"]
                step["depends_on"] = []
                with self.assertRaisesRegex(ValueError, "renewal dependency chain"):
                    self.check()
                step["depends_on"] = original
            for phase, old_step in (("verify", "WDQ-I03"), ("review", "WDQ-REVIEW")):
                points = completion["delivery"]["checkpoints"]
                original = points[phase]
                points[phase] = old_step
                with self.assertRaisesRegex(ValueError, "independent-review mapping"):
                    self.check()
                points[phase] = original

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

    def test_initial_legacy_mapping_migration_is_narrow_and_nonmutating(self):
        self.item["completion"]["delivery"]["contract_path"] = "specs/workflow_delivery/rollout.contract.json"
        del self.baseline["frontier"][0]["completion"]["delivery"]
        ref = {"path": "docs/policy.md", "marker": "policy"}
        pilot = {"frontier_key": "PILOT", "implementation_sessions": ["writer"], "activation_ref": ref}
        prior = {"schema_version": 1, "decision_ref": ref, "legacy_baseline": "1" * 40,
                 "enrolled": [pilot]}
        policy = dict(prior, schema_version=2, coverage=self.coverage,
                      enrolled=[deepcopy(pilot), dict(pilot, frontier_key=ROLLOUT)])
        before = deepcopy((self.manifest, self.baseline, policy, prior))
        self.assertTrue(initial_mapping_migration(self.manifest, self.baseline, policy, prior))
        self.assertEqual(before, (self.manifest, self.baseline, policy, prior))
        with self.assertRaisesRegex(ValueError, "independent-review mapping"):
            self.check()
        self.assertEqual(["scripts/owned.py"], promotion_boundary(self.manifest, self.baseline,
            self.coverage, ["scripts/owned.py"], legacy_migration=True))
        for mutate in (
            lambda m, b, p, o: b["frontier"][0]["completion"].update(delivery=None),
            lambda m, b, p, o: o.update(schema_version=True),
            lambda m, b, p, o: o["enrolled"].append(dict(pilot, frontier_key=ROLLOUT)),
            lambda m, b, p, o: p["enrolled"].pop(0),
            lambda m, b, p, o: p["enrolled"][0].update(implementation_sessions=["other"]),
            lambda m, b, p, o: p.update(legacy_baseline="2" * 40),
            lambda m, b, p, o: m["frontier"][0]["completion"]["delivery"].update(contract_path="other.json"),
        ):
            values = deepcopy(before)
            mutate(*values)
            with self.subTest(mutation=mutate), self.assertRaises(ValueError):
                initial_mapping_migration(*values)
        self.baseline["frontier"][0]["completion"]["steps"][-2]["status"] = "pending"
        with self.assertRaisesRegex(ValueError, "requires completed WDQ-RECHECK"):
            promotion_boundary(self.manifest, self.baseline, self.coverage,
                               ["scripts/owned.py"], legacy_migration=True)


class PreparationMigrationTest(unittest.TestCase):
    def setUp(self):
        ref = {"path": "docs/scope.md", "marker": "scope"}
        claim = {"agent": "codex", "harness": "codex-cli", "session": "repair",
                 "worktree": "/fixture", "head": "a" * 40,
                 "scope": ["scripts/owned.py"], "claimed_at": "2026-09-11T00:00:00Z",
                 "heartbeat_at": "2026-09-11T00:00:00Z",
                 "expires_at": "2026-09-11T08:00:00Z"}
        rollout = {"key": ROLLOUT, "issue_id": "dat-wdq-rollout-implementation-ffy",
                   "state": "in_progress", "authorization": "execution", "claim": claim,
                   "completion": {"canonical_next_step_id": "WDQ-I05", "steps": [
                       {"id": "WDQ-I03", "kind": "execution", "status": "complete"},
                       {"id": "WDQ-REVIEW", "kind": "execution", "status": "complete"},
                       {"id": "WDQ-I05", "kind": "execution", "status": "in_progress"}]}}
        s5a = {"key": S5A, "issue_id": "dat-uvt-s5a-build-1wv", "state": "specified",
               "authorization": "planning", "completion": {
                   "canonical_next_step_id": "S5A-C01", "steps": [
                       {"id": "S5A-C01", "kind": "planning", "status": "pending"}]}}
        self.manifest = {"frontier": [rollout, s5a]}
        self.baseline = deepcopy(self.manifest)
        coverage = {"baseline_ref": "1" * 40, "new_item_rule": "classification_required",
                    "production_roots": ["scripts", "crates"], "rows": [
                        {"frontier_key": ROLLOUT, "issue_id": rollout["issue_id"],
                         "category": "infrastructure", "boundary_ref": ref,
                         "external_handoff_ref": None},
                        {"frontier_key": S5A, "issue_id": s5a["issue_id"],
                         "category": "product", "boundary_ref": ref,
                         "external_handoff_ref": None}],
                    "source_scopes": [{"frontier_key": ROLLOUT,
                        "step_ids": ["WDQ-I03", "WDQ-REVIEW"],
                        "paths": ["scripts/owned.py"], "boundary_ref": ref}]}
        self.prior = {"schema_version": 2, "decision_ref": ref,
                      "legacy_baseline": "1" * 40, "enrolled": [{
                          "frontier_key": "PILOT", "implementation_sessions": ["writer"],
                          "activation_ref": ref}], "coverage": coverage}
        self.policy = deepcopy(self.prior)
        self.policy["coverage"]["source_scopes"][0]["step_ids"].append("WDQ-I05")
        self.policy["coverage"]["preparation_scopes"] = [deepcopy(S5A_PREPARATION_SCOPE)]

    def test_exact_amendment_and_path_boundary_pass_without_mutation(self):
        before = deepcopy((self.manifest, self.baseline, self.policy, self.prior))
        self.assertTrue(preparation_scope_migration(
            self.manifest, self.baseline, self.policy, self.prior))
        self.assertEqual(["scripts/owned.py"], preparation_upgrade_boundary(
            self.manifest, self.baseline, self.policy["coverage"], ["scripts/owned.py"]))
        self.assertEqual(before, (self.manifest, self.baseline, self.policy, self.prior))

    def test_unrelated_policy_scope_or_frontier_change_refuses(self):
        cases = [
            ("legacy", lambda: self.policy.update(legacy_baseline="2" * 40)),
            ("source", lambda: self.policy["coverage"]["source_scopes"][0]["paths"].append("scripts/other.py")),
            ("preparation", lambda: self.policy["coverage"]["preparation_scopes"][0]["paths"].append("crates/other")),
            ("frontier", lambda: self.manifest["frontier"][1].update(state="in_progress")),
        ]
        for label, mutate in cases:
            with self.subTest(mutation=label):
                snapshot = deepcopy((self.manifest, self.policy))
                mutate()
                with self.assertRaises(ValueError):
                    preparation_scope_migration(self.manifest, self.baseline,
                                                self.policy, self.prior)
                self.manifest, self.policy = snapshot

    def test_production_path_outside_policy_or_live_claim_refuses(self):
        with self.assertRaisesRegex(ValueError, "exact reviewed scope"):
            preparation_upgrade_boundary(self.manifest, self.baseline,
                self.policy["coverage"], ["scripts/other.py"])
        self.policy["coverage"]["source_scopes"][0]["paths"].append("scripts/other.py")
        with self.assertRaisesRegex(ValueError, "outside the live I05 claim"):
            preparation_upgrade_boundary(self.manifest, self.baseline,
                self.policy["coverage"], ["scripts/other.py"])

    def test_installed_rollout_stays_structural_only_for_exact_authority_inputs(self):
        class ManifestTree:
            def __init__(self, value):
                self.value = value

            def manifest(self, roots):
                return self.value

        item = self.manifest["frontier"][0]
        contract = {"input_roots": ["scripts/gate.py"]}
        trust = SimpleNamespace(policy=self.policy, authority=ManifestTree({"gate": "exact"}))
        tree = ManifestTree({"gate": "exact"})
        self.assertTrue(preparation_bootstrap_structure(tree, item, trust, contract))
        tree.value = {"gate": "changed"}
        self.assertFalse(preparation_bootstrap_structure(tree, item, trust, contract))
        tree.value = {"gate": "exact"}
        item["completion"]["canonical_next_step_id"] = "WDQ-I06"
        self.assertFalse(preparation_bootstrap_structure(tree, item, trust, contract))


class PreparationReviewTest(unittest.TestCase):
    def setUp(self):
        from workflow_delivery_tree import Tree
        self.f = Fixture()
        self.addCleanup(self.f.close)
        self.base = self.f.head
        self.f.write("scripts/repair.py", b"repair = True\n")
        self.f.stage()
        self.f.git("commit", "-qm", "Synthetic preparation repair producer")
        self.producer = self.f.git("rev-parse", "HEAD").decode().strip()
        checks = []
        review_root = PREPARATION_REVIEW.rsplit("/", 1)[0]
        for index, (key, command) in enumerate(PREPARATION_REPLAY_COMMANDS.items()):
            stdout = self.f.blob(f"{review_root}/{index}.stdout", b"pass\n")
            stderr = self.f.blob(f"{review_root}/{index}.stderr", b"")
            checks.append({"id": key, "command": command, "returncode": 0,
                           "stdout": stdout, "stderr": stderr})
        delta = publication_delta(self.f.root, base=self.base, candidate=self.producer)
        self.review = {"schema_version": 1,
            "kind": "datum.workflow-delivery.preparation-upgrade-review",
            "base_commit": self.base, "producer_commit": self.producer,
            "producer_session": "codex-wdq-i05-preparation-repair-20260911",
            "reviewer_session": "independent-reviewer",
            "independent_of": ["codex-wdq-i05-preparation-repair-20260911"],
            "disposition": "approve", "implementation_paths": delta["touched_paths"],
            "replay_checks": checks, "findings": [], "activation_asserted": False}
        self.f.save(PREPARATION_REVIEW, self.review)
        self.f.stage()
        self.f.git("commit", "-qm", "Synthetic independent preparation review")
        self.candidate = self.f.git("rev-parse", "HEAD").decode().strip()
        self.tree = Tree(self.f.root, revision=self.candidate)

    def test_exact_review_passes_without_mutation(self):
        before = self.f.snapshot()
        result = validate_preparation_upgrade_review(self.tree, base=self.base)
        self.assertEqual("approve", result["disposition"])
        self.assertEqual(before, self.f.snapshot())

    def test_self_review_refuses(self):
        self.review["reviewer_session"] = self.review["producer_session"]
        self.f.save(PREPARATION_REVIEW, self.review)
        self.f.stage()
        self.f.git("commit", "-qm", "Synthetic invalid self review")
        from workflow_delivery_tree import Tree
        with self.assertRaisesRegex(ValueError, "independent"):
            validate_preparation_upgrade_review(
                Tree(self.f.root, revision=self.f.git("rev-parse", "HEAD").decode().strip()),
                base=self.base)

    def test_post_review_source_change_refuses(self):
        self.f.write("scripts/repair.py", b"repair = 'changed after review'\n")
        self.f.stage()
        self.f.git("commit", "-qm", "Synthetic unreviewed descendant source")
        from workflow_delivery_tree import Tree
        with self.assertRaisesRegex(ValueError, "outside the exact independent-review artifacts"):
            validate_preparation_upgrade_review(
                Tree(self.f.root, revision=self.f.git("rev-parse", "HEAD").decode().strip()),
                base=self.base)
class InstalledRecoveryTest(unittest.TestCase):
    def setUp(self):
        self.case = PreparationMigrationTest()
        self.case.setUp()
        self.addCleanup(self.case.doCleanups)
        self.manifest = deepcopy(self.case.baseline)
        self.baseline = deepcopy(self.case.baseline)
        self.policy = deepcopy(self.case.policy)
        self.prior = deepcopy(self.policy)
        claim = self.manifest["frontier"][0]["claim"]
        claim["scope"] = sorted(set(claim["scope"]) | RECOVERY_IMPLEMENTATION_PATHS)
        paths = self.policy["coverage"]["source_scopes"][0]["paths"]
        paths[:] = sorted(set(paths) | RECOVERY_IMPLEMENTATION_PATHS)
        self.prior = deepcopy(self.policy)

    def test_exact_recovery_scope_passes_without_mutation(self):
        before = deepcopy((self.manifest, self.baseline, self.policy, self.prior))
        self.assertEqual(sorted(RECOVERY_IMPLEMENTATION_PATHS),
            installed_preparation_recovery(self.manifest, self.baseline,
                self.policy, self.prior, sorted(RECOVERY_IMPLEMENTATION_PATHS)))
        self.assertEqual(before, (self.manifest, self.baseline, self.policy, self.prior))

    def test_policy_frontier_and_incomplete_source_sets_refuse(self):
        cases = [
            lambda: self.policy.update(legacy_baseline="2" * 40),
            lambda: self.manifest["frontier"][1].update(state="in_progress"),
            lambda: self.manifest["frontier"][0]["claim"]["scope"].append("scripts/other.py"),
        ]
        for mutate in cases:
            values = deepcopy((self.manifest, self.policy))
            mutate()
            with self.assertRaises(ValueError):
                installed_preparation_recovery(self.manifest, self.baseline,
                    self.policy, self.prior, sorted(RECOVERY_IMPLEMENTATION_PATHS))
            self.manifest, self.policy = values
        with self.assertRaisesRegex(ValueError, "complete exact implementation"):
            installed_preparation_recovery(self.manifest, self.baseline,
                self.policy, self.prior, sorted(RECOVERY_IMPLEMENTATION_PATHS)[1:])
class InstalledRecoveryReviewTest(unittest.TestCase):
    def setUp(self):
        from workflow_delivery_tree import Tree
        self.f = Fixture()
        self.addCleanup(self.f.close)
        self.base = self.f.head
        for path in sorted(RECOVERY_IMPLEMENTATION_PATHS):
            self.f.write(path, (path + "\n").encode())
        self.f.stage()
        self.f.git("commit", "-qm", "Synthetic installed recovery producer")
        self.producer = self.f.git("rev-parse", "HEAD").decode().strip()
        root, checks = RECOVERY_REVIEW.rsplit("/", 1)[0], []
        for index, (key, command) in enumerate(RECOVERY_REPLAY_COMMANDS.items()):
            checks.append({"id": key, "command": command, "returncode": 0,
                "stdout": self.f.blob(f"{root}/{index}.stdout", b"pass\n"),
                "stderr": self.f.blob(f"{root}/{index}.stderr", b"")})
        self.review = {"schema_version": 1,
            "kind": "datum.workflow-delivery.installed-recovery-review",
            "base_commit": self.base, "producer_commit": self.producer,
            "producer_session": "codex-wdq-i05-preparation-repair-20260911",
            "reviewer_session": "independent-reviewer",
            "independent_of": ["codex-wdq-i05-preparation-repair-20260911"],
            "disposition": "approve",
            "implementation_paths": publication_delta(
                self.f.root, base=self.base, candidate=self.producer)["touched_paths"],
            "failed_activation_log_sha256": "a" * 64,
            "replay_checks": checks, "findings": [], "activation_asserted": False}
        self.f.save(RECOVERY_REVIEW, self.review)
        self.f.stage()
        self.f.git("commit", "-qm", "Synthetic installed recovery review")
        self.candidate = self.f.git("rev-parse", "HEAD").decode().strip()
        self.tree = Tree(self.f.root, revision=self.candidate)

    def test_exact_review_passes_and_post_review_source_refuses(self):
        self.assertEqual("approve", validate_installed_recovery_review(
            self.tree, base=self.base)["disposition"])
        self.f.write(next(iter(RECOVERY_IMPLEMENTATION_PATHS)), b"changed after review\n")
        self.f.stage()
        self.f.git("commit", "-qm", "Synthetic unreviewed recovery change")
        from workflow_delivery_tree import Tree
        with self.assertRaisesRegex(ValueError, "outside independent-review"):
            validate_installed_recovery_review(Tree(self.f.root, revision=
                self.f.git("rev-parse", "HEAD").decode().strip()), base=self.base)
class PromotionCandidateTest(unittest.TestCase):
    """Real inspector on complete synthetic proof, never actual rollout evidence."""

    def setUp(self):
        from project_status import render_block
        from workflow_delivery_trust_test_support import accepted_fixture
        from workflow_delivery_tree import Tree
        self.f = Fixture()
        self.addCleanup(self.f.close)
        self.f.git("branch", "-M", "main")
        _, _, _, self.f.head = accepted_fixture(self.f)
        manifest = Tree(self.f.root).json("specs/active_frontier.json")
        manifest["frontier"][1].update(state="planned", authorization="planning")
        manifest["frontier"][1]["completion"]["steps"][0]["kind"] = "planning"
        item = manifest["frontier"][0]
        issue_id = "dat-wdq-rollout-implementation-ffy"
        item.update(key=ROLLOUT, issue_id=issue_id, state="specified", authorization="owner_decision")
        del item["landing_commit"]
        completion = item["completion"]
        template = deepcopy(completion["steps"][0])
        owner_input = deepcopy(completion["steps"][-1]["owner_input"])
        owner_input["requests"][0]["source_ref"]["marker"] = "ACCEPT"
        doc = item["governing_docs"][0]
        steps, lines = [], []
        for sid, kind in (("WDQ-TOOLS", "execution"), ("WDQ-READY", "governance"),
                          ("WDQ-I03", "execution"), ("WDQ-REVIEW", "execution"),
                          ("WDQ-COMPAT", "execution"), ("WDQ-RECHECK", "execution"),
                          ("WDQ-I04", "owner_decision")):
            step = deepcopy(template)
            step.update(id=sid, kind=kind, status="pending" if sid == "WDQ-I04" else "complete",
                depends_on=[steps[-1]["id"]] if steps else [],
                requirement_refs=[{"path": doc, "marker": sid}],
                completion_evidence=[] if sid == "WDQ-I04" else [
                    {"kind": "document", "path": doc, "marker": "DONE-" + sid}])
            if sid == "WDQ-I04":
                step["owner_input"] = owner_input
                lines.append(f"<!-- OWNER:{ROLLOUT}:{sid}:ACCEPT -->")
            steps.append(step)
            lines.extend([f"<!-- REQ:{ROLLOUT}:{sid} -->", "REQ-" + sid, "DONE-" + sid])
        completion.update(steps=steps, canonical_next_step_id="WDQ-I04", delivery={
            "schema_version": 2, "contract_path": "contract.json", "checkpoints": {
                "ready": "WDQ-READY", "activate": None, "verify": "WDQ-COMPAT",
                "review": "WDQ-RECHECK", "accept": None}})
        self.f.write(doc, (self.f.root / doc).read_bytes() + ("\n" + "\n".join(lines)).encode())
        self.f.save("specs/active_frontier.json", manifest)
        self.f.write("specs/PROGRESS.md", render_block(manifest).encode())
        issues = [json.loads(line) for line in (self.f.root / ".beads/issues.jsonl").read_bytes().splitlines()]
        issues[0].update(id=issue_id, status="open", acceptance_criteria="\n".join(s["id"] + ": Fixture" for s in steps))
        self.f.write(".beads/issues.jsonl", b"".join(canonical_json(i) for i in issues))
        self.f.contract.update(frontier_key=ROLLOUT, issue_id=issue_id, category="infrastructure")
        self.f.contract["scenarios"][0]["method"] = "infrastructure"
        self.f.save("contract.json", self.f.contract)
        coverage = {"baseline_ref": self.f.head, "new_item_rule": "classification_required",
            "production_roots": ["src", "scripts", "crates"], "rows": [
                {"frontier_key": i["key"], "issue_id": i["issue_id"],
                 "category": "infrastructure" if i["key"] == ROLLOUT else "product",
                 "boundary_ref": self.f.ref, "external_handoff_ref": None} for i in manifest["frontier"]],
            "source_scopes": [{"frontier_key": ROLLOUT, "step_ids": ["WDQ-I03", "WDQ-REVIEW"],
                               "paths": ["src/read.py"], "boundary_ref": self.f.ref}]}
        self.f.save("specs/workflow_delivery_policy.json", {"schema_version": 2,
            "decision_ref": self.f.ref, "legacy_baseline": self.f.head, "coverage": coverage,
            "enrolled": [{"frontier_key": ROLLOUT, "implementation_sessions": ["writer"],
                          "activation_ref": self.f.ref}]})
        self.f.stage()
        self.f.git("commit", "-qm", "Synthetic owner-boundary baseline; not owner approval")
        self.base = self.f.git("rev-parse", "HEAD").decode().strip()
        self.f.write("src/read.py", b"def read(path):\n    # Synthetic publication change.\n    return path.read_bytes()\n")
        self.f.stage()
        self.f.git("commit", "-qm", "Synthetic source under exact workflow scope")
        self.f.head = self.f.git("rev-parse", "HEAD").decode().strip()
        self.make_proof()

    def make_proof(self):
        from workflow_delivery_authority import authority_sha256
        from workflow_delivery_native_test_support import typed_proof
        from workflow_delivery_proof import packet_sha256
        from workflow_delivery_tree import Tree
        proof, _, _, _ = typed_proof(self.f)
        replay, _, _, _ = typed_proof(self.f, "reviewer")
        self.f.save(self.f.contract["proof_path"], proof)
        self.f.save(self.f.contract["review_path"], {"schema_version": 1,
            "packet_sha256": packet_sha256(self.f.contract, proof, authority_sha256(Tree(self.f.root), self.f.contract)),
            "reviewer_session": "reviewer", "independent_of": ["writer", "original"],
            "disposition": "approve", "replay": self.f.blob("docs/reviews/replay.json", canonical_json(replay)),
            "findings": [], "owner_receipt": None})
        self.f.save("requested-environment.json", {"schema_version": 2,
            "kind": "datum.workflow-delivery.environments", "environments": [
                {"frontier_key": ROLLOUT, "environment": proof["environment"]}]})
        self.commit_candidate()

    def commit_candidate(self):
        self.f.stage()
        self.f.git("commit", "-qm", "Synthetic promotion evidence; not actual delivery")
        self.candidate = self.f.git("rev-parse", "HEAD").decode().strip()
        delta = publication_delta(self.f.root, base=self.base, candidate=self.candidate)
        self.review = {"delta_sha256": sha256(canonical_json(delta)), "paths": delta["touched_paths"]}

    def inspect(self):
        from workflow_delivery_activation_preflight import inspect_promotion_candidate
        return inspect_promotion_candidate(self.f.root, base=self.base, candidate=self.candidate,
            authority=self.candidate, environment_path="requested-environment.json", publication_review=self.review)

    def test_complete_candidate_runs_real_delivery_and_review_validation(self):
        before = protected_state(self.f.root, ["src"])
        result = self.inspect()
        self.assertEqual([ROLLOUT + ": review"], result["checks"])
        self.assertEqual(["src/read.py"], result["production_paths"])
        self.assertFalse(result["publication_authorized"])
        self.assertFalse(result["activation_asserted"])
        self.assertEqual(before, protected_state(self.f.root, ["src"]))

    def test_missing_proof_cannot_pass_from_completed_roadmap_labels(self):
        self.f.git("rm", "--", self.f.contract["proof_path"])
        self.commit_candidate()
        before = protected_state(self.f.root, ["src"])
        with self.assertRaisesRegex(ValueError, "required artifact is not in candidate Git tree"):
            self.inspect()
        self.assertEqual(before, protected_state(self.f.root, ["src"]))

    def test_stale_or_self_review_refuses_even_with_matching_candidate_authority(self):
        path = self.f.root / self.f.contract["review_path"]
        original = json.loads(path.read_bytes())
        for field, value, message in (("packet_sha256", "0" * 64, "packet"),
                                       ("reviewer_session", "writer", "independent")):
            review = dict(original, **{field: value})
            self.f.save(self.f.contract["review_path"], review)
            self.commit_candidate()
            with self.subTest(field=field), self.assertRaisesRegex(ValueError, message):
                self.inspect()

    def test_ordinary_candidate_gate_still_refuses_the_owner_step_source_change(self):
        from contextlib import redirect_stdout
        import io
        from check_workflow_delivery import main
        output = io.StringIO()
        with redirect_stdout(output):
            code = main(["--root", str(self.f.root), "--enforce", "--candidate-ref", self.candidate,
                         "--authority-ref", self.candidate, "--base-ref", self.base,
                         "--environment-path", "requested-environment.json"])
        self.assertEqual(1, code)
        self.assertIn("synchronized live execution claim", output.getvalue())

    def test_actual_promotion_inspection_cli_preserves_attached_main_and_trust(self):
        import tempfile
        from workflow_delivery_support_bundle import prepare_bundle
        from workflow_delivery_support_store import support_locations, proposed_local_trust
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        live = Path(temp.name) / "main"
        # Branch operations are limited to this synthetic test repository.
        self.f.git("branch", "-m", "fixture-candidate")
        self.f.git("branch", "main", self.base)
        self.f.git("worktree", "add", "--quiet", str(live), "main")
        prepare_bundle(live, self.candidate)
        locations = support_locations(live, self.candidate)
        before = protected_state(live, ["src"])
        request = {"schema_version": 1, "kind": "datum.workflow-delivery.promotion-inspection-request",
            "base": self.base, "candidate": self.candidate, "authority": self.candidate,
            "input_roots": ["src"], "prior_local_trust": before["local_trust"],
            "publication_review": self.review, "environment_path": "requested-environment.json",
            "proposed_local_trust": proposed_local_trust(live, authority=self.candidate,
                base=self.base, environment_path="requested-environment.json")}
        request_path = Path(temp.name) / "request.json"
        request_path.write_bytes(canonical_json(request))
        request_digest = sha256(request_path.read_bytes())
        response = locations["store"] / "owner-responses/fixture.json"
        response.parent.mkdir(parents=True)
        response.write_bytes(canonical_json({"schema_version": 1,
            "kind": "datum.workflow-delivery.activation-response",
            "response": "WORKFLOW-DELIVERY-IMPLEMENTATION: approve ACTIVATE — " + request_digest,
            "source": "Synthetic input, not actual owner approval", "recorded_at": "2026-09-08"}))
        output = locations["logs"] / "promotion-fixture.json"
        result = subprocess.run([sys.executable, "-I", "-S", "-B",
            str(Path(__file__).with_name("workflow_delivery_preflight_cli.py")), "--inspect-promotion",
            "--root", str(live), "--request", str(request_path), "--request-sha256", request_digest,
            "--owner-response", str(response), "--owner-response-sha256", sha256(response.read_bytes()),
            "--output", str(output)], capture_output=True, timeout=30)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        payload = json.loads(output.read_bytes())
        self.assertTrue(payload["ok"])
        self.assertEqual([ROLLOUT + ": review"], payload["promotion_inspection"]["checks"])
        self.assertFalse(payload["publication_authorized"])
        self.assertFalse(payload["activation_asserted"])
        self.assertEqual(before, protected_state(live, ["src"]))


if __name__ == "__main__":
    unittest.main()
