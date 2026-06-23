#!/usr/bin/env python3
"""Canonical schematic sheet MCP dispatch tests."""

from __future__ import annotations

import unittest

from server_runtime import StdioToolHost
from test_support import FakeDaemonClient


class TestDispatchSchematicSheet(unittest.TestCase):
    def test_create_sheet_preserves_sheet_report_in_target_envelope(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)

        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 900,
                "method": "tools/call",
                "params": {
                    "name": "datum.schematic.create_sheet",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "name": "Aux",
                        "sheet": "sheet-1",
                    },
                },
            }
        )

        self.assertEqual(
            daemon.calls, [("create_sheet", "/tmp/native-project", "Aux", "sheet-1")]
        )
        payload = response["result"]["content"][0]["json"]
        self.assertTrue(payload["ok"])
        self.assertEqual(
            payload["schema"], {"name": "datum.schematic.create_sheet", "version": 1}
        )
        self.assertEqual(payload["result"]["action"], "create_sheet")
        self.assertEqual(payload["result"]["sheet_uuid"], "sheet-1")
        self.assertEqual(payload["result"]["cascaded_objects"], 0)

    def test_delete_sheet_preserves_cascade_report_in_target_envelope(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)

        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 901,
                "method": "tools/call",
                "params": {
                    "name": "datum.schematic.delete_sheet",
                    "arguments": {"path": "/tmp/native-project", "sheet": "sheet-1"},
                },
            }
        )

        self.assertEqual(daemon.calls, [("delete_sheet", "/tmp/native-project", "sheet-1")])
        payload = response["result"]["content"][0]["json"]
        self.assertTrue(payload["ok"])
        self.assertEqual(
            payload["schema"], {"name": "datum.schematic.delete_sheet", "version": 1}
        )
        self.assertEqual(payload["result"]["action"], "delete_sheet")
        self.assertEqual(payload["result"]["sheet_uuid"], "sheet-1")
        self.assertEqual(payload["result"]["cascaded_objects"], 2)
        self.assertEqual(payload["cascaded_objects"], 2)

    def test_rename_sheet_preserves_sheet_report_in_target_envelope(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)

        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 902,
                "method": "tools/call",
                "params": {
                    "name": "datum.schematic.rename_sheet",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "sheet": "sheet-1",
                        "name": "Renamed",
                    },
                },
            }
        )

        self.assertEqual(
            daemon.calls, [("rename_sheet", "/tmp/native-project", "sheet-1", "Renamed")]
        )
        payload = response["result"]["content"][0]["json"]
        self.assertTrue(payload["ok"])
        self.assertEqual(
            payload["schema"], {"name": "datum.schematic.rename_sheet", "version": 1}
        )
        self.assertEqual(payload["result"]["action"], "rename_sheet")
        self.assertEqual(payload["result"]["sheet_uuid"], "sheet-1")
        self.assertEqual(payload["result"]["name"], "Renamed")

    def test_create_sheet_definition_preserves_definition_report_in_target_envelope(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)

        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 903,
                "method": "tools/call",
                "params": {
                    "name": "datum.schematic.create_sheet_definition",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "root_sheet": "sheet-1",
                        "name": "Main Definition",
                        "definition": "definition-1",
                    },
                },
            }
        )

        self.assertEqual(
            daemon.calls,
            [
                (
                    "create_sheet_definition",
                    "/tmp/native-project",
                    "sheet-1",
                    "Main Definition",
                    "definition-1",
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertTrue(payload["ok"])
        self.assertEqual(
            payload["schema"],
            {"name": "datum.schematic.create_sheet_definition", "version": 1},
        )
        self.assertEqual(payload["result"]["action"], "create_sheet_definition")
        self.assertEqual(payload["result"]["definition_uuid"], "definition-1")
        self.assertEqual(payload["result"]["root_sheet_uuid"], "sheet-1")

    def test_create_sheet_instance_preserves_instance_report_in_target_envelope(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)

        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 904,
                "method": "tools/call",
                "params": {
                    "name": "datum.schematic.create_sheet_instance",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "definition": "definition-1",
                        "parent_sheet": "sheet-1",
                        "name": "Main Instance",
                        "x_nm": 100,
                        "y_nm": 200,
                        "instance": "instance-1",
                    },
                },
            }
        )

        self.assertEqual(
            daemon.calls,
            [
                (
                    "create_sheet_instance",
                    "/tmp/native-project",
                    "definition-1",
                    "Main Instance",
                    100,
                    200,
                    "sheet-1",
                    "instance-1",
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertTrue(payload["ok"])
        self.assertEqual(
            payload["schema"],
            {"name": "datum.schematic.create_sheet_instance", "version": 1},
        )
        self.assertEqual(payload["result"]["action"], "create_sheet_instance")
        self.assertEqual(payload["result"]["instance_uuid"], "instance-1")

    def test_delete_sheet_instance_preserves_instance_report_in_target_envelope(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)

        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 905,
                "method": "tools/call",
                "params": {
                    "name": "datum.schematic.delete_sheet_instance",
                    "arguments": {"path": "/tmp/native-project", "instance": "instance-1"},
                },
            }
        )

        self.assertEqual(
            daemon.calls, [("delete_sheet_instance", "/tmp/native-project", "instance-1")]
        )
        payload = response["result"]["content"][0]["json"]
        self.assertTrue(payload["ok"])
        self.assertEqual(
            payload["schema"],
            {"name": "datum.schematic.delete_sheet_instance", "version": 1},
        )
        self.assertEqual(payload["result"]["action"], "delete_sheet_instance")
        self.assertEqual(payload["result"]["instance_uuid"], "instance-1")

    def test_move_sheet_instance_preserves_instance_report_in_target_envelope(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)

        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 906,
                "method": "tools/call",
                "params": {
                    "name": "datum.schematic.move_sheet_instance",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "instance": "instance-1",
                        "x_nm": 300,
                        "y_nm": 400,
                    },
                },
            }
        )

        self.assertEqual(
            daemon.calls,
            [("move_sheet_instance", "/tmp/native-project", "instance-1", 300, 400)],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertTrue(payload["ok"])
        self.assertEqual(
            payload["schema"],
            {"name": "datum.schematic.move_sheet_instance", "version": 1},
        )
        self.assertEqual(payload["result"]["action"], "move_sheet_instance")
        self.assertEqual(payload["result"]["x_nm"], 300)

    def test_bind_sheet_instance_port_preserves_instance_report_in_target_envelope(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)

        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 907,
                "method": "tools/call",
                "params": {
                    "name": "datum.schematic.bind_sheet_instance_port",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "instance": "instance-1",
                        "port": "port-1",
                    },
                },
            }
        )

        self.assertEqual(
            daemon.calls,
            [("bind_sheet_instance_port", "/tmp/native-project", "instance-1", "port-1")],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertTrue(payload["ok"])
        self.assertEqual(
            payload["schema"],
            {"name": "datum.schematic.bind_sheet_instance_port", "version": 1},
        )
        self.assertEqual(payload["result"]["action"], "bind_sheet_instance_port")
        self.assertEqual(payload["result"]["port_uuid"], "port-1")

    def test_unbind_sheet_instance_port_preserves_instance_report_in_target_envelope(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)

        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 908,
                "method": "tools/call",
                "params": {
                    "name": "datum.schematic.unbind_sheet_instance_port",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "instance": "instance-1",
                        "port": "port-1",
                    },
                },
            }
        )

        self.assertEqual(
            daemon.calls,
            [("unbind_sheet_instance_port", "/tmp/native-project", "instance-1", "port-1")],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertTrue(payload["ok"])
        self.assertEqual(
            payload["schema"],
            {"name": "datum.schematic.unbind_sheet_instance_port", "version": 1},
        )
        self.assertEqual(payload["result"]["action"], "unbind_sheet_instance_port")
        self.assertEqual(payload["result"]["port_uuid"], "port-1")


if __name__ == "__main__":
    unittest.main()
