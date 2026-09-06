"""Bounded capture tooling tests; these do not establish native pilot acceptance."""

import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from workflow_delivery_pilot_capture import Capture, OBJECT, files
from workflow_delivery_pilot_scenarios import actions


class CaptureTests(unittest.TestCase):
    def test_gvariant_child_references_keep_unannotated_siblings(self):
        raw = "([(':1.1', objectpath '/root'), (':1.1', '/menu')],)"
        self.assertEqual(OBJECT.findall(raw), [(":1.1", "/root"), (":1.1", "/menu")])

    def test_fixture_is_copied_to_owned_scratch_without_reusing_output(self):
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            (base / "scratch").mkdir()
            args = SimpleNamespace(output=base / "output", cli=Path("target/debug/datum-eda"))
            with patch("workflow_delivery_pilot_capture.tempfile.mkdtemp", return_value=str(base / "scratch")):
                capture = Capture(args)
            self.assertEqual(capture.before, files(capture.project))
            self.assertIn("project.json", capture.before)
            self.assertIn(".datum/journal/transactions.jsonl", capture.before)
            self.assertNotIn("DISPLAY", capture.env)
            self.assertNotIn("WAYLAND_DISPLAY", capture.env)
            self.assertEqual(capture.env["DATUM_ACTION_EVIDENCE"], "1")
            with self.assertRaises(FileExistsError):
                Capture(args)

    def test_finish_reports_new_and_changed_files_without_blessing_them(self):
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            (base / "scratch").mkdir()
            args = SimpleNamespace(output=base / "output", cli=Path("target/debug/datum-eda"))
            with patch("workflow_delivery_pilot_capture.tempfile.mkdtemp", return_value=str(base / "scratch")):
                capture = Capture(args)
            (capture.project / "project.json").write_text("changed")
            (capture.project / "unexpected.json").write_text("new")
            capture.finish()
            result = json.loads((capture.out / "source-diff.json").read_text())
            self.assertEqual(result["changed_or_removed"], ["project.json"])
            self.assertIn("unexpected.json", result["new_paths"])

    def test_scripts_have_native_inputs_and_unique_observation_names(self):
        allowed = {"click", "move", "key", "wheel", "capture", "close", "reopen"}
        for number in range(1, 6):
            steps = actions(f"PILOT-S0{number}")
            self.assertTrue(all(step[0] in allowed for step in steps))
            captures = [step[1] for step in steps if step[0] == "capture"]
            self.assertTrue(captures)
            self.assertEqual(len(captures), len(set(captures)))
        with self.assertRaises(ValueError):
            actions("unreviewed")


if __name__ == "__main__":
    unittest.main()
