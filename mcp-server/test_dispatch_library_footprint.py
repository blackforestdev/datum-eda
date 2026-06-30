#!/usr/bin/env python3
"""Native footprint-library MCP dispatch tests."""

from __future__ import annotations

import unittest

from server_runtime import StdioToolHost
from test_support import FakeDaemonClient


class TestDispatchLibraryFootprint(unittest.TestCase):
    def test_tools_call_dispatches_pool_footprint_writes(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        for tool_name, arguments, expected_call in [
            (
                "create_pool_footprint",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "footprint": "footprint-test",
                    "package": "package-test",
                    "name": "SOT23_LandPattern",
                },
                (
                    "create_pool_footprint",
                    "/tmp/native-project",
                    "pool",
                    "footprint-test",
                    "package-test",
                    "SOT23_LandPattern",
                ),
            ),
            (
                "set_pool_footprint_pad",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "footprint": "footprint-test",
                    "pad": "pad-test",
                    "padstack": "padstack-test",
                    "pad_name": "2",
                    "x_nm": 1000,
                    "y_nm": 2000,
                    "layer": 1,
                },
                (
                    "set_pool_footprint_pad",
                    "/tmp/native-project",
                    "pool",
                    "footprint-test",
                    "pad-test",
                    "padstack-test",
                    "2",
                    1000,
                    2000,
                    1,
                ),
            ),
            (
                "set_pool_footprint_courtyard_rect",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "footprint": "footprint-test",
                    "min_x_nm": 1000,
                    "min_y_nm": 2000,
                    "max_x_nm": 3000,
                    "max_y_nm": 4000,
                },
                (
                    "set_pool_footprint_courtyard_rect",
                    "/tmp/native-project",
                    "pool",
                    "footprint-test",
                    1000,
                    2000,
                    3000,
                    4000,
                ),
            ),
            (
                "set_pool_footprint_courtyard_polygon",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "footprint": "footprint-test",
                    "vertices": "0,0;1000,0;1000,2000;0,2000",
                },
                (
                    "set_pool_footprint_courtyard_polygon",
                    "/tmp/native-project",
                    "pool",
                    "footprint-test",
                    "0,0;1000,0;1000,2000;0,2000",
                ),
            ),
            (
                "add_pool_footprint_silkscreen_line",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "footprint": "footprint-test",
                    "from_x_nm": 1000,
                    "from_y_nm": 2000,
                    "to_x_nm": 3000,
                    "to_y_nm": 4000,
                    "width_nm": 150000,
                },
                (
                    "add_pool_footprint_silkscreen_line",
                    "/tmp/native-project",
                    "pool",
                    "footprint-test",
                    1000,
                    2000,
                    3000,
                    4000,
                    150000,
                ),
            ),
            (
                "add_pool_footprint_silkscreen_rect",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "footprint": "footprint-test",
                    "min_x_nm": 1000,
                    "min_y_nm": 2000,
                    "max_x_nm": 3000,
                    "max_y_nm": 4000,
                    "width_nm": 150000,
                },
                (
                    "add_pool_footprint_silkscreen_rect",
                    "/tmp/native-project",
                    "pool",
                    "footprint-test",
                    1000,
                    2000,
                    3000,
                    4000,
                    150000,
                ),
            ),
            (
                "add_pool_footprint_silkscreen_circle",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "footprint": "footprint-test",
                    "center_x_nm": 5000,
                    "center_y_nm": 6000,
                    "radius_nm": 7000,
                    "width_nm": 150000,
                },
                (
                    "add_pool_footprint_silkscreen_circle",
                    "/tmp/native-project",
                    "pool",
                    "footprint-test",
                    5000,
                    6000,
                    7000,
                    150000,
                ),
            ),
            (
                "add_pool_footprint_silkscreen_polygon",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "footprint": "footprint-test",
                    "vertices": "0,0;1000,0;1000,1000",
                    "closed": True,
                    "width_nm": 150000,
                },
                (
                    "add_pool_footprint_silkscreen_polygon",
                    "/tmp/native-project",
                    "pool",
                    "footprint-test",
                    "0,0;1000,0;1000,1000",
                    True,
                    150000,
                ),
            ),
        ]:
            response = host.handle_message(
                {
                    "jsonrpc": "2.0",
                    "id": 230,
                    "method": "tools/call",
                    "params": {"name": tool_name, "arguments": arguments},
                }
            )
            self.assertEqual(daemon.calls[-1], expected_call)
            payload = response["result"]["content"][0]["json"]
            self.assertEqual(payload["object_uuid"], expected_call[3])


if __name__ == "__main__":
    unittest.main()
