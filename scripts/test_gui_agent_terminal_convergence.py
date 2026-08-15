#!/usr/bin/env python3
"""Hermetic regressions for the terminal convergence guard."""

from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("check_gui_agent_terminal_convergence.py")
SPEC = importlib.util.spec_from_file_location("terminal_convergence", MODULE_PATH)
assert SPEC and SPEC.loader
guard = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(guard)


class TerminalGridWriterGuardTest(unittest.TestCase):
    def test_only_protocol_declaration_terminal_core_and_tests_may_mutate_grid(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            files = {
                "crates/gui-protocol/src/terminal_lane.rs": "fn pty_grid_mut() {}",
                "crates/gui-app/src/terminal_screen.rs": "state.pty_grid_mut();",
                "crates/gui-app/src/terminal_screen/parser.rs": "state.pty_grid_mut();",
                "crates/gui-render/src/terminal_contract_tests.rs": "state.pty_grid_mut();",
                "crates/gui-app/src/rogue_writer.rs": "state.pty_grid_mut();",
            }
            for relative, source in files.items():
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(source, encoding="utf-8")
            previous_root = guard.ROOT
            guard.ROOT = root
            try:
                failures: list[str] = []
                guard.check_terminal_grid_writers(failures)
            finally:
                guard.ROOT = previous_root
            self.assertEqual(
                ["terminal grid mutation escaped PTY interpretation: "
                 "crates/gui-app/src/rogue_writer.rs"],
                failures,
            )


if __name__ == "__main__":
    unittest.main()
