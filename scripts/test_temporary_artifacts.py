#!/usr/bin/env python3
"""Exercise proof-output enforcement against a disposable Git index."""

from pathlib import Path
import subprocess
import sys
import tempfile
import json
import unittest


GATE = Path(__file__).with_name("check_temporary_artifacts.py").resolve()
ROOT = GATE.parent.parent


class TemporaryArtifactTests(unittest.TestCase):
    def test_new_raw_counters_refused_and_historical_receipts_remain_pinned(self) -> None:
        path = "docs/reviews/trial/counters.json"
        self.stage(path)
        target = self.root / path
        value = {"samples": [{"raw_counters": [{"seconds": 0, "cpu_ticks": 10}]}]}
        target.write_text(json.dumps(value))
        self.git("add", "-f", "--", path)
        self.assertEqual(self.run_gate("--staged").returncode, 1)
        self.assertEqual(self.run_gate().returncode, 1)
        self.git("commit", "-qm", "historical fixture")
        self.assertEqual(self.run_gate().returncode, 0)
        value["new_finding"] = "changed record"
        target.write_text(json.dumps(value))
        self.git("add", "--", path)
        before = self.git("write-tree")
        self.assertEqual(self.run_gate("--staged").returncode, 1)
        self.assertEqual(before, self.git("write-tree"))
        target.write_text('{"raw_sample_retirement":{"raw_samples_available":false}}')
        self.git("add", "--", path)
        self.assertEqual(self.run_gate("--staged").returncode, 0)

    def test_capture_member_inventory_is_temporary_but_summary_is_allowed(self) -> None:
        path = "docs/reviews/trial/archive-members.json"
        self.stage(path)
        target = self.root / path
        target.write_text('{"capture_files":["trial/env.json","trial/stdout.txt"]}')
        self.git("add", "--", path)
        before = self.git("write-tree")
        self.assertEqual(self.run_gate("--staged").returncode, 1)
        self.assertEqual(before, self.git("write-tree"))
        self.git("commit", "-qm", "historical fixture")
        self.assertEqual(self.run_gate().returncode, 0)
        target.write_text('{"capture_file_count":2,"raw_source_commit":"historical"}')
        self.git("add", "--", path)
        self.assertEqual(self.run_gate("--staged").returncode, 0)

    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.git("init", "-q")
        self.git("config", "user.name", "Datum Test")
        self.git("config", "user.email", "datum-test@example.invalid")
        (self.root / ".gitignore").write_bytes((ROOT / ".gitignore").read_bytes())
        self.git("add", "--", ".gitignore")
        self.git("commit", "-qm", "fixture")

    def git(self, *args: str) -> bytes:
        return subprocess.check_output(["git", *args], cwd=self.root, stderr=subprocess.PIPE)

    def stage(self, path: str) -> None:
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("test capture", encoding="utf-8")
        self.git("add", "-f", "--", path)

    def run_gate(self, *args: str) -> subprocess.CompletedProcess:
        return subprocess.run(
            [sys.executable, str(GATE), *args], cwd=self.root,
            capture_output=True, check=False,
        )

    def test_forced_proof_adds_refuse_without_mutating_index(self) -> None:
        paths = (
            "docs/reviews/trial/a capture μ\n.png",
            "docs/reviews/trial/CAPTURE.PNG",
            "research/process-quality/evidence/trial/screen.png",
            "crates/test-harness/testdata/proof.png",
            "crates/gui-render/testdata/golden/revision/running-app/shell.png",
            "elsewhere/renamed-proof.tar.xz",
            "elsewhere/profile.txt.gz",
            "docs/new-proof-location/capture.png",
            "crates/gui-render/testdata/golden/arbitrary-proof.png",
            "elsewhere/capture.avif",
            "elsewhere/capture.bmp",
            "elsewhere/raw.bz2",
            "elsewhere/raw.zst",
            "elsewhere/raw.rar",
            "docs/reviews/new-trial/stdout.log",
            "docs/reviews/new-trial/frames.jsonl",
            "docs/reviews/new-trial/capture-atspi.json",
            "docs/reviews/workflow-delivery-pilot/trial/selected-state.json",
        )
        for path in paths:
            self.stage(path)
        before = self.git("write-tree")
        result = self.run_gate("--staged")
        self.assertEqual(result.returncode, 1, result.stdout)
        for path in paths:
            self.assertIn(repr(path).encode(), result.stdout)
        self.assertEqual(before, self.git("write-tree"))

    def test_review_records_and_test_inputs_pass(self) -> None:
        for path in (
            "docs/reviews/trial/result.json",
            "docs/reviews/trial/findings.md",
            "crates/gui-render/testdata/golden/shell/datum-shell.golden.png",
            "docs/gui/reference/board-editor.png",
            "crates/gui-render/assets/logo.png",
            "crates/test-harness/testdata/terminal/transcript.log",
        ):
            self.stage(path)
        self.assertEqual(self.run_gate("--staged").returncode, 0)
        self.assertEqual(self.run_gate().returncode, 0)

    def test_existing_archives_fail_inventory_but_deletion_passes(self) -> None:
        path = "docs/reviews/trial/raw.tar.gz"
        self.stage(path)
        self.git("commit", "-qm", "historical fixture")
        self.assertEqual(self.run_gate().returncode, 1)
        self.git("rm", "--", path)
        self.assertEqual(self.run_gate("--staged").returncode, 0)
        self.assertEqual(self.run_gate().returncode, 0)

    def test_ignored_capture_and_rename_into_proof_location(self) -> None:
        path = "docs/reviews/trial/capture.png"
        self.assertEqual(self.git("check-ignore", "--", path).strip().decode(), path)
        source = "crates/gui-render/assets/product.png"
        self.stage(source)
        self.git("commit", "-qm", "product fixture")
        (self.root / path).parent.mkdir(parents=True, exist_ok=True)
        self.git("mv", "--", source, path)
        self.assertEqual(self.run_gate("--staged").returncode, 1)

    def test_raw_samples_cannot_hide_in_a_review_json_record(self) -> None:
        path = "docs/reviews/trial/result.JSON"
        self.stage(path)
        target = self.root / path
        target.write_text('{"nested":{"drm_samples":[{"descriptors":["raw fd data"]}]}}')
        self.git("add", "-f", "--", path)
        before = self.git("write-tree")
        self.assertEqual(self.run_gate("--staged").returncode, 1)
        self.assertEqual(self.run_gate().returncode, 1)
        self.assertEqual(before, self.git("write-tree"))
        target.write_text('{"raw_sample_retirement":{"raw_samples_available":false,"sample_count":1}}')
        self.git("add", "--", path)
        self.assertEqual(self.run_gate("--staged").returncode, 0)


if __name__ == "__main__":
    unittest.main()
