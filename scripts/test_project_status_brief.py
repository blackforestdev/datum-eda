"""Routine roadmap reads stay small while full authority remains available."""
import copy
import io
import json
import unittest
from contextlib import redirect_stdout

import test_project_status as fixtures

status = fixtures.status


class BriefingTest(unittest.TestCase):
    def setUp(self):
        self.fixture = fixtures.ProjectStatusTest()
        self.fixture.setUp()
        self.addCleanup(self.fixture.tearDown)

    def run_command(self, *args):
        output = io.StringIO()
        with redirect_stdout(output):
            code = status.main(["--root", str(self.fixture.root), *args])
        self.assertEqual(0, code)
        return output.getvalue()

    def test_default_next_and_details_do_not_dump_contract(self):
        for command in ("next", "details"):
            brief = self.run_command(command)
            full = self.run_command(command, "--full")
            payload = json.loads(self.run_command(command, "--json"))[command]
            self.assertIn("dat-next", brief)
            self.assertIn("[TEST-C01]", brief)
            self.assertIn("atomically claim", brief)
            self.assertIn("--full", brief)
            self.assertNotIn("Presentation contract: return", full)
            self.assertEqual("human_summary", (payload.get("completion") or payload)["presentation_policy"]["mode"])
            self.assertNotIn("Completion evidence:", brief)
            self.assertLess(len(brief), 2000)
            if command == "details":
                self.assertIn("Completion evidence:", full)

    def test_large_contract_has_bounded_brief_and_explicit_omission(self):
        item = copy.deepcopy(self.fixture.manifest["frontier"][0])
        item["title"] = "Long title " * 10000
        item["summary"] = "Unneeded history " * 10000
        step = item["completion"]["steps"][0]
        step["action"] = "Long action " * 10000
        item["completion"]["steps"] = [dict(step, id=f"STEP-{i}") for i in range(100)]
        item["completion"]["canonical_next_step_id"] = "STEP-99"
        view = status.completion_view(item, self.fixture.issues[0])
        brief = status.render_brief(view, True)
        self.assertLess(len(brief), 3000)
        self.assertIn("Next step: [STEP-99]", brief)
        self.assertIn("95 more steps", brief)
        self.assertNotIn("Unneeded history", brief)

    def test_decision_brief_preserves_stop_and_routing_full_keeps_packet(self):
        item = self.fixture.configure_owner_boundary()
        view = status.completion_view(item, self.fixture.issues[0])
        brief = status.render_brief(view)
        self.assertIn("INPUT REQUIRED from project_owner", brief)
        self.assertIn("Do not claim, edit", brief)
        full = status.render_completion(view)
        self.assertIn("[OPEN-1] Approve?", full)
        self.assertIn("Owner response format: Reply approve or revise.", full)

    def test_hold_claim_completed_unselected_and_idle_remain_explicit(self):
        item = self.fixture.manifest["frontier"][0]
        issue = self.fixture.issues[0]
        cases = [
            ({"hold": {"reason": "Blocked", "resume_requires": "Resolve blocker"}}, "STOP: held work"),
            ({"claim": {"agent": "other"}}, "stand down"),
            ({"state": "landed"}, "STOP: completed work"),
            ({"canonical_next": False}, "STOP: unselected work"),
            ({"state": "deferred"}, "STOP: deferred work"),
        ]
        for changes, restriction in cases:
            with self.subTest(changes=changes):
                view = status.next_view(dict(item, **changes), issue)
                self.assertIn(restriction, status.render_brief(view))
        idle = status.idle_view({"selection": {"reason": "No successor " * 10000}})
        self.assertIn("STOP: explicit idle", status.render_brief(idle))
        self.assertLess(len(status.render_brief(idle)), 1000)


if __name__ == "__main__":
    unittest.main()
