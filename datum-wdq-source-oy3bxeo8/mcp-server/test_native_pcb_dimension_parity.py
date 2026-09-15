#!/usr/bin/env python3
"""Native MCP parity for PCB dimension authoring."""

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


class TestNativePcbDimensionParity(unittest.TestCase):
    def test_pcb_dimension_tools_create_edit_delete_and_replay(self) -> None:
        with tempfile.TemporaryDirectory(prefix="datum-mcp-dimension-") as tmp:
            root = Path(tmp)
            run_cli_json(root, "project", "new", str(root), "--name", "Dimension Parity")
            env = {"DATUM_CLI_BIN": " ".join(shlex.quote(part) for part in datum_cli_prefix())}
            with patch.dict(os.environ, env, clear=False):
                host = StdioToolHost(EngineDaemonClient())
                dimension = call_tool(
                    host,
                    "datum.pcb.place_dimension",
                    {
                        "path": str(root),
                        "from_x_nm": 0,
                        "from_y_nm": 0,
                        "to_x_nm": 1000,
                        "to_y_nm": 500,
                        "layer": 41,
                        "text": "1000x500",
                    },
                )["dimension_uuid"]
                dimensions = run_cli_json(root, "project", "query", str(root), "board-dimensions")
                self.assertEqual(len(dimensions), 1)
                self.assertEqual(dimensions[0]["uuid"], dimension)
                self.assertEqual(dimensions[0]["from"], {"x": 0, "y": 0})
                self.assertEqual(dimensions[0]["to"], {"x": 1000, "y": 500})
                self.assertEqual(dimensions[0]["layer"], 41)
                self.assertEqual(dimensions[0]["text"], "1000x500")
                operation = assert_latest_journal_operation(
                    self, host, str(root), "place board dimension", "create_board_dimension"
                )
                self.assertEqual(operation["dimension_id"], dimension)
                self.assertEqual(operation["dimension"], dimensions[0])
                edit = call_tool(
                    host,
                    "datum.pcb.edit_dimension",
                    {
                        "path": str(root),
                        "dimension": dimension,
                        "from_x_nm": 10,
                        "from_y_nm": 20,
                        "to_x_nm": 1010,
                        "to_y_nm": 520,
                        "layer": 42,
                        "text": "revised",
                    },
                )
                self.assertEqual(edit["action"], "edit_board_dimension")
                dimensions = run_cli_json(root, "project", "query", str(root), "board-dimensions")
                self.assertEqual(dimensions[0]["from"], {"x": 10, "y": 20})
                self.assertEqual(dimensions[0]["to"], {"x": 1010, "y": 520})
                self.assertEqual(dimensions[0]["layer"], 42)
                self.assertEqual(dimensions[0]["text"], "revised")
                operation = assert_latest_journal_operation(
                    self, host, str(root), "edit board dimension", "set_board_dimension"
                )
                self.assertEqual(operation["dimension"], dimensions[0])
                self.assertEqual(call_tool(host, "datum.journal.undo", {"path": str(root)})["status"], "applied")
                dimensions = run_cli_json(root, "project", "query", str(root), "board-dimensions")
                self.assertEqual(dimensions[0]["text"], "1000x500")
                self.assertEqual(call_tool(host, "datum.journal.redo", {"path": str(root)})["status"], "applied")
                self.assertEqual(call_tool(host, "datum.pcb.edit_dimension", {"path": str(root), "dimension": dimension, "clear_text": True})["text"], None)
                dimensions = run_cli_json(root, "project", "query", str(root), "board-dimensions")
                self.assertIsNone(dimensions[0]["text"])
                self.assertEqual(
                    call_tool(host, "datum.pcb.delete_dimension", {"path": str(root), "dimension": dimension})[
                        "action"
                    ],
                    "delete_board_dimension",
                )
                self.assertEqual(run_cli_json(root, "project", "query", str(root), "board-dimensions"), [])
                operation = assert_latest_journal_operation(
                    self, host, str(root), "delete board dimension", "delete_board_dimension"
                )
                self.assertEqual(operation["dimension_id"], dimension)


if __name__ == "__main__":
    unittest.main()
