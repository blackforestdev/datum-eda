#!/usr/bin/env python3
"""CLI exit/readonly behavior, checkpoint kinds, and legacy selector preservation."""

from contextlib import redirect_stdout
from copy import deepcopy
import io
import json
from pathlib import Path
import unittest

from check_workflow_delivery import main
from workflow_delivery_checkpoints import delivery_shape, validate_delivery
from workflow_delivery_io import DeliveryInputError
from workflow_delivery_native_test_support import environment
from workflow_delivery_selector import selector_failures
from workflow_delivery_test_support import Fixture
from workflow_delivery_tree import Tree
from workflow_delivery_trust_test_support import accepted_fixture


class CliTests(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.close)

    def run_cli(self, *args):
        before = self.f.snapshot()
        output = io.StringIO()
        with redirect_stdout(output):
            code = main(["--root", str(self.f.root), *args])
        self.assertEqual(before, self.f.snapshot())
        return code, json.loads(output.getvalue())

    def test_report_only_does_not_claim_readiness_or_acceptance(self):
        code, report = self.run_cli("--report-only", "--contract", "contract.json")
        self.assertEqual(code, 0)
        self.assertEqual(report["findings"], [])
        self.assertIs(report["readiness_asserted"], False)
        self.assertIs(report["acceptance_asserted"], False)

    def test_n01_report_contains_missing_and_parse_locations(self):
        code, report = self.run_cli("--contract", "absent.json")
        self.assertEqual(code, 0)
        self.assertEqual(report["findings"][0]["path"], "absent.json")
        for raw in (b"{", b'{"id":1,"id":2}', b'{"unknown":true}'):
            self.f.write("contract.json", raw)
            _, report = self.run_cli("--contract", "contract.json")
            self.assertEqual(report["findings"][0]["code"], "WDQ-CONTRACT")

    def test_n18_enforce_returns_two_without_explicit_trust(self):
        code, report = self.run_cli("--enforce")
        self.assertEqual(code, 2)
        self.assertEqual(report["findings"][0]["code"], "WDQ-TRUST")

    def test_enforce_success_and_refusal_exit_codes(self):
        _, _, _, authority = accepted_fixture(self.f)
        self.f.save("requested-environment.json", environment())
        args = ("--enforce", "--authority-ref", authority, "--base-ref", authority,
                "--environment-path", "requested-environment.json")
        code, report = self.run_cli(*args)
        self.assertEqual(code, 0, report)
        self.f.write("src/read.py", b"untested change")
        code, report = self.run_cli(*args)
        self.assertEqual(code, 1, report)
        self.assertEqual(report["findings"][0]["code"], "WDQ-STALE")
        self.assertEqual(report["findings"][0]["key"], "TASK")
        self.assertEqual(report["findings"][0]["step"], "A")

    def test_p01_readiness_does_not_demand_future_proof(self):
        item, _, _, _ = accepted_fixture(self.f)
        for step in item["completion"]["steps"]:
            step["status"] = "complete" if step["id"] == "R" else "pending"
        item["completion"]["canonical_next_step_id"] = "I"
        item["authorization"] = "execution"
        (self.f.root / self.f.contract["proof_path"]).unlink()
        self.assertEqual(validate_delivery(Tree(self.f.root), item), "ready")

    def test_checkpoint_nullability_kind_order_and_unknown_keys(self):
        item, _, _, _ = accepted_fixture(self.f)
        original = deepcopy(item)
        for mutate in (
            lambda i: i["completion"]["delivery"]["checkpoints"].update(activate=None),
            lambda i: i["completion"]["delivery"]["checkpoints"].update(accept="V"),
            lambda i: i["completion"]["steps"][2].update(depends_on=[]),
            lambda i: i["completion"]["delivery"].update(disabled=True),
        ):
            item = deepcopy(original)
            mutate(item)
            with self.assertRaises(DeliveryInputError) as caught:
                delivery_shape(item, self.f.contract)
            self.assertEqual(caught.exception.code, "WDQ-TRANSITION")

    def test_p07_unenrolled_history_and_other_session_dirt(self):
        self.f.save("specs/active_frontier.json", {"schema_version": 5, "frontier": []})
        self.f.write("preferences/unfinished.py", b"other session")
        code, report = self.run_cli("--report-only", "--staged")
        # The staged tree has no Frontier: worktree bytes cannot mask that absence.
        self.assertEqual(report["findings"][0]["code"], "WDQ-INDEX")
        self.f.stage()
        code, report = self.run_cli("--report-only", "--staged")
        self.assertEqual(code, 0)
        self.assertEqual(report["findings"], [])

    def test_n16_enforce_reuses_full_frontier_invariants(self):
        _, _, _, authority = accepted_fixture(self.f)
        self.f.save("requested-environment.json", environment())
        args = ("--enforce", "--authority-ref", authority, "--base-ref", authority,
                "--environment-path", "requested-environment.json")
        original = Tree(self.f.root).json("specs/active_frontier.json")
        for mutate in (lambda m: m.update(schema_version=7),
                       lambda m: m["frontier"].append(deepcopy(m["frontier"][0])),
                       lambda m: m["frontier"][1].update(canonical_next=False),
                       lambda m: m["frontier"][1]["completion"].update(canonical_next_step_id="WRONG")):
            changed = deepcopy(original)
            mutate(changed)
            self.f.save("specs/active_frontier.json", changed)
            code, report = self.run_cli(*args)
            self.assertEqual(code, 1, report)
            self.assertEqual(report["findings"][0]["code"], "WDQ-TRANSITION")


class SelectorCompatibility(unittest.TestCase):
    def test_schema6_preserves_legacy_next_and_details_bytes(self):
        from test_project_status import ProjectStatusTest, status
        fixture = ProjectStatusTest()
        fixture.setUp()
        self.addCleanup(fixture.tearDown)
        def output(command):
            stream = io.StringIO()
            with redirect_stdout(stream):
                result = status.main(["--root", str(fixture.root), command])
            self.assertEqual(result, 0, stream.getvalue())
            return stream.getvalue()
        before = {command: output(command) for command in ("next", "details")}
        fixture.manifest["schema_version"] = 6
        fixture.write_fixture()
        (fixture.root / "other-session.py").write_text("unfinished preferences\n")
        for command in before:
            self.assertEqual(output(command), before[command])

    def test_unknown_schema_still_refuses(self):
        from test_project_status import ProjectStatusTest, status
        fixture = ProjectStatusTest()
        fixture.setUp()
        self.addCleanup(fixture.tearDown)
        fixture.manifest["schema_version"] = 7
        fixture.write_fixture()
        failures, _ = status.validate(fixture.root)
        self.assertTrue(any("schema_version" in f for f in failures))


if __name__ == "__main__":
    unittest.main()
