"""Delivery admission regressions use real Git index snapshots, not worktree proof."""
from __future__ import annotations

import copy
import json
from pathlib import Path
import shlex
import subprocess
import tempfile
import unittest

import check_project_delivery_transitions as staged
from project_delivery_gaps import (gap_view, gap_lines, transition_errors,
                                   validate_delivery_gaps)


class DeliveryGapTests(unittest.TestCase):
    def setUp(self):
        self.gap = {"id": "fix", "title": "Correct emitted geometry", "status": "open",
                    "labels": ["delivery:required"], "acceptance_criteria": "- [ ] Exact output regression"}
        self.parent = {"id": "phase", "status": "open", "labels": [],
                       "dependencies": [{"type": "blocks", "depends_on_id": "fix"}]}
        self.issues = {"fix": self.gap, "phase": self.parent}
        self.item = {"key": "PHASE", "issue_id": "phase", "state": "specified",
                     "dependencies": ["fix"], "canonical_next": False}
        self.manifest = {"frontier": [self.item]}

    def test_open_gap_has_enforced_future_home_and_visible_details(self):
        self.assertEqual([], validate_delivery_gaps(self.manifest, self.issues))
        self.assertEqual("fix", gap_view(self.item, self.issues)[0]["issue_id"])
        self.assertIn("hard prerequisites", " ".join(gap_lines({"delivery_gaps": gap_view(self.item, self.issues)})))

    def test_related_link_or_old_landed_phase_cannot_hide_required_gap(self):
        self.item["dependencies"] = []
        self.parent["dependencies"][0]["type"] = "related"
        self.assertIn("not a hard prerequisite", " ".join(validate_delivery_gaps(self.manifest, self.issues)))
        self.item.update(dependencies=["fix"], state="landed")
        self.assertIn("nonterminal", " ".join(validate_delivery_gaps(self.manifest, self.issues)))

    def test_deferred_reentry_preserves_historical_steps_but_cannot_readmit(self):
        self.item.update(state="deferred", completion={"steps": [{"status": "complete"}]})
        self.assertEqual([], validate_delivery_gaps(self.manifest, self.issues))
        before = copy.deepcopy(self.manifest)
        self.item["state"] = "landed"
        self.assertIn("unresolved", " ".join(transition_errors(before, self.manifest, self.issues, self.issues)))

    def test_availability_cannot_bypass_open_prerequisites(self):
        self.item["feature_delivery"] = {"development_ready": True}
        self.assertIn("availability", " ".join(validate_delivery_gaps(self.manifest, self.issues)))

    def test_landed_claim_fails_even_when_another_phase_still_tracks_the_gap(self):
        self.manifest["frontier"].append({"key": "LATER", "issue_id": "later",
                                          "state": "specified", "dependencies": ["fix"]})
        self.item["state"] = "landed"
        self.assertIn("landed delivery", " ".join(validate_delivery_gaps(self.manifest, self.issues)))

    def test_required_closure_needs_checked_criteria_and_evidence(self):
        self.gap["status"] = "closed"
        self.assertIn("unchecked", " ".join(validate_delivery_gaps(self.manifest, self.issues)))
        self.gap["acceptance_criteria"] = "- [x] Exact output regression"
        self.assertIn("close_reason", " ".join(validate_delivery_gaps(self.manifest, self.issues)))
        self.gap["close_reason"] = "Repair and reviewed regression landed in candidate."
        self.assertEqual([], validate_delivery_gaps(self.manifest, self.issues))

    def test_same_change_edge_removal_does_not_authorize_closure(self):
        before = copy.deepcopy(self.manifest)
        old = copy.deepcopy(self.issues)
        self.item.update(dependencies=[], state="landed")
        self.parent.update(dependencies=[], status="closed")
        failures = transition_errors(before, self.manifest, old, self.issues)
        self.assertTrue(any("phase: new closure" in x for x in failures))
        self.assertTrue(any("PHASE: new delivery admission" in x for x in failures))

    def test_unmarking_or_deleting_open_correction_is_rejected(self):
        old = copy.deepcopy(self.issues)
        self.gap["labels"] = []
        self.assertIn("unmarked", " ".join(transition_errors(self.manifest, self.manifest, old, self.issues)))
        del self.issues["fix"]
        self.assertIn("deleted", " ".join(transition_errors(self.manifest, self.manifest, old, self.issues)))

    def test_required_prerequisite_cannot_be_detached_one_commit_before_landing(self):
        before = copy.deepcopy(self.manifest)
        self.item["dependencies"] = []
        self.assertIn("cannot be detached", " ".join(transition_errors(before, self.manifest, self.issues, self.issues)))

    def test_checklist_cannot_be_replaced_with_easier_work_before_closure(self):
        old = copy.deepcopy(self.issues)
        self.gap["acceptance_criteria"] = "- [x] Administrative record complete"
        self.assertIn("cannot be replaced", " ".join(transition_errors(self.manifest, self.manifest, old, self.issues)))
        self.gap.update(status="closed", close_reason="Done")
        self.assertIn("cannot be replaced", " ".join(transition_errors(self.manifest, self.manifest, old, self.issues)))

    def test_canceled_blocker_is_not_satisfied_by_tracker_closed(self):
        old = copy.deepcopy(self.manifest)
        self.manifest["frontier"].append({"issue_id": "fix", "state": "canceled"})
        self.item["state"] = "landed"
        self.gap["status"] = "closed"
        self.assertIn("unresolved", " ".join(transition_errors(old, self.manifest, self.issues, self.issues)))

    def test_historical_closed_task_and_independent_routine_fix_are_not_reopened(self):
        self.issues["historical"] = {"id": "historical", "status": "closed", "dependencies": self.parent["dependencies"]}
        self.issues["routine"] = {"id": "routine", "status": "closed", "dependencies": []}
        self.assertEqual([], transition_errors(self.manifest, self.manifest, self.issues, self.issues))


class StagedDeliveryTests(DeliveryGapTests):
    def setUp(self):
        super().setUp()
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / "specs").mkdir()
        (self.root / ".beads").mkdir()
        self.git("init", "-q")
        self.git("config", "user.name", "Test")
        self.git("config", "user.email", "test@example.com")
        self.git("config", "core.hooksPath", "/dev/null")
        self.write()
        self.git("add", "specs/active_frontier.json", ".beads/issues.jsonl")
        self.git("commit", "-qm", "fixture")

    def git(self, *args):
        return subprocess.check_output(["git", *args], cwd=self.root, text=True)

    def write(self):
        (self.root / staged.FRONTIER).write_text(json.dumps(self.manifest))
        (self.root / staged.TRACKER).write_text("".join(json.dumps(x) + "\n" for x in self.issues.values()))

    def stage(self):
        self.write()
        self.git("add", staged.FRONTIER, staged.TRACKER)

    def test_unstaged_resolution_cannot_bless_staged_bad_closure(self):
        self.item["state"] = "landed"
        self.parent["status"] = "closed"
        self.stage()
        self.gap.update(status="closed", acceptance_criteria="- [x] Exact output regression", close_reason="Reviewed repair landed.")
        self.write()
        self.assertIn("unresolved", " ".join(staged.check(self.root, "HEAD", ":")))

    def test_resolved_staged_correction_allows_dependent_landing(self):
        self.gap.update(status="closed", acceptance_criteria="- [x] Exact output regression", close_reason="Reviewed repair landed.")
        self.item["state"] = "landed"
        self.parent["status"] = "closed"
        self.stage()
        self.assertEqual([], staged.check(self.root, "HEAD", ":"))

    def test_forced_tracker_close_with_unchecked_criteria_is_refused(self):
        self.gap.update(status="closed", close_reason="Forced close")
        self.stage()
        self.assertIn("unchecked", " ".join(staged.check(self.root, "HEAD", ":")))

    def test_staged_criterion_rewrite_is_refused(self):
        self.gap.update(status="closed", acceptance_criteria="- [x] Administrative record complete", close_reason="Done")
        self.stage()
        self.assertIn("cannot be replaced", " ".join(staged.check(self.root, "HEAD", ":")))

    def test_staged_dependency_mismatch_is_refused(self):
        self.parent["dependencies"] = []
        self.stage()
        self.assertIn("mismatch", " ".join(staged.check(self.root, "HEAD", ":")))

    def ready_evidence(self):
        self.gap.update(status="closed", acceptance_criteria="- [x] Exact output regression",
                        close_reason="Reviewed repair landed.")
        self.item["feature_delivery"] = {
            "ordinary_workflow": ["Assign footprint"],
            "safety_requirements": ["Undo restores design"],
            "qualification_followups": [],
            "prerequisite_reasons": {"issue:fix": "Correct emitted geometry"},
            "development_ready": True,
            "ordinary_evidence": ["ordinary.json"],
        }
        return {"kind": "ordinary_application", "source_commit": self.git("rev-parse", "HEAD").strip(),
                "launch_command": "datum-gui --project-root owned-copy",
                "workflow_results": {"Assign footprint": True},
                "safety_results": {"Undo restores design": True}}

    def stage_evidence(self, record):
        (self.root / "ordinary.json").write_text(json.dumps(record))
        self.git("add", "ordinary.json")
        self.stage()

    def test_ready_requires_evidence_in_candidate(self):
        record = self.ready_evidence()
        self.stage()
        # Even a passing worktree-only file is not evidence for this commit.
        (self.root / "ordinary.json").write_text(json.dumps(record))
        self.assertIn("missing or invalid", " ".join(staged.check(self.root, "HEAD", ":")))

    def test_staged_failed_or_missing_results_cannot_be_blessed_by_worktree(self):
        record = self.ready_evidence()
        for field in ["workflow_results", "safety_results"]:
            for results in [{key: False for key in record[field]}, {}]:
                with self.subTest(field=field, results=results):
                    failed = copy.deepcopy(record)
                    failed[field] = results
                    self.stage_evidence(failed)
                    (self.root / "ordinary.json").write_text(json.dumps(record))
                    self.assertIn("passing " + field, " ".join(staged.check(self.root, "HEAD", ":")))

    def test_valid_staged_and_committed_evidence_ignore_worktree_failure(self):
        self.stage_evidence(self.ready_evidence())
        (self.root / "ordinary.json").write_text("invalid worktree JSON")
        self.assertEqual([], staged.check(self.root, "HEAD", ":"))
        self.git("commit", "-qm", "ready candidate")
        self.assertEqual([], staged.check(self.root, "HEAD^", "HEAD"))

    def test_evidence_only_change_runs_staged_gate(self):
        record = self.ready_evidence()
        self.stage_evidence(record)
        self.git("commit", "-qm", "ready candidate")
        record["workflow_results"]["Assign footprint"] = False
        (self.root / "ordinary.json").write_text(json.dumps(record))
        self.git("add", "ordinary.json")
        self.assertEqual(1, staged.main(["--root", str(self.root), "--staged"]))

    def test_staged_evidence_symlink_cannot_read_worktree_target(self):
        record = self.ready_evidence()
        (self.root / "passing.json").write_text(json.dumps(record))
        (self.root / "ordinary.json").symlink_to("passing.json")
        self.git("add", "ordinary.json")
        self.stage()
        self.assertIn("missing or invalid", " ".join(staged.check(self.root, "HEAD", ":")))

    def test_evidence_only_trigger_handles_relative_alias_and_quoted_names(self):
        for path in ["./ordinary.json", "ordinary-é.json", "ordinary\tresult.json"]:
            with self.subTest(path=path):
                record = self.ready_evidence()
                self.item["feature_delivery"]["ordinary_evidence"] = [path]
                target = self.root / path
                target.write_text(json.dumps(record))
                self.git("add", path)
                self.stage()
                self.assertEqual([], staged.check(self.root, "HEAD", ":"))
                self.git("commit", "-qm", "ready candidate")
                record["workflow_results"]["Assign footprint"] = False
                target.write_text(json.dumps(record))
                self.git("add", path)
                self.assertEqual(1, staged.main(["--root", str(self.root), "--staged"]))
                self.git("restore", "--source=HEAD", "--staged", "--worktree", "--", path)

    def install_delivery_hook(self):
        hook_dir = self.root / "hooks"
        hook_dir.mkdir()
        hook = hook_dir / "pre-commit"
        tool = Path(staged.__file__).resolve()
        hook.write_text("#!/bin/sh\nexec python3 " + shlex.quote(str(tool)) + " --root "
                        + shlex.quote(str(self.root)) + " --staged\n")
        hook.chmod(0o755)
        self.git("config", "core.hooksPath", str(hook_dir))

    def test_real_precommit_rejects_ready_without_evidence(self):
        self.install_delivery_hook()
        before = self.git("rev-parse", "HEAD")
        self.ready_evidence()
        self.stage()
        result = subprocess.run(["git", "commit", "-qm", "unsupported readiness"],
                                cwd=self.root, text=True, capture_output=True)
        self.assertNotEqual(0, result.returncode)
        self.assertIn("ordinary evidence is missing", result.stdout + result.stderr)
        self.assertEqual(before, self.git("rev-parse", "HEAD"))

    def test_real_precommit_rejects_bad_closure_and_preserves_head(self):
        self.install_delivery_hook()
        before = self.git("rev-parse", "HEAD")
        self.gap.update(status="closed", close_reason="Forced premature close")
        self.stage()
        result = subprocess.run(["git", "commit", "-qm", "bad closure"], cwd=self.root,
                                text=True, capture_output=True)
        self.assertNotEqual(0, result.returncode)
        self.assertIn("unchecked", result.stdout + result.stderr)
        self.assertEqual(before, self.git("rev-parse", "HEAD"))


if __name__ == "__main__":
    unittest.main()
