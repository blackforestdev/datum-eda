#!/usr/bin/env python3
"""OutputJob proposal MCP dispatch tests."""

from __future__ import annotations

import unittest

from server_runtime import StdioToolHost
from test_support import FakeDaemonClient


class TestDispatchOutputJobProposals(unittest.TestCase):
    def test_tools_call_dispatches_create_output_job_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 240,
                "method": "tools/call",
                "params": {
                    "name": "create_output_job_proposal",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "prefix": "release-a",
                        "include": "drill",
                        "name": "Reviewed Drill",
                        "proposal": "proposal-create-test",
                    },
                },
            }
        )
        self.assertEqual(daemon.calls[-1][0], "create_output_job_proposal")
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(payload["action"], "propose_create_output_job")

    def test_tools_call_dispatches_update_output_job_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 241,
                "method": "tools/call",
                "params": {
                    "name": "update_output_job_proposal",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "output_job": "output-job-test",
                        "name": "Reviewed CAM",
                        "output_dir": "/tmp/fab",
                        "clear_manufacturing_plan": True,
                        "proposal": "proposal-test",
                        "rationale": "review CAM job update",
                    },
                },
            }
        )
        self.assertEqual(daemon.calls[-1][0], "update_output_job_proposal")
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(payload["action"], "propose_update_output_job")

    def test_tools_call_dispatches_delete_output_job_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 242,
                "method": "tools/call",
                "params": {
                    "name": "delete_output_job_proposal",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "output_job": "output-job-test",
                        "proposal": "proposal-delete-test",
                    },
                },
            }
        )
        self.assertEqual(daemon.calls[-1][0], "delete_output_job_proposal")
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(payload["action"], "propose_delete_output_job")


if __name__ == "__main__":
    unittest.main()
