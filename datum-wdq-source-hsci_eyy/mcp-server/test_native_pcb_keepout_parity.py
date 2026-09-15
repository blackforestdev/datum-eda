#!/usr/bin/env python3
"""Native MCP parity for PCB keepout authoring."""

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


class TestNativePcbKeepoutParity(unittest.TestCase):
    def test_pcb_keepout_tools_create_edit_delete_and_replay(self) -> None:
        with tempfile.TemporaryDirectory(prefix="datum-mcp-keepout-") as tmp:
            root = Path(tmp)
            run_cli_json(root, "project", "new", str(root), "--name", "Keepout Parity")
            env = {"DATUM_CLI_BIN": " ".join(shlex.quote(part) for part in datum_cli_prefix())}
            with patch.dict(os.environ, env, clear=False):
                host = StdioToolHost(EngineDaemonClient())
                keepout = call_tool(
                    host,
                    "datum.pcb.place_keepout",
                    {
                        "path": str(root),
                        "vertices": ["0:0", "1000:0", "1000:500", "0:500"],
                        "layers": [1, 16],
                        "kind": "copper",
                    },
                )["keepout_uuid"]
                keepouts = run_cli_json(root, "project", "query", str(root), "board-keepouts")
                self.assertEqual(len(keepouts), 1)
                self.assertEqual(keepouts[0]["uuid"], keepout)
                self.assertEqual(keepouts[0]["kind"], "copper")
                self.assertEqual(keepouts[0]["layers"], [1, 16])
                operation = assert_latest_journal_operation(
                    self, host, str(root), "place board keepout", "create_board_keepout"
                )
                self.assertEqual(operation["keepout_id"], keepout)
                self.assertEqual(operation["keepout"], keepouts[0])
                edit = call_tool(
                    host,
                    "datum.pcb.edit_keepout",
                    {
                        "path": str(root),
                        "keepout": keepout,
                        "vertices": ["10:10", "1010:10", "1010:510", "10:510"],
                        "layers": [2],
                        "kind": "mixed",
                    },
                )
                self.assertEqual(edit["action"], "edit_board_keepout")
                keepouts = run_cli_json(root, "project", "query", str(root), "board-keepouts")
                self.assertEqual(keepouts[0]["kind"], "mixed")
                self.assertEqual(keepouts[0]["layers"], [2])
                self.assertEqual(keepouts[0]["polygon"]["vertices"][0], {"x": 10, "y": 10})
                operation = assert_latest_journal_operation(
                    self, host, str(root), "edit board keepout", "set_board_keepout"
                )
                self.assertEqual(operation["keepout_id"], keepout)
                self.assertEqual(operation["keepout"], keepouts[0])
                self.assertEqual(call_tool(host, "datum.journal.undo", {"path": str(root)})["status"], "applied")
                keepouts = run_cli_json(root, "project", "query", str(root), "board-keepouts")
                self.assertEqual(keepouts[0]["kind"], "copper")
                self.assertEqual(call_tool(host, "datum.journal.redo", {"path": str(root)})["status"], "applied")
                self.assertEqual(
                    call_tool(host, "datum.pcb.delete_keepout", {"path": str(root), "keepout": keepout})["action"],
                    "delete_board_keepout",
                )
                self.assertEqual(run_cli_json(root, "project", "query", str(root), "board-keepouts"), [])
                operation = assert_latest_journal_operation(
                    self, host, str(root), "delete board keepout", "delete_board_keepout"
                )
                self.assertEqual(operation["keepout_id"], keepout)


if __name__ == "__main__":
    unittest.main()
