#!/usr/bin/env python3
"""Write-surface MCP dispatch tests for basic command routing."""

from __future__ import annotations

import unittest

from server_runtime import StdioToolHost
from test_support import FakeDaemonClient


class TestDispatchWriteBasicsCommands(unittest.TestCase):
    def test_tools_call_dispatches_move_component(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 210,
                "method": "tools/call",
                "params": {
                    "name": "move_component",
                    "arguments": {
                        "uuid": "comp-1",
                        "x_mm": 15.0,
                        "y_mm": 12.0,
                        "rotation_deg": 90.0,
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [("move_component", "comp-1", 15.0, 12.0, 90.0)],
        )
        self.assertEqual(
            response["result"]["content"][0]["json"]["diff"]["modified"][0]["object_type"],
            "component",
        )

    def test_tools_call_dispatches_rotate_component(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 221,
                "method": "tools/call",
                "params": {
                    "name": "rotate_component",
                    "arguments": {
                        "uuid": "comp-1",
                        "rotation_deg": 180.0,
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [("rotate_component", "comp-1", 180.0)],
        )
        self.assertEqual(
            response["result"]["content"][0]["json"]["diff"]["modified"][0]["object_type"],
            "component",
        )

    def test_tools_call_dispatches_set_value(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 223,
                "method": "tools/call",
                "params": {
                    "name": "set_value",
                    "arguments": {
                        "uuid": "comp-1",
                        "value": "22k",
                    },
                },
            }
        )
        self.assertEqual(daemon.calls, [("set_value", "comp-1", "22k")])
        self.assertEqual(
            response["result"]["content"][0]["json"]["diff"]["modified"][0]["object_type"],
            "component",
        )

    def test_tools_call_dispatches_undo_and_redo(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        undo = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 206,
                "method": "tools/call",
                "params": {"name": "undo", "arguments": {}},
            }
        )
        redo = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 207,
                "method": "tools/call",
                "params": {"name": "redo", "arguments": {}},
            }
        )
        self.assertEqual(daemon.calls, [("undo", None), ("redo", None)])
        self.assertEqual(
            undo["result"]["content"][0]["json"]["diff"]["created"][0]["uuid"],
            "track-1",
        )
        self.assertEqual(
            redo["result"]["content"][0]["json"]["diff"]["deleted"][0]["uuid"],
            "track-1",
        )
