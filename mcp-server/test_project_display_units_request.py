#!/usr/bin/env python3
"""Project Working Units MCP request-shaping and dispatch proof."""

from __future__ import annotations

import unittest

from server_runtime import EngineDaemonClient, JsonRpcResponse, StdioToolHost


PROFILE = {
    "system": "metric",
    "board_length": "follow_system",
    "board_length_precision": "automatic",
    "drill_hole": "mm",
    "drill_hole_precision": "decimal_3",
    "schematic_geometry": "mil",
    "schematic_geometry_precision": "exact_nanometer",
    "angle_precision": "decimal_2",
}


class CapturingDaemon(EngineDaemonClient):
    def __init__(self) -> None:
        super().__init__("/not-used")
        self.request = None

    def call(self, request):
        self.request = request
        return JsonRpcResponse("2.0", request.id, {"status": "committed"}, None)


class CapturingCliDaemon(EngineDaemonClient):
    def __init__(self) -> None:
        super().__init__()
        self.request = None
        self.args = None
        self.allowed_statuses = None

    def _run_cli_json_allowing_statuses(self, request, args, allowed_statuses):
        self.request = request
        self.args = args
        self.allowed_statuses = allowed_statuses
        return JsonRpcResponse(
            "2.0", request.id, {"ok": True, "canonical_nm": 5_080_000}, None
        )


class TestProjectDisplayUnitsRequest(unittest.TestCase):
    def test_client_wraps_complete_profile_in_guarded_native_write(self) -> None:
        daemon = CapturingDaemon()
        response = daemon.set_project_display_units(
            "/tmp/native-project", PROFILE, "model-revision-7"
        )

        self.assertEqual(response.result, {"status": "committed"})
        self.assertIsNotNone(daemon.request)
        self.assertEqual(daemon.request.method, "native.write")
        self.assertEqual(
            daemon.request.params,
            {
                "project_root": "/tmp/native-project",
                "verb": "datum.project.set_display_units",
                "params": {"profile": PROFILE},
                "actor": "datum-mcp",
                "source": "tool",
                "reason": "Set Project Working Units through MCP",
                "expected_model_revision": "model-revision-7",
                "dry_run": False,
            },
        )

    def test_registered_tool_dispatches_all_arguments_without_reinterpretation(self) -> None:
        daemon = CapturingDaemon()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 41,
                "method": "tools/call",
                "params": {
                    "name": "datum.project.set_display_units",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "profile": PROFILE,
                        "expected_model_revision": "model-revision-8",
                    },
                },
            }
        )

        payload = response["result"]["content"][0]["json"]
        self.assertTrue(payload["ok"])
        self.assertEqual(payload["schema"]["name"], "datum.project.set_display_units")
        self.assertEqual(payload["result"], {"status": "committed"})
        self.assertEqual(daemon.request.params["params"], {"profile": PROFILE})
        self.assertEqual(
            daemon.request.params["expected_model_revision"], "model-revision-8"
        )

    def test_resolve_length_mcp_forwards_explicit_context_to_the_cli_adapter(self) -> None:
        daemon = CapturingCliDaemon()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 42,
                "method": "tools/call",
                "params": {
                    "name": "datum.units.resolve_length",
                    "arguments": {
                        "expression": "200",
                        "quantity": "board",
                        "unit": "mil",
                        "system": "metric",
                        "field": "board.track.width",
                        "project_id": "sensor-node",
                    },
                },
            }
        )

        payload = response["result"]["content"][0]["json"]
        self.assertTrue(payload["ok"])
        self.assertEqual(payload["result"]["canonical_nm"], 5_080_000)
        self.assertEqual(
            daemon.args,
            [
                "units",
                "resolve-length",
                "--expression",
                "200",
                "--quantity",
                "board",
                "--unit",
                "mil",
                "--system",
                "metric",
                "--field",
                "board.track.width",
                "--project-id",
                "sensor-node",
            ],
        )
        self.assertEqual(daemon.allowed_statuses, {0, 2})


if __name__ == "__main__":
    unittest.main()
