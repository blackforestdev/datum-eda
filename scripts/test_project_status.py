#!/usr/bin/env python3
"""Hermetic regressions for project_status.py."""

from __future__ import annotations

import copy
import importlib.util
import io
import json
import subprocess
import tempfile
import unittest
from contextlib import redirect_stdout
from datetime import datetime, timezone
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("project_status.py")
SPEC = importlib.util.spec_from_file_location("project_status", MODULE_PATH)
assert SPEC and SPEC.loader
status = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(status)


class ProjectStatusTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        (self.root / "specs").mkdir()
        (self.root / ".beads").mkdir()
        (self.root / "docs/decisions").mkdir(parents=True)
        self.doc = "docs/decisions/PRODUCT_MECHANICS_025_PROJECT_STATE_AUTHORITY.md"
        (self.root / self.doc).write_text(
            "# Decision\n\n<!-- REQ:alignment:TEST-C01 -->\nRequirement.\n",
            encoding="utf-8",
        )
        self.governance = {
            "entries": {self.doc: {"class": "doctrine", "controlling": True}}
        }
        self.issues = [
            self.issue("dat-next", "open", labels=["roadmap:frontier"]),
            self.issue("dat-intake", "open", labels=["roadmap:intake"]),
        ]
        self.manifest = {
            "schema_version": 2,
            "policy_decision": self.doc,
            "claim_ttl_hours": 24,
            "frontier": [self.item()],
        }
        self.write_fixture()
        subprocess.run(["git", "init", "-q"], cwd=self.root, check=True)
        subprocess.run(["git", "config", "user.email", "test@example.com"], cwd=self.root, check=True)
        subprocess.run(["git", "config", "user.name", "Test"], cwd=self.root, check=True)
        subprocess.run(["git", "add", "."], cwd=self.root, check=True)
        subprocess.run(["git", "commit", "-qm", "fixture"], cwd=self.root, check=True)

    def tearDown(self) -> None:
        self.temp.cleanup()

    @staticmethod
    def issue(
        issue_id: str, state: str, *, assignee: str | None = None,
        labels: list[str] | None = None, dependencies: list[dict] | None = None,
    ) -> dict:
        value = {
            "id": issue_id,
            "title": issue_id,
            "status": state,
            "priority": 1,
            "issue_type": "task",
            "labels": labels or [],
            "dependencies": dependencies or [],
            "acceptance_criteria": "TEST-C01: Complete the fixture requirement.",
        }
        if assignee is not None:
            value["assignee"] = assignee
        return value

    def item(self, **updates: object) -> dict:
        value = {
            "key": "alignment",
            "order": 0,
            "title": "Align project state",
            "issue_id": "dat-next",
            "related_issue_ids": [],
            "governing_docs": [self.doc],
            "state": "ready",
            "authorization": "execution",
            "canonical_next": True,
            "parallel": False,
            "summary": "Make every agent return the same next task.",
            "dependencies": [],
            "unblocks": [],
            "completion": {
                "outcome": "The aligned fixture is complete.",
                "steps": [{
                    "id": "TEST-C01",
                    "kind": "execution",
                    "status": "pending",
                    "action": "Complete the fixture requirement.",
                    "depends_on": [],
                    "requirement_refs": [{"path": self.doc, "marker": "TEST-C01"}],
                    "completion_evidence": [],
                }],
                "post_completion": {
                    "effects": ["The dependent feature becomes unblocked."],
                    "unblocks_issue_ids": [],
                    "selection": "explicit_frontier_update",
                    "authorizes_successor": False,
                    "selects_successor": False,
                },
            },
        }
        value.update(updates)
        return value

    def write_fixture(self, render: bool = True) -> None:
        (self.root / "specs/spec_governance_manifest.json").write_text(
            json.dumps(self.governance), encoding="utf-8"
        )
        (self.root / ".beads/issues.jsonl").write_text(
            "".join(json.dumps(issue) + "\n" for issue in self.issues), encoding="utf-8"
        )
        (self.root / "specs/active_frontier.json").write_text(
            json.dumps(self.manifest), encoding="utf-8"
        )
        progress = "# Progress\n\n" + status.START_MARKER + "\nstale\n" + status.END_MARKER + "\n\n## Detail\n"
        (self.root / "specs/PROGRESS.md").write_text(progress, encoding="utf-8")
        if render:
            status.render_status(self.root, True)

    def failures(self, now: datetime | None = None) -> list[str]:
        self.write_fixture()
        return status.validate(self.root, now)[0]

    def head(self) -> str:
        return subprocess.run(
            ["git", "rev-parse", "HEAD"], cwd=self.root, text=True,
            stdout=subprocess.PIPE, check=True,
        ).stdout.strip()

    def assert_failure(self, needle: str, now: datetime | None = None) -> None:
        found = self.failures(now)
        self.assertTrue(any(needle in failure for failure in found), found)

    def test_valid_state_and_intake_only_issue_pass(self) -> None:
        failures, state = status.validate(self.root)
        self.assertEqual([], failures)
        self.assertEqual("dat-next", state["next"]["issue_id"])
        self.assertNotIn("dat-intake", {item["issue_id"] for item in self.manifest["frontier"]})

    def test_next_supports_human_and_json_output(self) -> None:
        output = io.StringIO()
        with redirect_stdout(output):
            self.assertEqual(0, status.main(["--root", str(self.root), "next"]))
        human = output.getvalue()
        self.assertIn("dat-next", human)
        self.assertIn("Tracker: open; assignee: unassigned; live claim: none", human)

        for arguments in (
            ["--root", str(self.root), "next", "--json"],
            ["--root", str(self.root), "--json", "next"],
        ):
            output = io.StringIO()
            with redirect_stdout(output):
                self.assertEqual(0, status.main(arguments))
            payload = json.loads(output.getvalue())
            self.assertEqual("open", payload["next"]["tracker_status"])
            self.assertIsNone(payload["next"]["assignee"])
            self.assertFalse(payload["next"]["live_claim"])

    def test_details_supports_human_and_json_output(self) -> None:
        output = io.StringIO()
        with redirect_stdout(output):
            self.assertEqual(0, status.main(["--root", str(self.root), "details"]))
        human = output.getvalue()
        self.assertIn("[TEST-C01] (execution; pending) Complete the fixture requirement.", human)
        self.assertIn("atomically claim", human)
        output = io.StringIO()
        with redirect_stdout(output):
            self.assertEqual(0, status.main([
                "--root", str(self.root), "details", "--json",
            ]))
        details = json.loads(output.getvalue())["details"]
        self.assertEqual(["TEST-C01"], [step["id"] for step in details["steps"]])
        self.assertEqual("unassigned", details["assignee"] or "unassigned")

    def test_details_named_key_and_issue_id_resolve_identically(self) -> None:
        payloads = []
        for target in ("alignment", "dat-next"):
            output = io.StringIO()
            with redirect_stdout(output):
                self.assertEqual(0, status.main([
                    "--root", str(self.root), "details", target, "--json",
                ]))
            payloads.append(json.loads(output.getvalue())["details"])
        self.assertEqual(payloads[0], payloads[1])
        with redirect_stdout(io.StringIO()):
            self.assertEqual(1, status.main([
                "--root", str(self.root), "details", "unknown", "--json",
            ]))

    def test_canonical_next_requires_completion_plan(self) -> None:
        del self.manifest["frontier"][0]["completion"]
        self.assert_failure("canonical next requires a completion plan")

    def test_completion_rejects_bad_step_dependency_and_acceptance(self) -> None:
        step = self.manifest["frontier"][0]["completion"]["steps"][0]
        step["depends_on"] = ["LATER"]
        self.issues[0]["acceptance_criteria"] = "No stable requirement id."
        found = self.failures()
        self.assertTrue(any("must name an earlier step" in value for value in found), found)
        self.assertTrue(any("acceptance criteria" in value for value in found), found)

    def test_planning_completion_rejects_execution_step(self) -> None:
        self.manifest["frontier"][0].update({
            "state": "specified", "authorization": "planning",
        })
        self.assert_failure("execution step is forbidden by planning authorization")

    def test_completion_rejects_missing_and_uncovered_evidence_markers(self) -> None:
        reference = self.manifest["frontier"][0]["completion"]["steps"][0]["requirement_refs"][0]
        reference["marker"] = "MISSING"
        found = self.failures()
        self.assertTrue(any("must occur exactly once" in value for value in found), found)
        self.assertTrue(any("uncovered governing requirement" in value for value in found), found)

    def test_completion_rejects_duplicate_step_ids(self) -> None:
        step = copy.deepcopy(self.manifest["frontier"][0]["completion"]["steps"][0])
        self.manifest["frontier"][0]["completion"]["steps"].append(step)
        self.assert_failure("completion step ids must be unique")

    def test_completion_progress_requires_proof_and_completed_dependencies(self) -> None:
        first = self.manifest["frontier"][0]["completion"]["steps"][0]
        second = copy.deepcopy(first)
        second.update({"id": "TEST-C02", "status": "complete", "depends_on": ["TEST-C01"]})
        second["requirement_refs"] = [{"path": self.doc, "marker": "TEST-C02"}]
        self.manifest["frontier"][0]["completion"]["steps"].append(second)
        self.issues[0]["acceptance_criteria"] += " TEST-C02: Complete the second requirement."
        (self.root / self.doc).write_text(
            (self.root / self.doc).read_text(encoding="utf-8")
            + "\n<!-- REQ:alignment:TEST-C02 -->\nSecond requirement.\n",
            encoding="utf-8",
        )
        found = self.failures()
        self.assertTrue(any("requires completed dependency" in value for value in found), found)
        self.assertTrue(any("requires completion evidence" in value for value in found), found)

    def test_completion_rejects_bogus_proof(self) -> None:
        step = self.manifest["frontier"][0]["completion"]["steps"][0]
        step.update({
            "status": "complete",
            "completion_evidence": [{"kind": "commit", "revision": "not-a-commit"}],
        })
        self.assert_failure("commit does not resolve")

    def test_completion_rejects_non_string_proof_kind_cleanly(self) -> None:
        step = self.manifest["frontier"][0]["completion"]["steps"][0]
        step.update({
            "status": "complete",
            "completion_evidence": [{"kind": [], "revision": self.head()}],
        })
        self.assert_failure("has invalid kind")

    def test_landed_item_requires_all_completion_steps_complete(self) -> None:
        self.issues[0]["status"] = "closed"
        self.manifest["frontier"][0].update({
            "state": "landed", "authorization": "none", "canonical_next": False,
            "landing_commit": self.head(),
        })
        self.issues[1]["labels"] = ["roadmap:frontier"]
        second = self.item(key="second", issue_id="dat-intake", order=1)
        self.manifest["frontier"].append(second)
        self.assert_failure("landed item requires every completion step to be complete")

    def test_repository_s5_contract_has_exhaustive_stable_ids(self) -> None:
        repository = MODULE_PATH.parents[1]
        manifest = json.loads(
            (repository / "specs/active_frontier.json").read_text(encoding="utf-8")
        )
        s5 = next(item for item in manifest["frontier"] if item["key"] == "UVT-S5-SPEC")
        expected = [f"S5-C{number:02d}" for number in range(1, 14)]
        self.assertEqual(expected, [step["id"] for step in s5["completion"]["steps"]])
        self.assertFalse(s5["completion"]["post_completion"]["authorizes_successor"])
        self.assertFalse(s5["completion"]["post_completion"]["selects_successor"])

    def test_duplicate_next_and_order_fail(self) -> None:
        duplicate = self.item(key="second", issue_id="dat-intake")
        self.manifest["frontier"].append(duplicate)
        found = self.failures()
        self.assertTrue(any("duplicate frontier order" in value for value in found), found)
        self.assertTrue(any("exactly one canonical_next" in value for value in found), found)

    def test_duplicate_keys_primary_issues_and_tracker_ids_fail(self) -> None:
        duplicate = self.item(order=1, canonical_next=False)
        self.manifest["frontier"].append(duplicate)
        self.issues.append(copy.deepcopy(self.issues[0]))
        found = self.failures()
        self.assertTrue(any("duplicate frontier key" in value for value in found), found)
        self.assertTrue(any("duplicate primary frontier issue" in value for value in found), found)
        self.assertTrue(any("duplicate tracker issue id" in value for value in found), found)

    def test_orphan_document_and_issue_fail(self) -> None:
        self.manifest["frontier"][0]["governing_docs"] = ["docs/missing.md"]
        self.manifest["frontier"][0]["issue_id"] = "dat-missing"
        found = self.failures()
        self.assertTrue(any("document does not exist" in value for value in found), found)
        self.assertTrue(any("tracker issue does not exist" in value for value in found), found)

    def test_unclassified_document_fails(self) -> None:
        self.governance["entries"].clear()
        self.assert_failure("governing document is not active/classified")

    def test_stale_claim_fails(self) -> None:
        self.issues[0] = self.issue(
            "dat-next", "in_progress", assignee="codex", labels=["roadmap:frontier"]
        )
        self.manifest["frontier"][0].update({
            "state": "in_progress",
            "claim": {
                "agent": "codex",
                "harness": "codex-cli",
                "session": "fixture-session",
                "worktree": ".",
                "head": self.head(),
                "claimed_at": "2026-08-12T10:00:00Z",
                "heartbeat_at": "2026-08-12T11:00:00Z",
                "expires_at": "2026-08-13T11:00:00Z",
                "scope": ["scripts/project_status.py"],
            },
        })
        self.assert_failure("claim expired", datetime(2026, 8, 13, 12, tzinfo=timezone.utc))

    def test_fresh_claim_passes(self) -> None:
        self.issues[0] = self.issue(
            "dat-next", "in_progress", assignee="codex", labels=["roadmap:frontier"]
        )
        self.manifest["frontier"][0].update({
            "state": "in_progress",
            "claim": {
                "agent": "codex", "harness": "codex-cli", "session": "session-1",
                "worktree": ".", "head": self.head(),
                "claimed_at": "2026-08-13T09:00:00Z",
                "heartbeat_at": "2026-08-13T10:00:00Z",
                "expires_at": "2026-08-14T10:00:00Z",
                "scope": ["scripts/project_status.py", "scripts/test_project_status.py"],
            },
        })
        self.assertEqual([], self.failures(datetime(2026, 8, 13, 12, tzinfo=timezone.utc)))
        output = io.StringIO()
        with redirect_stdout(output):
            self.assertEqual(0, status.main([
                "--root", str(self.root), "next", "--json",
            ]))
        payload = json.loads(output.getvalue())
        self.assertEqual("in_progress", payload["next"]["tracker_status"])
        self.assertEqual("codex", payload["next"]["assignee"])
        self.assertTrue(payload["next"]["live_claim"])

    def test_hard_blocker_rejects_ready_next(self) -> None:
        self.issues.append(self.issue("dat-blocker", "open", labels=["roadmap:intake"]))
        self.issues[0]["dependencies"] = [{
            "issue_id": "dat-next", "depends_on_id": "dat-blocker", "type": "blocks"
        }]
        self.manifest["frontier"][0]["dependencies"] = ["dat-blocker"]
        self.assert_failure("unresolved hard blockers")
        self.assert_failure("canonical next is not actionable")

    def test_related_and_parent_edges_are_nonblocking(self) -> None:
        self.issues.extend([
            self.issue("dat-related", "open", labels=["roadmap:intake"]),
            self.issue("dat-parent", "open", labels=["roadmap:intake"]),
        ])
        self.issues[0]["dependencies"] = [
            {"issue_id": "dat-next", "depends_on_id": "dat-related", "type": "related"},
            {"issue_id": "dat-next", "depends_on_id": "dat-parent", "type": "parent-child"},
        ]
        self.assertEqual([], self.failures())

    def test_deferred_next_fails(self) -> None:
        self.issues[0]["status"] = "deferred"
        self.manifest["frontier"][0]["state"] = "deferred"
        found = self.failures()
        self.assertTrue(any("deferred item cannot be canonical next" in value for value in found), found)
        self.assertTrue(any("canonical next is not actionable" in value for value in found), found)

    def test_dependency_mismatch_fails(self) -> None:
        self.issues.append(self.issue("dat-blocker", "closed"))
        self.issues[0]["dependencies"] = [{
            "issue_id": "dat-next", "depends_on_id": "dat-blocker", "type": "blocks"
        }]
        self.assert_failure("dependency mismatch")

    def test_scheduled_issue_without_frontier_record_fails(self) -> None:
        self.issues[1]["labels"] = ["roadmap:frontier"]
        self.assert_failure("scheduled tracker issue is absent from frontier")

    def test_non_closed_issue_requires_one_roadmap_label(self) -> None:
        self.issues[1]["labels"] = []
        self.assert_failure("must have exactly one roadmap label")

    def test_multiple_roadmap_labels_fail(self) -> None:
        self.issues[1]["labels"] = ["roadmap:intake", "roadmap:frontier"]
        self.assert_failure("must have exactly one roadmap label")

    def test_deferred_status_and_label_must_match_both_directions(self) -> None:
        self.issues[1].update({"status": "deferred", "labels": ["roadmap:intake"]})
        self.assert_failure("deferred status/label mismatch")
        self.issues[1].update({"status": "open", "labels": ["roadmap:deferred"]})
        self.assert_failure("deferred status/label mismatch")

    def test_intake_issue_cannot_be_primary_frontier_item(self) -> None:
        self.manifest["frontier"][0]["issue_id"] = "dat-intake"
        self.issues[0]["labels"] = []
        self.assert_failure("intake-only tracker issue cannot be a frontier item")

    def test_specified_planning_item_may_be_next(self) -> None:
        self.manifest["frontier"][0].update({"state": "specified", "authorization": "planning"})
        self.manifest["frontier"][0]["completion"]["steps"][0]["kind"] = "planning"
        self.assertEqual([], self.failures())

    def test_planned_execution_item_cannot_be_next(self) -> None:
        self.manifest["frontier"][0]["state"] = "planned"
        self.assert_failure("planned canonical next requires planning or owner_decision")

    def test_ready_without_separate_authorization_may_be_next(self) -> None:
        self.manifest["frontier"][0]["authorization"] = "none"
        self.assertEqual([], self.failures())

    def test_ready_planning_authorization_fails(self) -> None:
        self.manifest["frontier"][0]["authorization"] = "planning"
        self.assert_failure("authorization planning is invalid for state ready")

    def test_overlapping_active_claims_fail(self) -> None:
        head = self.head()
        claim = {
            "agent": "codex", "harness": "codex-cli", "session": "session-1",
            "worktree": ".", "head": head,
            "claimed_at": "2026-08-13T09:00:00Z",
            "heartbeat_at": "2026-08-13T10:00:00Z",
            "expires_at": "2026-08-14T10:00:00Z",
            "scope": ["shared-subsystem"],
        }
        self.issues[0] = self.issue(
            "dat-next", "in_progress", assignee="codex", labels=["roadmap:frontier"]
        )
        self.issues[1] = self.issue(
            "dat-intake", "in_progress", assignee="claude", labels=["roadmap:frontier"]
        )
        self.manifest["frontier"][0].update({"state": "in_progress", "claim": claim})
        second_claim = copy.deepcopy(claim)
        second_claim.update({"agent": "claude", "session": "session-2"})
        self.manifest["frontier"].append(self.item(
            key="second", order=1, issue_id="dat-intake", canonical_next=False,
            state="in_progress", claim=second_claim,
        ))
        self.assert_failure(
            "overlapping active claims", datetime(2026, 8, 13, 12, tzinfo=timezone.utc)
        )

    def test_unresolvable_claimed_head_fails(self) -> None:
        self.issues[0] = self.issue(
            "dat-next", "in_progress", assignee="codex", labels=["roadmap:frontier"]
        )
        self.manifest["frontier"][0].update({
            "state": "in_progress",
            "claim": {
                "agent": "codex", "harness": "codex-cli", "session": "session-1",
                "worktree": ".", "head": "deadbeef",
                "claimed_at": "2026-08-13T09:00:00Z",
                "heartbeat_at": "2026-08-13T10:00:00Z",
                "expires_at": "2026-08-14T10:00:00Z",
                "scope": ["scripts/project_status.py"],
            },
        })
        self.assert_failure(
            "claimed head does not resolve", datetime(2026, 8, 13, 12, tzinfo=timezone.utc)
        )

    def test_stale_render_fails_and_render_repairs_it(self) -> None:
        progress = self.root / "specs/PROGRESS.md"
        progress.write_text(progress.read_text().replace("CANONICAL NEXT", "OLD NEXT"))
        current, result = status.render_status(self.root, False)
        self.assertFalse(current)
        self.assertEqual("stale", result)
        with redirect_stdout(io.StringIO()):
            self.assertEqual(1, status.main(["--root", str(self.root), "check-render", "--json"]))
            self.assertEqual(1, status.main(["--root", str(self.root), "next", "--json"]))
        current, result = status.render_status(self.root, True)
        self.assertFalse(current)
        self.assertEqual("updated", result)
        self.assertTrue(status.render_status(self.root, False)[0])

    def test_render_uses_stable_keys_without_numbered_list(self) -> None:
        block = status.render_block(self.manifest)
        self.assertIn("(`alignment`; `dat-next`)", block)
        self.assertNotIn("0. **", block)

    def test_landed_requires_closed_issue_and_real_commit(self) -> None:
        revision = self.head()
        decision = self.root / self.doc
        decision.write_text(
            decision.read_text(encoding="utf-8")
            + "\n<!-- REQ:second:TEST-C01 -->\nSecond requirement.\n",
            encoding="utf-8",
        )
        self.issues[0]["status"] = "closed"
        self.manifest["frontier"][0]["completion"]["steps"][0].update({
            "status": "complete",
            "completion_evidence": [{"kind": "commit", "revision": revision}],
        })
        self.manifest["frontier"][0].update({
            "state": "landed", "authorization": "none", "canonical_next": False,
            "landing_commit": revision,
        })
        self.issues[1]["labels"] = ["roadmap:frontier"]
        second = self.item(key="second", issue_id="dat-intake", order=1)
        self.manifest["frontier"].append(second)
        self.assertEqual([], self.failures())
        self.manifest["frontier"][0]["landing_commit"] = "deadbeef"
        self.assert_failure("landing commit does not resolve")


if __name__ == "__main__":
    unittest.main()
