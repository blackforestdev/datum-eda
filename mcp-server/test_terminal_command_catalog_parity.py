#!/usr/bin/env python3
"""Terminal handoff command catalog parity tests."""

from __future__ import annotations

import re
import unittest
from pathlib import Path

from tool_dispatch import registered_tool_names


ROOT = Path(__file__).resolve().parents[1]
TERMINAL_COMMAND_CATALOG = ROOT / "crates/gui-protocol/src/terminal_command_catalog.rs"


class TestTerminalCommandCatalogParity(unittest.TestCase):
    def test_terminal_handoff_aliases_exist_in_mcp_catalog(self) -> None:
        source = TERMINAL_COMMAND_CATALOG.read_text(encoding="utf-8")
        terminal_command_ids = re.findall(r'entry\(\s*"([^"]+)"', source)
        self.assertGreater(
            len(terminal_command_ids),
            4,
            "terminal handoff catalog should cover more than the first four production commands",
        )
        mcp_tool_names = set(registered_tool_names())
        missing = [
            command_id
            for command_id in terminal_command_ids
            if command_id.startswith("datum.") and command_id not in mcp_tool_names
        ]
        self.assertEqual(missing, [])

    def test_terminal_catalog_advertises_production_proposal_commands(self) -> None:
        source = TERMINAL_COMMAND_CATALOG.read_text(encoding="utf-8")
        required_templates = {
            "datum.proposal.create_output_job": "create-output-job",
            "datum.proposal.update_output_job": "update-output-job",
            "datum.proposal.delete_output_job": "delete-output-job",
            "datum.proposal.create_manufacturing_plan": "create-manufacturing-plan",
            "datum.proposal.update_manufacturing_plan": "update-manufacturing-plan",
            "datum.proposal.delete_manufacturing_plan": "delete-manufacturing-plan",
            "datum.proposal.create_panel_projection": "create-panel-projection",
            "datum.proposal.update_panel_projection": "update-panel-projection",
            "datum.proposal.delete_panel_projection": "delete-panel-projection",
        }
        for command_id, cli_subcommand in required_templates.items():
            with self.subTest(command_id=command_id):
                self.assertIn(f'"{command_id}"', source)
                self.assertIn(f'"{cli_subcommand}"', source)

    def test_terminal_catalog_advertises_canonical_check_and_journal_commands(self) -> None:
        source = TERMINAL_COMMAND_CATALOG.read_text(encoding="utf-8")
        required_templates = {
            "datum.check.run": ("check", "run"),
            "datum.check.list": ("check", "list"),
            "datum.check.show": ("check", "show"),
            "datum.check.profiles": ("check", "profiles"),
            "datum.check.repair_standards": ("check", "repair-standards"),
            "datum.check.fill_zones": ("check", "fill-zones"),
            "datum.check.waive": ("check", "waive"),
            "datum.check.accept_deviation": ("check", "accept-deviation"),
            "datum.journal.list": ("journal", "list"),
            "datum.journal.show": ("journal", "show"),
            "datum.journal.undo": ("journal", "undo"),
            "datum.journal.redo": ("journal", "redo"),
        }
        for command_id, tokens in required_templates.items():
            with self.subTest(command_id=command_id):
                self.assertIn(f'"{command_id}"', source)
                for token in tokens:
                    self.assertIn(f'"{token}"', source)


if __name__ == "__main__":
    unittest.main()
