#!/usr/bin/env python3
"""Manufacturing proposal MCP dispatch tests."""

from __future__ import annotations

import unittest

from server_runtime import StdioToolHost
from test_support import FakeDaemonClient


class TestDispatchManufacturingProposals(unittest.TestCase):
    def test_tools_call_dispatches_manufacturing_and_panel_proposal_tools(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        cases = [
            ("create_manufacturing_plan_proposal", {"path": "/tmp/native-project", "prefix": "release-a"}),
            ("update_manufacturing_plan_proposal", {"path": "/tmp/native-project", "manufacturing_plan": "plan-test", "name": "Release A"}),
            ("delete_manufacturing_plan_proposal", {"path": "/tmp/native-project", "manufacturing_plan": "plan-test"}),
            ("create_panel_projection_proposal", {"path": "/tmp/native-project", "key": "panel-a"}),
            ("update_panel_projection_proposal", {"path": "/tmp/native-project", "panel_projection": "panel-test", "name": "Panel A"}),
            ("delete_panel_projection_proposal", {"path": "/tmp/native-project", "panel_projection": "panel-test"}),
        ]
        for index, (name, arguments) in enumerate(cases, start=250):
            response = host.handle_message(
                {
                    "jsonrpc": "2.0",
                    "id": index,
                    "method": "tools/call",
                    "params": {"name": name, "arguments": arguments},
                }
            )
            self.assertEqual(daemon.calls[-1][0], name)
            payload = response["result"]["content"][0]["json"]
            self.assertEqual(payload["contract"], "proposal_create_v1")
            self.assertTrue(payload["action"].startswith("propose_"))


if __name__ == "__main__":
    unittest.main()
