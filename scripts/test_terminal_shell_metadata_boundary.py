#!/usr/bin/env python3
"""Hermetic mutations for the terminal shell-metadata boundary."""

import importlib.util
from pathlib import Path
import unittest

MODULE = Path(__file__).with_name("check_terminal_shell_metadata_boundary.py")
SPEC = importlib.util.spec_from_file_location("terminal_shell_metadata", MODULE)
assert SPEC and SPEC.loader
guard = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(guard)


class TerminalShellMetadataBoundaryTests(unittest.TestCase):
    def test_valid_projection_passes_and_authority_mutations_fail(self) -> None:
        lane = """
struct TerminalShellMetadata;
enum TerminalShellPhase {}
self.event_sequence.saturating_add(1);
shell_metadata;
"""
        reducer = """
CoreEvent::WorkingDirectoryChanged(directory);
CoreEvent::ShellMark(mark);
lane.shell_metadata.observe(mark);
record_terminal_shell_metadata_event();
"""
        events = '''
event: "terminal_shell_metadata",
origin: "pty_osc",
trust: "untrusted",
'''
        tests = "fn osc7_and_osc133_are_untrusted_session_metadata_not_design_authority() {}"
        self.assertEqual([], guard.check_sources(lane, reducer, events, tests))

        failures = guard.check_sources(
            lane.replace("event_sequence.saturating_add(1)", "event_sequence += 1"),
            reducer.replace(
                "record_terminal_shell_metadata_event();",
                "prepare_terminal_command_execution(); Operation; approval;",
            ),
            events.replace('trust: "untrusted",', 'trust: "trusted",'),
            tests.replace(
                "osc7_and_osc133_are_untrusted_session_metadata_not_design_authority",
                "removed",
            ),
        )
        self.assertTrue(any("projection is missing" in item for item in failures))
        self.assertTrue(any("audit boundary is missing" in item for item in failures))
        self.assertIn("terminal OSC metadata authority proof is missing", failures)
        self.assertTrue(any("gained authority marker" in item for item in failures))


if __name__ == "__main__":
    unittest.main()
