#!/usr/bin/env python3
"""Check and standards-repair MCP tools/call dispatch tests."""

from __future__ import annotations

import unittest

from server_runtime import StdioToolHost
from test_support import FakeDaemonClient


class TestDispatchChecks(unittest.TestCase):
    def test_tools_call_dispatches_get_check_run(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 101,
                "method": "tools/call",
                "params": {
                    "name": "get_check_run",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "profile": "native-combined",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [("get_check_run", "/tmp/native-project", "native-combined")],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["action"], "native_project_check_run")
        self.assertEqual(payload["check_run_id"], "check-run-test")
        self.assertEqual(payload["persisted"], True)

    def test_datum_check_run_dispatches_standards_profile(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 102,
                "method": "tools/call",
                "params": {
                    "name": "datum.check.run",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "profile": "standards",
                    },
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_check_run", "/tmp/native-project", "standards")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["schema"], {"name": "datum.check.run", "version": 1})
        self.assertEqual(payload["result"]["profile_id"], "standards")
        self.assertEqual(payload["result"]["coverage"][1]["status"], "filtered_by_profile")

    def test_tools_call_dispatches_get_zone_fills(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 103,
                "method": "tools/call",
                "params": {
                    "name": "get_zone_fills",
                    "arguments": {"path": "/tmp/native-project"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_zone_fills", "/tmp/native-project")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "zone_fills_query_v1")
        self.assertEqual(payload["zone_fill_count"], 1)
        self.assertEqual(payload["zone_fills"][0]["state"], "unfilled")

    def test_tools_call_dispatches_fill_zones(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 104,
                "method": "tools/call",
                "params": {
                    "name": "fill_zones",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "zone": "zone-test",
                        "net": "net-test",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [("fill_zones", "/tmp/native-project", "zone-test", "net-test")],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "zone_fill_generate_v1")
        self.assertEqual(payload["zone_fills"][0]["state"], "unsupported")

    def test_tools_call_dispatches_get_check_runs(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 106,
                "method": "tools/call",
                "params": {
                    "name": "get_check_runs",
                    "arguments": {"path": "/tmp/native-project"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_check_runs", "/tmp/native-project")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "check_run_list_v1")
        self.assertEqual(payload["check_run_count"], 1)

    def test_tools_call_dispatches_show_check_run(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 107,
                "method": "tools/call",
                "params": {
                    "name": "show_check_run",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "check_run": "check-run-test",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [("show_check_run", "/tmp/native-project", "check-run-test")],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "check_run_record_v1")
        self.assertEqual(payload["check_run"]["check_run_id"], "check-run-test")
        self.assertEqual(payload["check_run"]["proposal_refs"], ["proposal-test"])
        self.assertEqual(
            payload["check_run"]["findings"][0]["proposal_refs"],
            ["proposal-test"],
        )

    def test_datum_check_show_normalizes_persisted_record_links(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 1071,
                "method": "tools/call",
                "params": {
                    "name": "datum.check.show",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "check_run": "check-run-test",
                    },
                },
            }
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["schema"], {"name": "datum.check.show", "version": 1})
        self.assertEqual(payload["result"]["check_run_id"], "check-run-test")
        self.assertEqual(payload["result"]["proposal_refs"], ["proposal-test"])
        self.assertEqual(payload["result"]["findings"][0]["proposal_refs"], ["proposal-test"])

    def test_tools_call_dispatches_get_check_profiles(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 108,
                "method": "tools/call",
                "params": {
                    "name": "get_check_profiles",
                    "arguments": {"path": "/tmp/native-project"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_check_profiles", "/tmp/native-project")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "check_profiles_v1")
        self.assertEqual(payload["default_profile_id"], "native-combined")

    def test_tools_call_dispatches_generate_standards_repair_proposals(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 102,
                "method": "tools/call",
                "params": {
                    "name": "generate_standards_repair_proposals",
                    "arguments": {"path": "/tmp/native-project"},
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [("generate_standards_repair_proposals", "/tmp/native-project")],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["action"], "generate_standards_repair_proposals")
        self.assertEqual(payload["proposal_count"], 1)

    def test_tools_call_dispatches_waive_finding(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 104,
                "method": "tools/call",
                "params": {
                    "name": "waive_finding",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "fingerprint": "sha256:finding",
                        "rationale": "Intentional design decision",
                        "created_by": "mcp-test",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "waive_finding",
                    "/tmp/native-project",
                    "sha256:finding",
                    "Intentional design decision",
                    "mcp-test",
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "project_waive_finding_v1")
        self.assertEqual(payload["fingerprint"], "sha256:finding")
        self.assertEqual(payload["domain"], "standards")
        self.assertEqual(payload["status"], "applied")

    def test_tools_call_dispatches_accept_deviation(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 105,
                "method": "tools/call",
                "params": {
                    "name": "accept_deviation",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "fingerprint": "sha256:finding",
                        "rationale": "Accepted design deviation",
                        "accepted_by": "mcp-test",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "accept_deviation",
                    "/tmp/native-project",
                    "sha256:finding",
                    "Accepted design deviation",
                    "mcp-test",
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "project_accept_deviation_v1")
        self.assertEqual(payload["fingerprint"], "sha256:finding")
        self.assertEqual(payload["domain"], "standards")
        self.assertEqual(payload["status"], "applied")
