#!/usr/bin/env python3
"""Regressions for the three foundation consumer completion contracts.

Use committed contract shapes as inputs; mutations remain in memory or in an
owned temporary directory. No tracker, claim, acceptance or runtime is changed.
"""

from __future__ import annotations

import copy
import io
import json
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from unittest.mock import patch

import project_status
from project_task_details import validate_completion


ROOT = Path(__file__).resolve().parents[1]
PLAN = "specs/FOUNDATION_CONSUMER_COMPLETION_PLAN.md"
KEYS = ("UVT-S5A-BUILD", "GUI-WRITE-PATH", "NATIVE-AUTHORING")


class FoundationConsumerCompletionTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / "specs").mkdir()
        self.source = (ROOT / PLAN).read_text(encoding="utf-8")
        (self.root / PLAN).write_text(self.source, encoding="utf-8")
        manifest = json.loads((ROOT / "specs/active_frontier.json").read_text())
        self.items = {i["key"]: i for i in manifest["frontier"] if i["key"] in KEYS}
        # Test contract semantics at a planning boundary, not today's mutable
        # ownership/status. Legitimate later progress must not break this suite.
        for item in self.items.values():
            item.update(state="specified", authorization="planning", canonical_next=False)
            item.pop("claim", None)
            completion = item["completion"]
            completion["canonical_next_step_id"] = completion["steps"][0]["id"]
            for step in completion["steps"]:
                step.update(status="pending", completion_evidence=[])
        self.issues = {
            i["id"]: i for line in (ROOT / ".beads/issues.jsonl").read_text().splitlines()
            if line.strip() for i in [json.loads(line)]
        }
        self.governed = json.loads(
            (ROOT / "specs/spec_governance_manifest.json").read_text()
        )["entries"]

    def errors(self, item):
        return validate_completion(self.root, item, self.issues, self.governed)

    def test_all_consumers_have_valid_complete_contract_shapes(self):
        self.assertEqual(set(KEYS), set(self.items))
        for key, item in self.items.items():
            with self.subTest(key=key):
                self.assertEqual([], self.errors(item))
                steps = item["completion"]["steps"]
                self.assertEqual(
                    ["planning", "owner_decision", "execution", "execution",
                     "execution", "owner_decision"],
                    [s["kind"] for s in steps],
                )
                self.assertTrue(all(s["status"] == "pending" for s in steps))
                self.assertEqual(steps[0]["id"], item["completion"]["canonical_next_step_id"])
                self.assertEqual("planning", item["authorization"])
                self.assertNotIn("claim", item)
                self.assertFalse(item["canonical_next"])

    def test_missing_plan_is_rejected_without_inventing_a_step(self):
        for key, original in self.items.items():
            with self.subTest(key=key):
                item = copy.deepcopy(original)
                del item["completion"]
                state = {"next": item, "manifest": {"frontier": [item]},
                         "issues": self.issues}
                output, errors = io.StringIO(), io.StringIO()
                with patch.object(project_status, "validate", return_value=([], state)), \
                     patch.object(project_status, "render_status", return_value=(True, "current")), \
                     redirect_stdout(output), redirect_stderr(errors):
                    result = project_status.main(["--root", str(self.root), "details", key])
                self.assertEqual(1, result)
                self.assertEqual(
                    f"Project task details failed: {key} has no completion plan\n",
                    errors.getvalue(),
                )
                self.assertEqual("", output.getvalue())

    def test_acceptance_ids_cannot_omit_required_proof(self):
        for item in self.items.values():
            with self.subTest(key=item["key"]):
                issue = self.issues[item["issue_id"]]
                before = issue["acceptance_criteria"]
                issue["acceptance_criteria"] = before.replace(
                    item["completion"]["steps"][3]["id"] + ":", "omitted:"
                )
                self.assertTrue(any("acceptance" in e for e in self.errors(item)))
                issue["acceptance_criteria"] = before

    def test_missing_requirement_marker_is_rejected(self):
        for item in self.items.values():
            with self.subTest(key=item["key"]):
                step = item["completion"]["steps"][0]
                marker = f"<!-- REQ:{item['key']}:{step['id']} -->"
                (self.root / PLAN).write_text(self.source.replace(marker, ""))
                self.assertTrue(any("marker must occur exactly once" in e
                                    for e in self.errors(item)))
        (self.root / PLAN).write_text(self.source)

    def test_planning_cannot_select_execution(self):
        for original in self.items.values():
            with self.subTest(key=original["key"]):
                item = copy.deepcopy(original)
                item["completion"]["canonical_next_step_id"] = item["completion"]["steps"][2]["id"]
                errors = self.errors(item)
                self.assertTrue(any("incomplete dependencies" in e for e in errors))
                self.assertTrue(any("requires item authorization" in e for e in errors))

    def test_acceptance_cannot_be_completed_without_review(self):
        item = copy.deepcopy(self.items["NATIVE-AUTHORING"])
        item["completion"]["steps"][-1]["status"] = "complete"
        self.assertTrue(any("complete step requires completion evidence" in e
                            for e in self.errors(item)))

    def test_successor_effects_must_match_real_tracker_edges(self):
        for original in self.items.values():
            with self.subTest(key=original["key"]):
                item = copy.deepcopy(original)
                item["completion"]["post_completion"]["unblocks_issue_ids"] = ["dat-invented"]
                self.assertTrue(any("unblocks mismatch" in e for e in self.errors(item)))


if __name__ == "__main__":
    unittest.main()
