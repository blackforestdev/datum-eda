"""Real CLI/selector regression for proposed external terminal-owner closure.

Synthetic Git fixtures only; never product acceptance or installed authority.
Run with Python -I -S -B. The historical reviewed runtime remains unchanged.
"""

from copy import deepcopy
from pathlib import Path
import sys
import unittest
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
RUNTIME = ROOT / ".git/datum-wdq/proposals/sequencing-repair-20260910/scripts"
sys.path.insert(0, str(RUNTIME))
sys.path.insert(0, str(HERE))

import test_workflow_delivery_category_boundaries as support
import workflow_delivery_coverage_runtime as runtime
from terminal_owner_categories import terminal_owner_closeout, validate_categories
from workflow_delivery_selector import selector_failures


class TerminalOwnerTest(unittest.TestCase):
    def setUp(self):
        self.h = support.ExternalBoundaryTest()
        self.h.setUp()
        self.addCleanup(self.h.doCleanups)
        self.item = self.h.item
        first, last = self.item["completion"]["steps"]
        first.update(status="complete", completion_evidence=[{"kind": "document", **self.h.f.ref}])
        last.update(kind="owner_decision", status="pending", owner_input={
            "response_format": "Fixture owner accept or revise",
            "requests": [{"id": "FINAL", "question": "Accept the exact fixture?",
                          "recommended_response": "Fixture only",
                          "source_ref": last["requirement_refs"][0]}]})
        self.item.update(state="specified", authorization="owner_decision", claim=None)
        self.item["completion"]["canonical_next_step_id"] = last["id"]
        self.h.issue.update(status="open", assignee="")
        self.successor = deepcopy(self.item)
        self.successor.update(key="AFTER", issue_id="dat-after", order=2, canonical_next=False)
        self.h.m["frontier"].append(self.successor)
        successor_issue = deepcopy(self.h.issue)
        successor_issue.update(id="dat-after")
        self.h.issues.append(successor_issue)
        row = deepcopy(self.h.policy["coverage"]["rows"][1])
        row.update(frontier_key="AFTER", issue_id="dat-after")
        self.h.policy["coverage"]["rows"].append(row)
        doc = last["requirement_refs"][0]["path"]
        self.h.f.write(doc, self.h.f.root.joinpath(doc).read_bytes() + (
            "\n<!-- REQ:AFTER:NEXT-C01 -->\n<!-- REQ:AFTER:NEXT-C02 -->\n"
            "<!-- OWNER:AFTER:NEXT-C02:NEXT-C02 -->\n"
            "<!-- OWNER:NEXT:NEXT-C02:NEXT-C02 -->\n"
            "<!-- FIXTURE-OWNER-REVIEW -->\n"
            "Synthetic fixture owner boundary, not a real product disposition.\n").encode())
        self.h.promote()
        self.landing = self.h.h.authority

    def close(self):
        self.item.update(state="landed", authorization="none", claim=None,
                         canonical_next=False, landing_commit=self.landing)
        self.item["completion"]["canonical_next_step_id"] = None
        self.item["completion"]["steps"][-1].update(status="complete",
            completion_evidence=[{"kind": "review",
                "path": self.item["completion"]["steps"][-1]["requirement_refs"][0]["path"],
                "marker": "<!-- FIXTURE-OWNER-REVIEW -->"}])
        self.successor["canonical_next"] = True
        self.h.issue.update(status="closed", assignee="")
        self.h.save()

    def revised_cli(self):
        with patch.object(runtime, "validate_categories", validate_categories):
            return self.h.h.cli()

    def test_historical_runtime_reproduces_the_freeze(self):
        self.close()
        code, report = self.h.h.cli()
        self.assertEqual(1, code, report)
        self.assertIn("external-lane authorization changed", report["findings"][0]["detail"])

    def test_terminal_owner_closure_passes_cli_and_selector(self):
        self.close()
        code, report = self.revised_cli()
        self.assertEqual(0, code, report)
        with patch.object(runtime, "validate_categories", validate_categories):
            self.assertEqual([], selector_failures(self.h.f.root, self.h.m))

    def test_unchanged_owner_boundary_passes(self):
        code, report = self.revised_cli()
        self.assertEqual(0, code, report)

    def test_owner_closure_does_not_allow_source_changes(self):
        self.close()
        self.h.f.write("external/owned/input.py", b"unapproved execution\n")
        self.h.save()
        code, report = self.revised_cli()
        self.assertEqual(1, code, report)
        self.assertTrue(report["findings"])

    def test_requirements_and_other_steps_stay_immutable(self):
        for index in (0, 1):
            with self.subTest(index=index):
                original = deepcopy(self.item)
                self.close()
                self.item["completion"]["steps"][index]["action"] = "Unauthorized expansion"
                self.h.save()
                code, report = self.revised_cli()
                self.assertEqual(1, code, report)
                self.item.clear()
                self.item.update(original)

    def test_missing_owner_completion_evidence_refuses(self):
        self.close()
        self.item["completion"]["steps"][-1]["completion_evidence"] = []
        self.h.save()
        code, report = self.revised_cli()
        self.assertEqual(1, code, report)

    def test_missing_tracker_closure_refuses(self):
        self.close()
        self.h.issue["status"] = "open"
        self.h.save()
        code, report = self.revised_cli()
        self.assertEqual(1, code, report)

    def test_document_only_evidence_is_not_owner_review(self):
        self.close()
        self.item["completion"]["steps"][-1]["completion_evidence"][0]["kind"] = "document"
        self.h.save()
        code, report = self.revised_cli()
        self.assertEqual(1, code, report)
        self.assertIn("review/decision evidence", report["findings"][0]["detail"])

    def test_unfinished_promoted_step_disables_exception(self):
        prior = deepcopy(self.item)
        prior["completion"]["steps"][0]["status"] = "pending"
        self.close()
        self.assertFalse(terminal_owner_closeout(self.item, prior))

    def test_missing_or_invalid_landing_commit_refuses(self):
        self.close()
        for landing in (None, "0" * 40):
            with self.subTest(landing=landing):
                self.item["landing_commit"] = landing
                self.h.save()
                code, report = self.revised_cli()
                self.assertEqual(1, code, report)
                self.assertRegex(report["findings"][0]["detail"], r"landing[ _]commit")

    def test_owner_completion_cannot_authorize_new_execution(self):
        self.close()
        self.item.update(state="ready", authorization="execution")
        self.h.issue["status"] = "open"
        self.h.save()
        code, report = self.revised_cli()
        self.assertEqual(1, code, report)


if __name__ == "__main__":
    unittest.main()
