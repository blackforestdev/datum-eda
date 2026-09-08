"""Run the inspection script; publication and live trust are never changed."""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from workflow_delivery_capture_state import protected_state
from workflow_delivery_io import canonical_json, sha256
from workflow_delivery_test_support import Fixture
from workflow_delivery_trust_test_support import accepted_fixture
from workflow_delivery_support_bundle import prepare_bundle
from workflow_delivery_support_store import support_locations


class PreflightCliTest(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.close)
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.store = Path(self.temp.name)
        _, _, _, self.f.head = accepted_fixture(self.f)
        prepare_bundle(self.f.root, self.f.head)
        tree = self.f.git("rev-parse", "HEAD^{tree}").decode().strip()
        candidate = self.f.git("commit-tree", tree, "-p", self.f.head, "-m", "synthetic candidate").decode().strip()
        self.request = {"schema_version": 1, "kind": "datum.workflow-delivery.preflight-request",
                        "base": self.f.head, "candidate": candidate, "authority": self.f.head, "input_roots": ["src"],
                        "prior_local_trust": protected_state(self.f.root, ["src"])["local_trust"]}
        self.path = self.store / "request.json"
        self.path.write_bytes(canonical_json(self.request))
        self.output = self.f.root / ".git/datum-wdq/logs/preflight.json"
        self.script = Path(__file__).with_name("workflow_delivery_preflight_cli.py")
        self.env = {"PATH": os.defpath, "HOME": str(self.store), "XDG_CONFIG_HOME": str(self.store),
                    "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull}

    def run_cli(self, *, output=None, digest=None, isolated=True, extra=()):
        return subprocess.run([sys.executable, *(["-I", "-S", "-B"] if isolated else []),
            str(self.script), "--inspect", "--root", str(self.f.root), "--request", str(self.path),
            "--request-sha256", digest or sha256(self.path.read_bytes()),
            "--output", str(output or self.output), *extra], cwd=self.store,
            env=self.env, capture_output=True, timeout=30)

    def test_success_retains_exact_log_and_leaves_protected_state_unchanged(self):
        before = protected_state(self.f.root, ["src"])
        result = self.run_cli()
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        report = json.loads(result.stdout)
        self.assertTrue(report["ok"])
        self.assertEqual(sha256(self.output.read_bytes()), report["log_sha256"])
        payload = json.loads(self.output.read_bytes())
        self.assertEqual(self.request, payload["request"])
        self.assertFalse(payload["publication_authorized"])
        self.assertEqual(self.f.head, payload["support_bundle"]["authority"])
        self.assertEqual(before, protected_state(self.f.root, ["src"]))

    def test_dirty_failure_is_visible_and_retained_without_repair(self):
        self.f.write("src/read.py", b"dirty input\n")
        before = protected_state(self.f.root, ["src"])
        result = self.run_cli()
        self.assertEqual(2, result.returncode)
        self.assertIn(b"dirty", result.stdout)
        self.assertFalse(json.loads(self.output.read_bytes())["ok"])
        self.assertEqual(before, protected_state(self.f.root, ["src"]))

    def test_existing_log_is_never_overwritten(self):
        self.assertEqual(0, self.run_cli().returncode)
        original = self.output.read_bytes()
        self.assertEqual(2, self.run_cli().returncode)
        self.assertEqual(original, self.output.read_bytes())

    def test_wrong_digest_external_output_and_nonisolated_python_refuse(self):
        for kwargs in ({"digest": "0" * 64}, {"output": self.store / "outside.json"}, {"isolated": False}):
            with self.subTest(kwargs=kwargs):
                result = self.run_cli(**kwargs)
                self.assertEqual(2, result.returncode)
                self.assertFalse(json.loads(result.stdout)["ok"])
                self.assertFalse(self.output.exists())

    def test_activation_option_is_not_supported(self):
        result = self.run_cli(extra=["--activate"])
        self.assertEqual(2, result.returncode)
        self.assertIn(b"unrecognized arguments", result.stderr)
        self.assertFalse(self.output.exists())

    def test_missing_or_changed_proposed_runner_is_logged_without_repair(self):
        runner = support_locations(self.f.root, self.f.head)["runner"]
        original = runner.read_bytes()
        runner.chmod(0o644)
        runner.write_bytes(original + b"\n# altered proposed runner\n")
        runner.chmod(0o444)
        result = self.run_cli()
        self.assertEqual(2, result.returncode)
        self.assertIn(b"pinned Git authority", result.stdout)
        self.assertEqual(original + b"\n# altered proposed runner\n", runner.read_bytes())
        # This file is solely inside this freshly created fixture bundle.
        runner.unlink()
        result = self.run_cli(output=self.output.with_name("missing-runner.json"))
        self.assertEqual(2, result.returncode)
        self.assertIn(b"runtime module set differs", result.stdout)
        self.assertFalse(runner.exists())

    def test_missing_or_moving_authority_never_falls_back_to_candidate(self):
        for authority in ("HEAD", "0" * 40):
            self.request["authority"] = authority
            self.path.write_bytes(canonical_json(self.request))
            result = self.run_cli()
            self.assertEqual(2, result.returncode)
            self.assertFalse(json.loads(result.stdout)["ok"])
            self.assertFalse(self.output.exists())

    def test_malformed_request_is_rejected_before_log_creation(self):
        for change in ({"schema_version": True}, {"extra": "not allowed"}, {"input_roots": [False]}):
            with self.subTest(change=change):
                self.path.write_bytes(canonical_json(dict(self.request, **change)))
                result = self.run_cli()
                self.assertEqual(2, result.returncode)
                self.assertFalse(json.loads(result.stdout)["ok"])
                self.assertFalse(self.output.exists())

    def test_incomplete_script_copy_returns_structured_import_failure(self):
        copy = self.store / "standalone-preflight.py"
        copy.write_bytes(self.script.read_bytes())
        self.script = copy
        result = self.run_cli()
        self.assertEqual(2, result.returncode)
        report = json.loads(result.stdout)
        self.assertFalse(report["ok"])
        self.assertEqual("ModuleNotFoundError", report["findings"][0]["type"])
        self.assertFalse(self.output.exists())


if __name__ == "__main__":
    unittest.main()
