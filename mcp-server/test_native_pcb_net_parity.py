#!/usr/bin/env python3
"""Native MCP parity for PCB net and net-class authoring."""

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
    query_result,
    run_cli_json,
)


class TestNativePcbNetParity(unittest.TestCase):
    def test_pcb_net_class_tools_create_edit_delete_and_replay(self) -> None:
        with tempfile.TemporaryDirectory(prefix="datum-mcp-net-class-") as tmp:
            root = Path(tmp)
            run_cli_json(root, "project", "new", str(root), "--name", "Net Class Parity")
            env = {"DATUM_CLI_BIN": " ".join(shlex.quote(part) for part in datum_cli_prefix())}
            with patch.dict(os.environ, env, clear=False):
                host = StdioToolHost(EngineDaemonClient())
                net_class = call_tool(
                    host,
                    "datum.pcb.place_net_class",
                    {
                        "path": str(root),
                        "name": "Default",
                        "clearance_nm": 150000,
                        "track_width_nm": 200000,
                        "via_drill_nm": 300000,
                        "via_diameter_nm": 600000,
                        "diffpair_width_nm": 180000,
                        "diffpair_gap_nm": 170000,
                    },
                )["net_class_uuid"]
                classes = query_result(host, "datum.query.board_net_classes", root)
                self.assertEqual(classes[0]["uuid"], net_class)
                self.assertEqual(classes[0]["diffpair_gap"], 170000)
                operation = assert_latest_journal_operation(
                    self, host, str(root), "place board net class", "create_board_net_class"
                )
                self.assertEqual(operation["net_class"], classes[0])
                call_tool(
                    host,
                    "datum.pcb.edit_net_class",
                    {"path": str(root), "net_class": net_class, "name": "HighSpeed", "track_width_nm": 250000},
                )
                classes = query_result(host, "datum.query.board_net_classes", root)
                self.assertEqual(classes[0]["name"], "HighSpeed")
                self.assertEqual(classes[0]["track_width"], 250000)
                operation = assert_latest_journal_operation(
                    self, host, str(root), "edit board net class", "set_board_net_class"
                )
                self.assertEqual(operation["net_class"], classes[0])
                self.assertEqual(call_tool(host, "datum.journal.undo", {"path": str(root)})["status"], "applied")
                self.assertEqual(query_result(host, "datum.query.board_net_classes", root)[0]["name"], "Default")
                self.assertEqual(call_tool(host, "datum.journal.redo", {"path": str(root)})["status"], "applied")
                self.assertEqual(
                    call_tool(host, "datum.pcb.delete_net_class", {"path": str(root), "net_class": net_class})[
                        "action"
                    ],
                    "delete_board_net_class",
                )
                self.assertEqual(query_result(host, "datum.query.board_net_classes", root), [])
                operation = assert_latest_journal_operation(
                    self, host, str(root), "delete board net class", "delete_board_net_class"
                )
                self.assertEqual(operation["net_class_id"], net_class)

    def test_pcb_net_tools_create_edit_clear_delete_and_replay(self) -> None:
        with tempfile.TemporaryDirectory(prefix="datum-mcp-net-") as tmp:
            root = Path(tmp)
            run_cli_json(root, "project", "new", str(root), "--name", "Net Parity")
            env = {"DATUM_CLI_BIN": " ".join(shlex.quote(part) for part in datum_cli_prefix())}
            with patch.dict(os.environ, env, clear=False):
                host = StdioToolHost(EngineDaemonClient())
                net_class = call_tool(
                    host,
                    "datum.pcb.place_net_class",
                    {
                        "path": str(root),
                        "name": "Default",
                        "clearance_nm": 150000,
                        "track_width_nm": 200000,
                        "via_drill_nm": 300000,
                        "via_diameter_nm": 600000,
                    },
                )["net_class_uuid"]
                net = call_tool(
                    host,
                    "datum.pcb.place_net",
                    {
                        "path": str(root),
                        "name": "GND",
                        "class": net_class,
                        "impedance_target_ohms": "50.0",
                        "impedance_tolerance_pct": "10",
                        "controlled_dielectric_layer": 2,
                    },
                )["net_uuid"]
                nets = query_result(host, "datum.query.board_nets", root)
                self.assertEqual(nets[0]["uuid"], net)
                self.assertEqual(nets[0]["controlled_impedance"]["controlled_dielectric"], 2)
                operation = assert_latest_journal_operation(self, host, str(root), "place board net", "create_board_net")
                self.assertEqual(operation["net"], nets[0])
                call_tool(
                    host,
                    "datum.pcb.edit_net",
                    {"path": str(root), "net": net, "name": "PWR_GND", "impedance_tolerance_pct": "7.5"},
                )
                nets = query_result(host, "datum.query.board_nets", root)
                self.assertEqual(nets[0]["name"], "PWR_GND")
                self.assertEqual(nets[0]["controlled_impedance"]["tolerance_pct"], 7.5)
                operation = assert_latest_journal_operation(self, host, str(root), "edit board net", "set_board_net")
                self.assertEqual(operation["net"], nets[0])
                self.assertEqual(call_tool(host, "datum.journal.undo", {"path": str(root)})["status"], "applied")
                self.assertEqual(query_result(host, "datum.query.board_nets", root)[0]["name"], "GND")
                self.assertEqual(call_tool(host, "datum.journal.redo", {"path": str(root)})["status"], "applied")
                call_tool(host, "datum.pcb.edit_net", {"path": str(root), "net": net, "clear_controlled_impedance": True})
                self.assertNotIn("controlled_impedance", query_result(host, "datum.query.board_nets", root)[0])
                self.assertEqual(
                    call_tool(host, "datum.pcb.delete_net", {"path": str(root), "net": net})["action"],
                    "delete_board_net",
                )
                self.assertEqual(query_result(host, "datum.query.board_nets", root), [])
                operation = assert_latest_journal_operation(self, host, str(root), "delete board net", "delete_board_net")
                self.assertEqual(operation["net_id"], net)


if __name__ == "__main__":
    unittest.main()
