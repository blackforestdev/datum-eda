#!/usr/bin/env python3
"""Preparation transformations and owner-hook wiring, not actual promotion."""

from copy import deepcopy
import json
from pathlib import Path
import subprocess
import shutil
import tempfile
import unittest

from workflow_delivery_prepare import KEY, completion_proposal, prepare, replace_item, live_state, require_unchanged
from workflow_delivery_test_support import Fixture
from workflow_delivery_prepare_handoff import promotion_markdown


class ProposalTests(unittest.TestCase):
    def test_owner_commands_are_syntax_valid_and_pin_exact_ids(self):
        result = {"candidate": "a" * 40, "base": "b" * 40,
                  "runner": "/external with spaces/scripts/check_workflow_delivery.py",
                  "hooks": "/external with spaces/hooks", "environment": "evidence/environment.json",
                  "packet_sha256": "c" * 64, "review_sha256": "d" * 64}
        text = promotion_markdown(Path("/repo with spaces"), result)
        commands = text.split("```bash\n")[2].split("```")[0]
        self.assertEqual(subprocess.run(["bash", "-n"], input=commands.encode(), capture_output=True).returncode, 0)
        self.assertLess(commands.index("--candidate-ref"), commands.index("git merge --ff-only"))
        self.assertIn("--authority-ref " + result["candidate"], commands)
        self.assertNotIn("--no-verify", commands)

    def manifest(self):
        return {"frontier": [{"key": "UNRELATED", "claim": {"agent": "other"}},
            {"key": KEY, "authorization": "owner_decision", "state": "planned",
             "completion": {"canonical_next_step_id": "WDQ-G06", "steps": [
                 {"id": "WDQ-G06P", "status": "complete"},
                 {"id": "WDQ-G06", "status": "pending", "completion_evidence": []}]}}]}

    def test_only_pilot_is_changed_in_isolated_proposal(self):
        original = self.manifest()
        proposal = completion_proposal(deepcopy(original), "a" * 40)
        self.assertEqual(proposal["frontier"][0], original["frontier"][0])
        item = proposal["frontier"][1]
        self.assertEqual(item["state"], "landed")
        self.assertIsNone(item["completion"]["canonical_next_step_id"])
        self.assertEqual(item["completion"]["delivery"]["checkpoints"]["accept"], "WDQ-G06")
        self.assertNotIn("claim", item)

    def test_incomplete_preparation_or_active_claim_refuses(self):
        for change in ("preparation", "claim", "authorization", "accepted"):
            with self.subTest(change=change):
                value = self.manifest()
                item = value["frontier"][1]
                if change == "preparation":
                    item["completion"]["steps"][0]["status"] = "in_progress"
                elif change == "accepted":
                    item["completion"]["steps"][1]["status"] = "complete"
                elif change == "claim":
                    item["claim"] = {"agent": "writer"}
                else:
                    item["authorization"] = "execution"
                with self.assertRaises(ValueError):
                    completion_proposal(value, "a" * 40)

    def test_other_item_bytes_survive_proposal_serialization(self):
        raw = json.dumps(self.manifest(), separators=(",", ": "))
        proposal = completion_proposal(self.manifest(), "a" * 40)
        changed = replace_item(raw, proposal["frontier"][1])
        self.assertEqual(json.loads(changed), proposal)
        self.assertEqual(raw[:raw.index('{"key": "' + KEY)],
                         changed[:changed.index('{\n      "key": "' + KEY)])

    def test_authorized_preparation_can_generate_before_live_completion(self):
        value = self.manifest()
        item = value["frontier"][1]
        item["authorization"] = "execution"
        item["claim"] = {"agent": "codex"}
        item["completion"]["steps"][0]["status"] = "in_progress"
        proposed = completion_proposal(deepcopy(value), "a" * 40)
        self.assertEqual(value["frontier"][1]["completion"]["steps"][0]["status"], "in_progress")
        self.assertEqual(proposed["frontier"][1]["completion"]["steps"][0]["status"], "complete")

    def test_predecessor_only_loses_the_closed_pilot_unblock(self):
        value = self.manifest()
        value["frontier"].append({"key": "WORKFLOW-DELIVERY-QUALITY",
            "unblocks": ["keep-other", "dat-workflow-gate-pilot-b3s"], "completion": {
            "post_completion": {"unblocks_issue_ids": ["keep-other", "dat-workflow-gate-pilot-b3s"]}}})
        proposed = completion_proposal(value, "a" * 40)
        self.assertEqual(proposed["frontier"][-1]["completion"]["post_completion"]["unblocks_issue_ids"],
                         ["keep-other"])
        self.assertEqual(proposed["frontier"][-1]["unblocks"], ["keep-other"])

    def test_existing_or_in_repository_output_refuses_without_writes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for target in (root, root / "output"):
                with self.assertRaises(ValueError):
                    prepare(root, target)
            self.assertEqual(list(root.iterdir()), [])


class OwnerHookTests(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.close)
        self.external = tempfile.TemporaryDirectory()
        self.addCleanup(self.external.cleanup)
        self.runner = Path(self.external.name) / "check_workflow_delivery.py"
        self.hook = Path(__file__).resolve().parents[1] / "docs/reviews/workflow-delivery-pilot/owner-pre-commit.sh"
        for name, filename in (("lane", "check_file_lane_ownership.py"), ("format", "check_rustfmt.py")):
            self.f.write("scripts/" + filename, self.recorder(name))
        self.runner.write_bytes(self.recorder("enforce"))

    @staticmethod
    def recorder(name, exit_code=0):
        return ("from pathlib import Path\nimport sys\n"
                "assert '--staged' in sys.argv\n"
                + ("assert '--enforce' in sys.argv and '--report-only' not in sys.argv\n" if name == "enforce" else "")
                + f"with Path('calls.log').open('a') as out: out.write({name!r}+'\\n')\n"
                + f"raise SystemExit({exit_code})\n").encode()

    def configure(self):
        for key, value in {"AuthorityRef": "a" * 40, "BaseRef": "b" * 40,
                           "EnvironmentPath": "evidence/environment.json",
                           "RunnerPath": str(self.runner)}.items():
            self.f.git("config", "--local", "datum.workflowDelivery" + key, value)

    def run_hook(self):
        return subprocess.run(["bash", str(self.hook)], cwd=self.f.root, capture_output=True)

    def test_missing_trust_never_falls_back_to_report_only(self):
        result = self.run_hook()
        self.assertEqual(result.returncode, 2)
        self.assertIn(b"WDQ-TRUST", result.stderr)
        self.assertFalse((self.f.root / "calls.log").exists())

    def test_mid_preparation_config_index_and_worktree_drift_refuse(self):
        self.f.git("add", "scripts")
        self.f.git("commit", "-m", "test fixture")
        for change in ("config", "index", "worktree"):
            with self.subTest(change=change):
                before = live_state(self.f.root)
                require_unchanged(self.f.root, before)
                if change == "config":
                    self.f.git("config", "--local", "datum.testDrift", "true")
                else:
                    self.f.write("drift-" + change, b"changed")
                    if change == "index":
                        self.f.git("add", "drift-index")
                with self.assertRaisesRegex(ValueError, "changed during preparation"):
                    require_unchanged(self.f.root, before)

    def test_candidate_runner_and_moving_ref_refuse(self):
        self.configure()
        self.f.git("config", "--local", "datum.workflowDeliveryRunnerPath",
                   str(self.f.root / "scripts/check_workflow_delivery.py"))
        self.f.write("scripts/check_workflow_delivery.py", self.recorder("enforce"))
        self.assertEqual(self.run_hook().returncode, 2)
        self.configure()
        self.f.git("config", "--local", "datum.workflowDeliveryAuthorityRef", "HEAD")
        self.assertEqual(self.run_hook().returncode, 2)
        self.assertFalse((self.f.root / "calls.log").exists())

    def test_existing_gates_precede_blocking_external_check(self):
        self.configure()
        self.assertEqual(self.run_hook().returncode, 0)
        self.assertEqual((self.f.root / "calls.log").read_text().splitlines(), ["lane", "format", "enforce"])

    def test_lane_refusal_stops_before_format_or_delivery(self):
        self.configure()
        self.f.write("scripts/check_file_lane_ownership.py", self.recorder("lane", 1))
        self.assertEqual(self.run_hook().returncode, 1)
        self.assertEqual((self.f.root / "calls.log").read_text().splitlines(), ["lane"])

    def test_format_and_external_enforcement_failures_propagate(self):
        self.configure()
        self.f.write("scripts/check_rustfmt.py", self.recorder("format", 1))
        self.assertEqual(self.run_hook().returncode, 1)
        self.assertEqual((self.f.root / "calls.log").read_text().splitlines(), ["lane", "format"])
        self.f.write("scripts/check_rustfmt.py", self.recorder("format"))
        self.runner.write_bytes(self.recorder("enforce", 2))
        self.assertEqual(self.run_hook().returncode, 2)
        self.assertEqual((self.f.root / "calls.log").read_text().splitlines()[-3:], ["lane", "format", "enforce"])

    def test_real_external_validator_refuses_candidate_and_runner_drift(self):
        from workflow_delivery_native_test_support import environment
        from workflow_delivery_trust_test_support import accepted_fixture
        _, _, _, authority = accepted_fixture(self.f)
        self.f.save("requested-environment.json", environment())
        scripts = Path(self.external.name) / "scripts"
        scripts.mkdir()
        for source in Path(__file__).resolve().parent.glob("*.py"):
            shutil.copyfile(source, scripts / source.name)
        self.runner = scripts / "check_workflow_delivery.py"
        self.configure()
        for key in ("AuthorityRef", "BaseRef"):
            self.f.git("config", "--local", "datum.workflowDelivery" + key, authority)
        self.f.git("config", "--local", "datum.workflowDeliveryEnvironmentPath", "requested-environment.json")
        self.f.git("add", "requested-environment.json")
        result = self.run_hook()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        original = (self.f.root / "src/read.py").read_bytes()
        self.f.write("src/read.py", b"untested candidate")
        self.f.git("add", "src/read.py")
        result = self.run_hook()
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn(b"WDQ-INDEX", result.stdout)
        self.f.write("src/read.py", original)
        self.f.git("add", "src/read.py")
        self.runner.write_bytes(self.runner.read_bytes() + b"\n# unpromoted runner drift\n")
        result = self.run_hook()
        self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
        self.assertIn(b"WDQ-TRUST", result.stdout)


if __name__ == "__main__":
    unittest.main()
