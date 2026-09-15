#!/usr/bin/env python3
"""Native MCP parity for PCB board outline authoring."""

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


class TestNativePcbOutlineParity(unittest.TestCase):
    def test_pcb_outline_tool_sets_model_and_replays(self) -> None:
        with tempfile.TemporaryDirectory(prefix="datum-mcp-outline-") as tmp:
            root = Path(tmp)
            run_cli_json(root, "project", "new", str(root), "--name", "Outline Parity")
            env = {"DATUM_CLI_BIN": " ".join(shlex.quote(part) for part in datum_cli_prefix())}
            with patch.dict(os.environ, env, clear=False):
                host = StdioToolHost(EngineDaemonClient())
                result = call_tool(
                    host,
                    "datum.pcb.set_outline",
                    {
                        "path": str(root),
                        "vertices": ["0:0", "2000:0", "1500:1000", "0:1000"],
                    },
                )
                self.assertEqual(result["action"], "set_board_outline")
                self.assertEqual(result["vertex_count"], 4)
                self.assertTrue(result["closed"])
                outline = run_cli_json(root, "project", "query", str(root), "board-outline")
                self.assertEqual(
                    outline,
                    {
                        "vertices": [
                            {"x": 0, "y": 0},
                            {"x": 2000, "y": 0},
                            {"x": 1500, "y": 1000},
                            {"x": 0, "y": 1000},
                        ],
                        "closed": True,
                    },
                )
                operation = assert_latest_journal_operation(
                    self, host, str(root), "set board outline", "set_board_outline"
                )
                self.assertEqual(operation["outline"], outline)
                self.assertEqual(call_tool(host, "datum.journal.undo", {"path": str(root)})["status"], "applied")
                outline = run_cli_json(root, "project", "query", str(root), "board-outline")
                self.assertEqual(outline, {"vertices": [], "closed": True})
                self.assertEqual(call_tool(host, "datum.journal.redo", {"path": str(root)})["status"], "applied")
                outline = run_cli_json(root, "project", "query", str(root), "board-outline")
                self.assertEqual(len(outline["vertices"]), 4)
                self.assertEqual(outline["vertices"][2], {"x": 1500, "y": 1000})


if __name__ == "__main__":
    unittest.main()
