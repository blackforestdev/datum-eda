#!/usr/bin/env python3
"""Canonical manufacturing/output-job MCP alias dispatch tests."""

from __future__ import annotations

import unittest

from server_runtime import StdioToolHost
from test_support import FakeDaemonClient


class TestDispatchOutputJobCanonicalAliases(unittest.TestCase):
    def test_dispatches_direct_manufacturing_and_output_job_aliases(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        cases = [
            ("datum.manufacturing.create_panel_projection", "create_panel_projection", {"path": "/tmp/native-project", "key": "main-panel"}),
            ("datum.manufacturing.update_panel_projection", "update_panel_projection", {"path": "/tmp/native-project", "panel_projection": "main-panel", "name": "Panel A"}),
            ("datum.manufacturing.delete_panel_projection", "delete_panel_projection", {"path": "/tmp/native-project", "panel_projection": "main-panel"}),
            ("datum.manufacturing.create_plan", "create_manufacturing_plan", {"path": "/tmp/native-project", "prefix": "fab/doa2526"}),
            ("datum.manufacturing.update_plan", "update_manufacturing_plan", {"path": "/tmp/native-project", "manufacturing_plan": "fab/doa2526", "name": "Plan A"}),
            ("datum.manufacturing.delete_plan", "delete_manufacturing_plan", {"path": "/tmp/native-project", "manufacturing_plan": "fab/doa2526"}),
            ("datum.output_job.create_gerber_set", "create_gerber_output_job", {"path": "/tmp/native-project", "prefix": "fab/doa2526"}),
            ("datum.output_job.create", "create_output_job", {"path": "/tmp/native-project", "prefix": "fab/doa2526", "include": "drill"}),
            ("datum.output_job.update", "update_output_job", {"path": "/tmp/native-project", "output_job": "gerber-set-default", "name": "Updated"}),
            ("datum.output_job.run", "run_output_job", {"path": "/tmp/native-project", "output_job": "gerber-set-default"}),
            ("datum.output_job.delete", "delete_output_job", {"path": "/tmp/native-project", "output_job": "gerber-set-default"}),
        ]
        for index, (tool_name, method_name, arguments) in enumerate(cases, start=710):
            response = host.handle_message(
                {
                    "jsonrpc": "2.0",
                    "id": index,
                    "method": "tools/call",
                    "params": {"name": tool_name, "arguments": arguments},
                }
            )
            self.assertEqual(daemon.calls[-1][0], method_name)
            payload = response["result"]["content"][0]["json"]
            self.assertTrue(payload["ok"])
            self.assertEqual(payload["schema"], {"name": tool_name, "version": 1})


if __name__ == "__main__":
    unittest.main()
