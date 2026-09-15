#!/usr/bin/env python3
"""run_drc dispatch tests for selected DRC rule forwarding."""

from __future__ import annotations

import unittest

from stdio_tool_host import StdioToolHost
from test_support import FakeDaemonClient


class TestRunDrcRuleDispatch(unittest.TestCase):
    def test_tools_call_dispatches_run_drc_with_selected_rules(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 78,
                "method": "tools/call",
                "params": {
                    "name": "run_drc",
                    "arguments": {"rules": ["TrackWidth"]},
                },
            }
        )
        self.assertEqual(daemon.calls, [("run_drc", ["TrackWidth"])])
        self.assertEqual(response["result"]["content"][0]["json"]["passed"], False)


if __name__ == "__main__":
    unittest.main()
