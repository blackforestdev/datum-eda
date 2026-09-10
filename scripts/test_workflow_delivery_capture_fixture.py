"""Run actual CLI/selector processes against retained synthetic Git inputs."""

import json
import os
from pathlib import Path
import sys
import tempfile
import unittest

from workflow_delivery_capture_command import capture_command
from workflow_delivery_capture_fixture import prepare_fixture, restore_fixture
from workflow_delivery_io import sha256


class CaptureFixtureTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.store = Path(self.temp.name)
        self.root, self.packet = self.store / "fixture", self.store / "packet"
        self.recipe = prepare_fixture(self.root, self.packet)
        self.env = {"PATH": os.defpath, "HOME": str(self.store), "XDG_CONFIG_HOME": str(self.store),
                    "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull,
                    "PYTHONDONTWRITEBYTECODE": "1", "LANG": "C"}

    def capture(self, script, args, surface, suffix):
        observer = Path(__file__).with_name("workflow_delivery_observe_python.py")
        out = self.store / suffix
        command = [sys.executable, "-I", "-S", "-B", str(observer),
            "--script", str(self.root / "scripts" / script), "--source-root", str(self.root),
            "--output", str(out / "calls.jsonl"), "--function", "main",
            "--function", "selector_failures", "--", *args]
        result = capture_command(self.root, out, command, self.env,
            fixture_nonce=self.recipe["fixture_nonce"], case_id="synthetic-baseline", surface=surface,
            untracked_roots=["src"], timeout=30)
        self.assertEqual("exited", result["kind"], result)
        self.assertEqual(0, result["returncode"], (out / "stderr.bin").read_text())
        self.assertEqual([], result["changed_state_fields"])
        return out

    def test_retained_baseline_runs_cli_and_selector_with_real_observations(self):
        authority = self.recipe["head"]
        out = self.capture("check_workflow_delivery.py", ["--root", str(self.root), "--enforce",
            "--staged", "--authority-ref", authority, "--base-ref", authority,
            "--environment-path", "requested-environment.json"], "C", "cli")
        report = json.loads((out / "stdout.bin").read_text())
        self.assertEqual(["TASK: accept"], report["checks"])
        self.assertFalse(report["acceptance_asserted"])
        out = self.capture("project_status.py", ["check"], "S", "selector")
        calls = [json.loads(line) for line in (out / "calls.jsonl").read_text().splitlines()]
        self.assertTrue(any(row.get("function") == "selector_failures" for row in calls))

    def test_bundle_restores_exact_history_and_tracked_file_bytes(self):
        bundle = self.packet / self.recipe["bundle"]["path"]
        self.assertEqual(self.recipe["bundle"]["sha256"], sha256(bundle.read_bytes()))
        restored = self.store / "restored"
        self.assertEqual(self.recipe, restore_fixture(self.packet, restored))
        state = json.loads((self.packet / "prepared-state.json").read_text())
        for path, record in state["files"].items():
            if record["kind"] == "file":
                self.assertEqual(record["sha256"], sha256((restored / path).read_bytes()), path)
        self.root = restored
        self.capture("project_status.py", ["check"], "S", "restored-selector")

    def test_tampered_packet_refuses_before_restoration(self):
        (self.packet / "fixture.bundle").write_bytes(b"tampered")
        restored = self.store / "refused"
        with self.assertRaisesRegex(ValueError, "hash mismatch"):
            restore_fixture(self.packet, restored)
        self.assertFalse(restored.exists())


if __name__ == "__main__":
    unittest.main()
