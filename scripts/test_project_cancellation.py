#!/usr/bin/env python3
"""Hermetic lifecycle regressions for terminal roadmap cancellation."""
from __future__ import annotations

import copy
import io
import json
import subprocess
import unittest
from contextlib import redirect_stdout
from unittest.mock import patch

import test_project_status as fixtures
from project_cancellation import validate_cancellation_history


class ProjectCancellationTest(unittest.TestCase):
    # Reuse fixture construction without inheriting the unrelated status tests.
    setUp = fixtures.ProjectStatusTest.setUp
    tearDown = fixtures.ProjectStatusTest.tearDown
    issue = staticmethod(fixtures.ProjectStatusTest.issue)
    item = fixtures.ProjectStatusTest.item
    write_fixture = fixtures.ProjectStatusTest.write_fixture
    failures = fixtures.ProjectStatusTest.failures
    assert_failure = fixtures.ProjectStatusTest.assert_failure

    def cancel(self) -> dict:
        marker = "<!-- OWNER:CANCEL-1 -->"
        document = self.root / self.doc
        document.write_text(document.read_text() + "\n" + marker +
                            "\nOwner canceled this task and selected explicit idle.\n")
        source = {"path": self.doc, "marker": marker}
        item = self.manifest["frontier"][0]
        item.update(state="canceled", authorization="none", canonical_next=False,
                    cancellation={"reason": "Owner withdrew this task.", "source_ref": source})
        item["completion"]["canonical_next_step_id"] = None
        self.issues[0]["status"] = "closed"
        self.manifest.update(schema_version=7, selection={
            "mode": "idle", "reason": "No successor selected.", "source_ref": source,
        })
        return item

    def query(self, *arguments: str) -> str:
        self.write_fixture()
        output = io.StringIO()
        with redirect_stdout(output):
            result = fixtures.status.main(["--root", str(self.root), *arguments])
        self.assertEqual(0, result, output.getvalue())
        return output.getvalue()

    def commit_cancellation(self) -> dict:
        item = self.cancel()
        self.assertEqual([], self.failures())
        subprocess.run(["git", "add", "specs/active_frontier.json", "specs/PROGRESS.md",
                        ".beads/issues.jsonl", self.doc], cwd=self.root, check=True)
        subprocess.run(["git", "commit", "-qm", "Cancel fixture task"],
                       cwd=self.root, check=True)
        return item

    def test_canceled_pending_execution_remains_historical_and_idle(self) -> None:
        original = copy.deepcopy(self.manifest["frontier"][0]["completion"]["steps"])
        item = self.cancel()
        self.assertEqual([], self.failures())
        self.assertEqual(original, item["completion"]["steps"])
        for command in ("next", "details"):
            with self.subTest(command=command):
                payload = json.loads(self.query(command, "--json"))
                self.assertIsNone(payload[command])
                self.assertEqual("idle", payload["selection"]["mode"])
                self.assertIn("STOP", payload["work_start"])
                for flag in ((), ("--full",)):
                    self.assertIn("explicit idle", self.query(command, *flag))

    def test_missing_or_malformed_cancellation_disposition_is_rejected(self) -> None:
        item = self.cancel()
        valid = copy.deepcopy(item["cancellation"])
        variants = [None, {}, {**valid, "reason": " "}, {**valid, "reason": 1},
                    {**valid, "source_ref": {"path": self.doc, "marker": "MISSING"}},
                    {**valid, "source_ref": {"path": "../outside.md", "marker": "CANCEL"}},
                    {**valid, "unexpected": True}]
        for record in variants:
            with self.subTest(record=record):
                item["cancellation"] = record
                self.assert_failure("canceled work requires reason and governed owner disposition")
        del item["cancellation"]
        self.assert_failure("canceled work requires reason and governed owner disposition")

    def test_cancellation_disposition_must_be_governed(self) -> None:
        self.cancel()
        self.governance["entries"][self.doc]["class"] = "pending"
        self.assert_failure("canceled work requires reason and governed owner disposition")

    def test_canceled_task_cannot_be_selected_or_authorized(self) -> None:
        item = self.cancel()
        item["canonical_next"] = True
        self.assert_failure("canceled work cannot be selected or authorized")
        item["canonical_next"] = False
        for authorization in ("execution", "planning", "owner_decision"):
            with self.subTest(authorization=authorization):
                item["authorization"] = authorization
                self.assert_failure("canceled work cannot be selected or authorized")

    def test_canceled_task_cannot_retain_claim_or_hold(self) -> None:
        item = self.cancel()
        for field in ("claim", "hold"):
            with self.subTest(field=field):
                item[field] = {}
                self.assert_failure("canceled work cannot retain a claim or hold")
                del item[field]

    def test_canceled_plan_cannot_select_or_run_step(self) -> None:
        item = self.cancel()
        completion = item["completion"]
        completion["canonical_next_step_id"] = "TEST-C01"
        self.assert_failure("canceled plan cannot have a selected or active step")
        completion["canonical_next_step_id"] = None
        completion["steps"][0]["status"] = "in_progress"
        self.assert_failure("canceled plan cannot have a selected or active step")

    def test_canceled_task_requires_closed_tracker(self) -> None:
        self.cancel()
        for tracker_status in ("open", "in_progress", "deferred"):
            with self.subTest(tracker_status=tracker_status):
                self.issues[0]["status"] = tracker_status
                self.assert_failure("state canceled requires tracker status closed")

    def test_current_cancellation_record_cannot_be_reactivated(self) -> None:
        item = self.cancel()
        item.update(state="ready", authorization="execution", canonical_next=True)
        item["completion"]["canonical_next_step_id"] = "TEST-C01"
        self.issues[0]["status"] = "open"
        self.manifest["selection"] = {"mode": "selected"}
        self.assert_failure("cancellation requires terminal canceled state")

    def test_committed_cancellation_cannot_reopen_even_without_marker(self) -> None:
        item = self.commit_cancellation()
        item.update(state="ready", authorization="execution", canonical_next=True)
        item["completion"]["canonical_next_step_id"] = "TEST-C01"
        self.issues[0]["status"] = "open"
        self.manifest["selection"] = {"mode": "selected"}
        self.assert_failure("committed cancellation must be preserved")
        del item["cancellation"]
        self.assert_failure("committed cancellation must be preserved")

    def test_committed_cancellation_cannot_disappear(self) -> None:
        self.commit_cancellation()
        self.manifest["frontier"] = []
        self.assert_failure("committed cancellation must be preserved")

    def test_just_committed_removal_is_rejected_against_parent(self) -> None:
        self.commit_cancellation()
        self.manifest["frontier"] = []
        self.write_fixture()
        subprocess.run(["git", "add", "specs/active_frontier.json"], cwd=self.root, check=True)
        subprocess.run(["git", "commit", "-qm", "Invalid removal"], cwd=self.root, check=True)
        self.assert_failure("committed cancellation must be preserved")

    def test_malformed_committed_history_fails_without_crashing(self) -> None:
        for data in ([], {"frontier": None}, {"frontier": [{"issue_id": []}]}):
            with self.subTest(data=data), patch("project_cancellation.subprocess.run") as run:
                run.return_value = subprocess.CompletedProcess([], 0, json.dumps(data), "")
                self.assertTrue(validate_cancellation_history(self.root, self.manifest))

    def test_malformed_current_identity_cannot_hide_committed_cancellation(self) -> None:
        item = self.commit_cancellation()
        item["issue_id"] = []
        self.assertTrue(validate_cancellation_history(self.root, self.manifest))

    def test_malformed_current_frontier_fails_without_crashing(self) -> None:
        for value in (None, 3, {}):
            with self.subTest(value=value):
                self.assertTrue(validate_cancellation_history(self.root, {"frontier": value}))

    def test_committed_cancellation_cannot_rewrite_disposition(self) -> None:
        item = self.commit_cancellation()
        item["cancellation"]["reason"] = "Replacement account of cancellation."
        self.assert_failure("committed cancellation must be preserved")

    def test_canceled_prerequisite_does_not_satisfy_delivery_dependency(self) -> None:
        canceled = self.cancel()
        dependent = self.item(key="dependent", order=1, issue_id="dat-dependent",
                              state="blocked", authorization="none", canonical_next=False,
                              dependencies=["dat-next"])
        del dependent["completion"]
        self.manifest["frontier"].append(dependent)
        self.issues.append(self.issue("dat-dependent", "open", labels=["roadmap:frontier"],
                                     dependencies=[{"depends_on_id": "dat-next", "type": "blocks"}]))
        canceled["unblocks"] = ["dat-dependent"]
        canceled["completion"]["post_completion"]["unblocks_issue_ids"] = ["dat-dependent"]
        self.assertEqual([], self.failures())
        dependent.update(state="ready", authorization="execution")
        self.assert_failure("dependent: state ready has unresolved hard blockers: ['dat-next']")

    def test_canceled_details_preserve_truth_in_brief_full_and_json(self) -> None:
        item = self.cancel()
        for flag in ((), ("--full",)):
            with self.subTest(flag=flag):
                output = self.query("details", "alignment", *flag)
                self.assertIn("State: canceled", output)
                self.assertIn("none; canceled", output)
                self.assertIn("STOP", output)
                self.assertNotIn("all steps complete", output)
                self.assertNotIn("every completion step is complete", output)
        output = self.query("details", "alignment", "--full")
        self.assertIn("(execution; pending)", output)
        payload = json.loads(self.query("details", "dat-next", "--json"))["details"]
        self.assertEqual("canceled", payload["state"])
        self.assertEqual("closed", payload["tracker_status"])
        self.assertEqual(item["cancellation"], payload["cancellation"])
        self.assertEqual(item["completion"]["steps"], payload["steps"])
        self.assertIsNone(payload["canonical_next_step_id"])
        self.assertFalse(payload["canonical_next"])
        self.assertIn("canceled", payload["work_start"])


if __name__ == "__main__":
    unittest.main()
