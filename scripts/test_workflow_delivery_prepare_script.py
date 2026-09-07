"""Test runnable owner handoff in disposable repositories, never real promotion."""

from pathlib import Path
import subprocess
import tempfile
import unittest

from workflow_delivery_prepare import live_state
from workflow_delivery_prepare_handoff import promotion_markdown
from workflow_delivery_prepare_script import activation_script
from workflow_delivery_test_support import Fixture


class ScriptTests(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.close)
        self.output = tempfile.TemporaryDirectory(prefix="wdq owner script ")
        self.addCleanup(self.output.cleanup)
        self.script = Path(self.output.name) / "activate.py"
        self.result = {"candidate": "a" * 40, "base": "b" * 40,
                       "runner": str(Path(self.output.name) / "candidate/scripts/check_workflow_delivery.py"),
                       "hooks": str(Path(self.output.name) / "hooks"), "environment": "env.json",
                       "packet_sha256": "c" * 64, "review_sha256": "d" * 64}
        self.script.write_text(activation_script(self.f.root, self.result))

    def run_script(self, *args):
        return subprocess.run(["python3", str(self.script), *args], capture_output=True, text=True)

    def test_default_help_and_invalid_arguments_do_not_activate_or_log(self):
        before = live_state(self.f.root), self.f.snapshot()
        for args, expected in (([], 0), (["--help"], 0), (["--force"], 2)):
            result = self.run_script(*args)
            self.assertEqual(result.returncode, expected)
            self.assertIn("No activation performed", result.stdout)
        self.assertEqual(list(Path(self.output.name).glob("activation-*.log")), [])
        self.assertEqual(before, (live_state(self.f.root), self.f.snapshot()))

    def test_obsolete_base_is_visible_logged_and_returns_failure(self):
        before = live_state(self.f.root), self.f.snapshot()
        result = self.run_script("--activate")
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("WDQ activation stopped", result.stdout)
        self.assertIn("exit code 1", result.stdout)
        logs = list(Path(self.output.name).glob("activation-*.log"))
        self.assertEqual(len(logs), 1)
        self.assertEqual(logs[0].stat().st_mode & 0o777, 0o600)
        self.assertIn("WDQ activation stopped", logs[0].read_text())
        self.assertNotIn("validating the exact candidate", logs[0].read_text())
        self.assertEqual(before, (live_state(self.f.root), self.f.snapshot()))

    def test_script_and_markdown_use_identical_commands(self):
        namespace = {"__name__": "fixture_not_main"}
        exec(compile(self.script.read_text(), str(self.script), "exec"), namespace)
        block = promotion_markdown(self.f.root, self.result).split("```bash\n")[2].split("```")[0]
        self.assertEqual(namespace["COMMANDS"], block)


if __name__ == "__main__":
    unittest.main()
