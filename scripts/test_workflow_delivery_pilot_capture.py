"""Bounded capture tooling tests; these do not establish native pilot acceptance."""

import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from workflow_delivery_pilot_capture import Capture, OBJECT, files
from workflow_delivery_pilot_scenarios import actions
from workflow_delivery_pilot_build import build_command, snapshot_inputs
from workflow_delivery_pilot_observations import flag, records, native_menu_events_present


class CaptureTests(unittest.TestCase):
    def test_monitor_error_or_startup_alone_is_not_native_menu_event_evidence(self):
        for raw in ["Error: Destination is not specified", "member=NameAcquired",
                    "interface=org.a11y.atspi.Event.Object; member=ChildrenChanged"]:
            self.assertFalse(native_menu_events_present(raw))
        self.assertTrue(native_menu_events_present(
            "interface=org.a11y.atspi.Event.Object; member=ChildrenChanged\n"
            "path=/org/a11y/atspi/accessible/menus/n123; "
            "interface=org.a11y.atspi.Event.Object; member=StateChanged"))

    def test_dead_event_monitor_blocks_input_instead_of_producing_silent_evidence(self):
        capture = Capture.__new__(Capture)
        capture.monitor = SimpleNamespace(poll=lambda: 1)
        with self.assertRaisesRegex(RuntimeError, "event monitor stopped"):
            capture.step(["click", 1, 1])

    def test_accessible_states_parse_word_bits_not_the_uint32_type_name(self):
        node = {"states": "([uint32 4096, 0],)"}
        self.assertTrue(flag(node, 12))
        self.assertFalse(flag(node, 8))
        self.assertFalse(flag(node, 24))

    def test_trace_reader_preserves_noninvocation_and_does_not_infer_success(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            event = {"event": "dispatch", "value": {"enabled": True, "invoked": False}}
            (root / "gui-1.log").write_text("other log\nDATUM_ACTION_EVIDENCE " + json.dumps(event) + "\n")
            self.assertEqual(records(root), [event])

    def test_target_is_owned_by_guard_not_a_later_cargo_override(self):
        command = build_command(Path("/owned/source"), Path("/owned/target"))
        boundary = command.index("--")
        self.assertLess(command.index("--target-dir"), boundary)
        self.assertNotIn("--target-dir", command[boundary + 1:])
        self.assertEqual(command[command.index("--target-dir") + 1], "/owned/target")

    def test_snapshot_manifest_includes_untracked_input_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "crates").mkdir()
            (root / "crates/extra.rs").write_text("uncommitted")
            (root / "Cargo.toml").write_text("manifest")
            manifest = snapshot_inputs(root, ["crates", "Cargo.toml"])
            self.assertEqual([item["path"] for item in manifest], ["Cargo.toml", "crates/extra.rs"])

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
        allowed = {"click", "move", "key", "wheel", "capture", "close", "reopen", "focus-window"}
        for scenario in [f"PILOT-S0{number}" for number in range(1, 6)] + ["PILOT-S03-pointer-regression"]:
            steps = actions(scenario)
            self.assertTrue(all(step[0] in allowed for step in steps))
            captures = [step[1] for step in steps if step[0] == "capture"]
            self.assertTrue(captures)
            self.assertEqual(len(captures), len(set(captures)))
        with self.assertRaises(ValueError):
            actions("unreviewed")


if __name__ == "__main__":
    unittest.main()
