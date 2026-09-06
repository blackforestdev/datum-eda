#!/usr/bin/env python3
"""GP-CM03 MCP catalog, transport-authority, and envelope regressions."""

from __future__ import annotations

import unittest

from stdio_tool_host import StdioToolHost
from test_support import FakeDaemonClient
from tools_catalog_data import TOOL_BY_NAME, TOOLS


EXPECTED_TOOLS = {
    "datum.preferences.describe",
    "datum.preferences.list",
    "datum.preferences.get",
    "datum.preferences.search",
    "datum.preferences.explain",
    "datum.preferences.preview_project_units_seed",
    "datum.preferences.proposal.prepare",
    "datum.preferences.proposal.validate",
    "datum.preferences.proposal.accept_apply",
    "datum.preferences.proposal.reject",
}


class TestPreferencesProductMcp(unittest.TestCase):
    def test_catalog_exposes_exact_inventory_without_direct_mutation(self) -> None:
        public = {tool["name"] for tool in TOOLS if tool["name"].startswith("datum.preferences.")}
        self.assertEqual(public, EXPECTED_TOOLS)
        self.assertNotIn("datum.preferences.set", TOOL_BY_NAME)
        self.assertNotIn("datum.preferences.reset", TOOL_BY_NAME)
        for name in EXPECTED_TOOLS:
            schema = TOOL_BY_NAME[name]["inputSchema"]
            self.assertNotIn("actor", schema.get("properties", {}))
            self.assertNotIn("_transport_actor", schema.get("properties", {}))

    def test_host_injects_mcp_actor_and_preserves_product_envelope(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 7,
                "method": "tools/call",
                "params": {"name": "datum.preferences.describe", "arguments": {}},
            }
        )
        product = response["result"]["content"][0]["json"]
        self.assertTrue(product["ok"])
        self.assertEqual(product["schema"]["name"], "datum.preferences.describe")
        self.assertEqual(product["context"]["scope"], "global_this_device")
        self.assertNotIn("data", product)
        method, params = daemon.calls[-1]
        self.assertEqual(method, "preferences.mcp_product")
        self.assertEqual(params["actor"]["kind"], "mcp_agent")
        self.assertEqual(params["actor"]["session_id"], "unscoped-test-session")
        self.assertEqual(params["request"]["payload"]["request"], {"kind": "describe"})

    def test_prepare_carries_only_typed_payload_and_transport_actor(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        mutation = {
            "kind": "set_user",
            "key": "datum.accessibility.reduced_motion",
            "value": True,
            "expected": {"kind": "missing"},
            "request_id": "00000000-0000-4000-8000-000000000001",
            "reason": "reduce animation",
        }
        host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 8,
                "method": "tools/call",
                "params": {
                    "name": "datum.preferences.proposal.prepare",
                    "arguments": {"mutation": mutation, "rationale": "user comfort"},
                },
            }
        )
        _, params = daemon.calls[-1]
        payload = params["request"]["payload"]
        self.assertEqual(payload["class"], "proposal")
        self.assertEqual(payload["request"]["kind"], "prepare")
        self.assertEqual(payload["request"]["mutation"], mutation)
        self.assertNotIn("actor", payload["request"])

    def test_project_new_requires_explicit_units_source_and_injects_actor(self) -> None:
        schema = TOOL_BY_NAME["datum.project.new"]["inputSchema"]
        self.assertIn("units_source", schema["required"])
        self.assertNotIn("actor", schema["properties"])
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        request_id = "00000000-0000-4000-8000-000000000041"
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 41,
                "method": "tools/call",
                "params": {
                    "name": "datum.project.new",
                    "arguments": {
                        "request_id": request_id,
                        "destination": "/tmp/mcp-project",
                        "project_name": "MCP Project",
                        "units_source": {
                            "kind": "factory",
                            "profile_id": "datum.units.factory.v1",
                        },
                    },
                },
            }
        )
        envelope = response["result"]["content"][0]["json"]
        self.assertTrue(envelope["ok"])
        self.assertEqual(envelope["schema"]["name"], "datum.project.new")
        method, params = daemon.calls[-1]
        self.assertEqual(method, "project.genesis")
        self.assertEqual(params["actor"]["kind"], "mcp_agent")
        self.assertEqual(params["request"]["units_source"]["kind"], "factory")


if __name__ == "__main__":
    unittest.main()
