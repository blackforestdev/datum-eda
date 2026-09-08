"""Execute the prepared owner hook in fixtures only; never install on Datum."""

import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

from workflow_delivery_capture_fixture import prepare_fixture
from workflow_delivery_capture_state import git
from workflow_delivery_support_bundle import prepare_bundle, verify_bundle
from workflow_delivery_support_store import support_locations


class OwnerHookTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.store = Path(self.temp.name)
        self.root = self.store / "fixture"
        self.recipe = prepare_fixture(self.root, self.store / "packet")
        self.authority = self.recipe["head"]
        self.locations = support_locations(self.root, self.authority)
        prepare_bundle(self.root, self.authority)
        git(self.root, "config", "--local", "datum.workflowDeliveryRunnerPath", str(self.locations["runner"]))
        git(self.root, "config", "--local", "core.hooksPath", str(self.locations["hooks"]))
        self.environment = {"PATH": os.defpath, "HOME": str(self.store), "XDG_CONFIG_HOME": str(self.store),
            "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}

    def hook(self, environment=None):
        return subprocess.run([str(self.locations["hook"])], cwd=self.root,
            env=environment or self.environment, capture_output=True, timeout=30)

    def test_real_hook_runs_both_earlier_gates_and_enforcement(self):
        result = self.hook()
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertIn(b"visual-truth file-lane gate passed", result.stdout)
        self.assertIn(b"rustfmt gate passed", result.stdout)
        report = json.loads(result.stdout.splitlines()[-1])
        self.assertEqual(["TASK: accept"], report["checks"])
        self.assertFalse(report["acceptance_asserted"])
        verify_bundle(self.root, self.authority)

    def test_git_commit_with_alternate_unscoped_index_is_refused(self):
        alternate = self.root / ".git/alternate-index"
        alternate.write_bytes((self.root / ".git/index").read_bytes())
        (self.root / "src/unscoped.py").write_text("# unpermitted alternate-index source\n")
        environment = dict(self.environment, GIT_INDEX_FILE=str(alternate))
        subprocess.run(["git", "add", "--", "src/unscoped.py"], cwd=self.root, env=environment, check=True)
        self.assertEqual(0, self.hook().returncode)
        result = subprocess.run(["git", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
            "commit", "-m", "test(workflow): attempt unpermitted fixture commit\n\nMust be refused by the real hook."],
            cwd=self.root, env=environment, capture_output=True, timeout=30)
        self.assertNotEqual(0, result.returncode)
        self.assertIn(b"WDQ-COVERAGE", result.stdout + result.stderr)
        self.assertEqual(self.authority, git(self.root, "rev-parse", "HEAD").decode().strip())

    def test_modified_bootstrap_is_refused_before_it_executes(self):
        bootstrap = self.locations["scripts"] / "workflow_delivery_bootstrap.py"
        bootstrap.chmod(0o644)
        bootstrap.write_text('print("UNTRUSTED BOOTSTRAP EXECUTED")\n')
        bootstrap.chmod(0o444)
        result = self.hook()
        self.assertEqual(2, result.returncode)
        self.assertIn(b"changed pinned scripts/workflow_delivery_bootstrap.py", result.stderr)
        self.assertNotIn(b"UNTRUSTED BOOTSTRAP EXECUTED", result.stdout)

    def test_modified_runner_is_refused_before_it_executes(self):
        runner = self.locations["runner"]
        runner.chmod(0o644)
        runner.write_text('print("UNTRUSTED RUNNER EXECUTED")\n')
        runner.chmod(0o444)
        result = self.hook()
        self.assertEqual(2, result.returncode)
        self.assertIn(b"pinned Git authority", result.stderr)
        self.assertNotIn(b"UNTRUSTED RUNNER EXECUTED", result.stdout)

    def test_candidate_prerequisite_gate_change_is_not_executed(self):
        (self.root / "scripts/check_file_lane_ownership.py").write_text('print("UNTRUSTED PRECHECK EXECUTED")\n')
        result = self.hook()
        self.assertEqual(2, result.returncode)
        self.assertIn(b"candidate prerequisite gate differs", result.stderr)
        self.assertNotIn(b"UNTRUSTED PRECHECK EXECUTED", result.stdout)

    def test_missing_owner_selection_has_visible_failure(self):
        git(self.root, "config", "--local", "--unset", "datum.workflowDeliveryRunnerPath")
        result = self.hook()
        self.assertEqual(2, result.returncode)
        self.assertIn(b"WDQ-TRUST: missing runner", result.stderr)

    def test_foreign_repository_environment_is_not_silently_substituted(self):
        result = self.hook(dict(self.environment, GIT_DIR=str(self.store / "missing-foreign-git")))
        self.assertEqual(2, result.returncode)
        self.assertIn(b"Git directory differs", result.stderr)


if __name__ == "__main__":
    unittest.main()
