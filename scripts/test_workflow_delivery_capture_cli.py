"""Actual user-facing capture command; failures remain retained and non-accepting."""

import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from workflow_delivery_capture_fixture import prepare_fixture
from workflow_delivery_io import sha256
from workflow_delivery_test_support import Fixture


SCRIPT = Path(__file__).with_name("workflow_delivery_capture_cli.py")


class CaptureCliTest(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.packet = self.root / "packet"
        prepare_fixture(self.root / "baseline", self.packet)
        self.digest = sha256((self.packet / "recipe.json").read_bytes())
        self.runtime = Fixture(self.root / "runtime")
        for source in SCRIPT.parent.iterdir():
            if source.is_file() and source.suffix in (".py", ".sh"):
                self.runtime.write("scripts/" + source.name, source.read_bytes())
        self.runtime.stage()
        self.runtime.git("commit", "-qm", "test(workflow): retain isolated capture CLI runtime\n\nSynthetic test source only.")
        self.script = self.runtime.root / "scripts" / SCRIPT.name
        self.output = self.runtime.root / ".git/datum-wdq/proposals"

    def invoke(self, name, case="INFRA-S01-01", surface="C", digest=None, isolated=True):
        command = [sys.executable, *(["-I", "-S", "-B"] if isolated else ["-B"]), str(self.script),
                   "--capture", "--packet", str(self.packet), "--recipe-sha256", digest or self.digest,
                   "--destination", str(self.output / name), "--case", case, "--surface", surface]
        process = subprocess.run(command, capture_output=True, check=False)
        return process.returncode, json.loads(process.stdout)

    def test_actual_cli_staged_hook_and_candidate_history_results_are_retained(self):
        for name, case, surface in (("staged", "INFRA-S01-01", "C"),
                                    ("hook", "INFRA-S05-01.staged-bad", "H"),
                                    ("history", "INFRA-S05-07.add-delete", "R")):
            with self.subTest(surface=surface):
                status, report = self.invoke(name, case, surface)
                self.assertEqual(0, status, report)
                self.assertTrue(report["ok"])
                self.assertTrue(report["assessment"]["observation_matches_expected"])
                for key in ("complete_matrix", "input_closure_verified", "readiness_asserted", "acceptance_asserted"):
                    self.assertFalse(report[key])
                raw = (self.output / name / "driver-result.json").read_bytes()
                self.assertEqual(sha256(raw), report["driver_result_sha256"])
                self.assertTrue((self.output / name / "case/capture/stdout.bin").is_file())

    def test_bad_recipe_and_nonisolated_launch_refuse_before_creating_output(self):
        for name, options in (("bad-hash", {"digest": "0" * 64}),
                              ("nonisolated", {"isolated": False})):
            status, report = self.invoke(name, **options)
            self.assertEqual(2, status)
            self.assertFalse(report["ok"])
            self.assertFalse((self.output / name).exists())

    def test_failed_case_is_retained_and_repeated_destination_is_not_overwritten(self):
        status, report = self.invoke("failed", case="UNIMPLEMENTED")
        self.assertEqual(2, status)
        result = self.output / "failed/driver-result.json"
        original = result.read_bytes()
        self.assertFalse(json.loads(original)["ok"])
        status, second = self.invoke("failed")
        self.assertEqual(2, status)
        self.assertEqual(original, result.read_bytes())
        self.assertIn("fresh canonical destination", second["findings"][0]["detail"])

    def test_invalid_bundle_subprocess_failure_is_retained_not_reported_as_refusal_proof(self):
        bundle = self.packet / "fixture.bundle"
        bundle.write_bytes(b"not a Git bundle\n")
        recipe_path = self.packet / "recipe.json"
        recipe = json.loads(recipe_path.read_bytes())
        recipe["bundle"]["sha256"] = sha256(bundle.read_bytes())
        recipe_path.write_text(json.dumps(recipe))
        self.digest = sha256(recipe_path.read_bytes())
        status, report = self.invoke("bad-bundle")
        self.assertEqual(2, status)
        self.assertFalse(report["ok"])
        self.assertEqual("CalledProcessError", report["findings"][0]["type"])
        self.assertNotIn("assessment", report)
        self.assertTrue((self.output / "bad-bundle/driver-result.json").is_file())

    def test_worktree_documents_and_trusted_support_destinations_refuse_without_writes(self):
        for target in (self.root / "Documents-run", self.runtime.root / "capture-output",
                       self.runtime.root / ".git/datum-wdq/trusted/run",
                       self.output / "nested/run"):
            with self.subTest(destination=target):
                status, report = self.invoke(target)
                self.assertEqual(2, status)
                self.assertIn("directly under Git-common", report["findings"][0]["detail"])
                self.assertFalse(target.exists())
        self.assertEqual(b"", self.runtime.git("status", "--porcelain"))

    def test_redirected_proposal_store_refuses_without_following_symlink(self):
        self.output.parent.mkdir(parents=True)
        redirected = self.root / "redirected"
        redirected.mkdir()
        self.output.symlink_to(redirected, target_is_directory=True)
        status, report = self.invoke("unsafe")
        self.assertEqual(2, status)
        self.assertIn("symlink", report["findings"][0]["detail"])
        self.assertEqual([], list(redirected.iterdir()))

    def test_linked_runtime_uses_the_shared_git_common_proposal_store(self):
        linked = self.root / "linked-runtime"
        self.runtime.git("worktree", "add", "--detach", str(linked), "HEAD")
        self.script = linked / "scripts" / SCRIPT.name
        status, report = self.invoke("linked-capture")
        self.assertEqual(0, status, report)
        self.assertEqual(str(self.output / "linked-capture"), report["destination"])
        driver = json.loads((self.output / "linked-capture/driver-input.json").read_bytes())
        self.assertEqual(str(linked), driver["runtime_root"])
        self.assertFalse((linked / "datum-wdq").exists())


if __name__ == "__main__":
    unittest.main()
