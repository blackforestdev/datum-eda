#!/usr/bin/env python3
"""Native MCP parity for PCB stackup authoring."""

from __future__ import annotations

import os
import shlex
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from server_runtime import EngineDaemonClient, StdioToolHost
from test_native_write_parity import (
    assert_latest_journal_operation,
    call_tool,
    datum_cli_prefix,
    run_cli_json,
)


CUSTOM_STACKUP = [
    "1:Top:Copper:35000:::1.0:0.4:RA Copper",
    "2:Core:Dielectric:1600000:4.2:0.018::0.2:FR-4",
    "3:Bottom:Copper:35000",
]


class TestNativePcbStackupParity(unittest.TestCase):
    def test_pcb_stackup_tool_sets_model_and_replays(self) -> None:
        with tempfile.TemporaryDirectory(prefix="datum-mcp-stackup-") as tmp:
            root = Path(tmp)
            run_cli_json(root, "project", "new", str(root), "--name", "Stackup Parity")
            env = {"DATUM_CLI_BIN": " ".join(shlex.quote(part) for part in datum_cli_prefix())}
            with patch.dict(os.environ, env, clear=False):
                host = StdioToolHost(EngineDaemonClient())
                result = call_tool(host, "datum.pcb.set_stackup", {"path": str(root), "layers": CUSTOM_STACKUP})
                self.assertEqual(result["action"], "set_board_stackup")
                self.assertEqual(result["layer_count"], 3)
                stackup = run_cli_json(root, "project", "query", str(root), "board-stackup")
                self.assertEqual(len(stackup), 3)
                self.assertEqual(stackup[0]["name"], "Top")
                self.assertEqual(stackup[0]["copper_weight_oz"], 1.0)
                self.assertEqual(stackup[0]["material_name"], "RA Copper")
                self.assertEqual(stackup[1]["layer_type"], "Dielectric")
                self.assertEqual(stackup[1]["dielectric_constant"], 4.2)
                self.assertEqual(stackup[1]["loss_tangent"], 0.018)
                self.assertEqual(stackup[1]["material_name"], "FR-4")
                operation = assert_latest_journal_operation(
                    self, host, str(root), "set board stackup", "set_board_stackup"
                )
                self.assertEqual(operation["stackup"]["layers"], stackup)
                self.assertEqual(call_tool(host, "datum.journal.undo", {"path": str(root)})["status"], "applied")
                stackup = run_cli_json(root, "project", "query", str(root), "board-stackup")
                self.assertEqual(len(stackup), 5)
                self.assertEqual(stackup[0]["name"], "Top Copper")
                self.assertEqual(stackup[4]["name"], "Mechanical 41")
                self.assertEqual(call_tool(host, "datum.journal.redo", {"path": str(root)})["status"], "applied")
                stackup = run_cli_json(root, "project", "query", str(root), "board-stackup")
                self.assertEqual(len(stackup), 3)
                self.assertEqual(stackup[2]["name"], "Bottom")

    def test_pcb_default_top_stackup_tool_retrofits_missing_layers(self) -> None:
        with tempfile.TemporaryDirectory(prefix="datum-mcp-stackup-default-") as tmp:
            root = Path(tmp)
            run_cli_json(root, "project", "new", str(root), "--name", "Default Stackup Parity")
            run_cli_json(
                root,
                "project",
                "set-board-stackup",
                str(root),
                "--layer",
                "1:Top Copper:Copper:35000",
                "--layer",
                "3:Top Silk:Silkscreen:10000",
            )
            env = {"DATUM_CLI_BIN": " ".join(shlex.quote(part) for part in datum_cli_prefix())}
            with patch.dict(os.environ, env, clear=False):
                host = StdioToolHost(EngineDaemonClient())
                result = call_tool(host, "datum.pcb.add_default_top_stackup", {"path": str(root)})
                self.assertEqual(result["action"], "add_default_top_stackup")
                self.assertEqual(result["layer_count"], 5)
                stackup = run_cli_json(root, "project", "query", str(root), "board-stackup")
                self.assertEqual([layer["id"] for layer in stackup], [1, 2, 3, 4, 41])
                self.assertEqual(stackup[1]["layer_type"], "SolderMask")
                self.assertEqual(stackup[3]["layer_type"], "Paste")
                operation = assert_latest_journal_operation(
                    self, host, str(root), "add default top stackup", "set_board_stackup"
                )
                self.assertEqual(len(operation["stackup"]["layers"]), 5)


if __name__ == "__main__":
    unittest.main()
