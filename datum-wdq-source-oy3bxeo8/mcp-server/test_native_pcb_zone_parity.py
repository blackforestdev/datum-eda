#!/usr/bin/env python3
"""Native MCP parity for PCB copper zones."""

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
    seed_board_net,
)


class TestNativePcbZoneParity(unittest.TestCase):
    def test_pcb_zone_tools_call_writes_deletes_and_replays(self) -> None:
        with tempfile.TemporaryDirectory(prefix="datum-mcp-zone-") as tmp:
            root = Path(tmp)
            run_cli_json(root, "project", "new", str(root), "--name", "Zone Parity")
            env = {"DATUM_CLI_BIN": " ".join(shlex.quote(part) for part in datum_cli_prefix())}
            with patch.dict(os.environ, env, clear=False):
                host = StdioToolHost(EngineDaemonClient())
                net = seed_board_net(host, root)
                zone = call_tool(
                    host,
                    "datum.pcb.place_zone",
                    {
                        "path": str(root),
                        "net": net,
                        "vertices": ["0:0", "1000:0", "1000:1000", "0:1000"],
                        "layer": 1,
                        "priority": 2,
                        "thermal_relief": True,
                        "thermal_gap_nm": 250000,
                        "thermal_spoke_width_nm": 200000,
                    },
                )["zone_uuid"]
                zones = query_result(host, "datum.query.board_zones", root)
                self.assertEqual(len(zones), 1)
                self.assertEqual(zones[0]["uuid"], zone)
                self.assertEqual(zones[0]["net"], net)
                self.assertEqual(zones[0]["layer"], 1)
                self.assertEqual(zones[0]["priority"], 2)
                self.assertTrue(zones[0]["thermal_relief"])
                self.assertEqual(zones[0]["thermal_gap"], 250000)
                self.assertEqual(zones[0]["thermal_spoke_width"], 200000)
                self.assertEqual(
                    zones[0]["polygon"],
                    {
                        "vertices": [
                            {"x": 0, "y": 0},
                            {"x": 1000, "y": 0},
                            {"x": 1000, "y": 1000},
                            {"x": 0, "y": 1000},
                        ],
                        "closed": True,
                    },
                )
                operation = assert_latest_journal_operation(
                    self, host, str(root), "place board zone", "create_board_zone"
                )
                self.assertEqual(operation["zone_id"], zone)
                self.assertEqual(operation["zone"]["uuid"], zone)
                self.assertEqual(operation["zone"]["net"], net)
                self.assertEqual(operation["zone"]["polygon"]["vertices"][2], {"x": 1000, "y": 1000})
                self.assertEqual(call_tool(host, "datum.journal.undo", {"path": str(root)})["status"], "applied")
                self.assertEqual(query_result(host, "datum.query.board_zones", root), [])
                self.assertEqual(call_tool(host, "datum.journal.redo", {"path": str(root)})["status"], "applied")
                self.assertEqual(query_result(host, "datum.query.board_zones", root)[0]["uuid"], zone)
                self.assertEqual(
                    call_tool(
                        host,
                        "datum.pcb.edit_zone",
                        {
                            "path": str(root),
                            "zone": zone,
                            "vertices": ["0:0", "2000:0", "2000:1000", "0:1000"],
                            "layer": 2,
                            "priority": 5,
                            "thermal_relief": False,
                            "thermal_gap_nm": 0,
                            "thermal_spoke_width_nm": 0,
                        },
                    )["action"],
                    "edit_board_zone",
                )
                zones = query_result(host, "datum.query.board_zones", root)
                self.assertEqual(zones[0]["uuid"], zone)
                self.assertEqual(zones[0]["net"], net)
                self.assertEqual(zones[0]["layer"], 2)
                self.assertEqual(zones[0]["priority"], 5)
                self.assertFalse(zones[0]["thermal_relief"])
                self.assertEqual(zones[0]["thermal_gap"], 0)
                self.assertEqual(zones[0]["thermal_spoke_width"], 0)
                self.assertEqual(zones[0]["polygon"]["vertices"][1], {"x": 2000, "y": 0})
                operation = assert_latest_journal_operation(
                    self, host, str(root), "edit board zone", "set_board_zone"
                )
                self.assertEqual(operation["zone_id"], zone)
                self.assertEqual(operation["zone"]["uuid"], zone)
                self.assertEqual(operation["zone"]["layer"], 2)
                fill_report = call_tool(
                    host,
                    "datum.check.fill_zones",
                    {"path": str(root), "zone": zone},
                )
                self.assertEqual(fill_report["action"], "fill_zones")
                self.assertEqual(fill_report["zone_fills"][0]["zone_id"], zone)
                self.assertEqual(fill_report["zone_fills"][0]["state"], "filled")
                operation = assert_latest_journal_operation(
                    self, host, str(root), "fill zones", "set_zone_fill"
                )
                self.assertEqual(operation["zone_id"], zone)
                self.assertEqual(operation["zone_fill"]["zone_id"], zone)
                self.assertEqual(
                    call_tool(host, "datum.pcb.delete_zone", {"path": str(root), "zone": zone})["action"],
                    "delete_board_zone",
                )
                self.assertEqual(query_result(host, "datum.query.board_zones", root), [])
                operation = assert_latest_journal_operation(
                    self, host, str(root), "delete board zone", "delete_board_zone"
                )
                self.assertEqual(operation["zone_id"], zone)


if __name__ == "__main__":
    unittest.main()
