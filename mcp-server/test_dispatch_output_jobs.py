#!/usr/bin/env python3
"""OutputJob MCP tools/call dispatch tests."""

from __future__ import annotations

import unittest

from server_runtime import StdioToolHost
from test_support import FakeDaemonClient

class TestDispatchOutputJobs(unittest.TestCase):
    def test_tools_call_dispatches_generate_artifacts(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 89,
                "method": "tools/call",
                "params": {
                    "name": "generate_artifacts",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "output_dir": "/tmp/fab",
                        "include": "gerber-set",
                        "prefix": "doa2526",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "generate_artifacts",
                    {
                        "path": "/tmp/native-project",
                        "output_dir": "/tmp/fab",
                        "include": "gerber-set",
                        "prefix": "doa2526", "output_job": None,
                    },
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "artifact_generate_v1")
        self.assertEqual(payload["generated_count"], 1)

    def test_tools_call_dispatches_get_artifacts(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 90,
                "method": "tools/call",
                "params": {
                    "name": "get_artifacts",
                    "arguments": {"path": "/tmp/native-project"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_artifacts", "/tmp/native-project")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["action"], "artifact_metadata_list")
        self.assertEqual(payload["artifact_count"], 1)

    def test_tools_call_dispatches_show_artifact(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 91,
                "method": "tools/call",
                "params": {
                    "name": "show_artifact",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "artifact": "artifact-test",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "show_artifact",
                    {"path": "/tmp/native-project", "artifact": "artifact-test"},
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "artifact_metadata_v1")
        self.assertEqual(payload["artifact"]["artifact_id"], "artifact-test")

    def test_tools_call_dispatches_get_artifact_files(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 95,
                "method": "tools/call",
                "params": {
                    "name": "get_artifact_files",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "artifact": "artifact-test",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "get_artifact_files",
                    {"path": "/tmp/native-project", "artifact": "artifact-test"},
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "artifact_files_v1")
        self.assertEqual(payload["artifact_id"], "artifact-test")
        self.assertEqual(payload["file_count"], 1)
        self.assertEqual(payload["production_projection_count"], 1)

    def test_tools_call_dispatches_preview_artifact_file(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 96,
                "method": "tools/call",
                "params": {
                    "name": "preview_artifact_file",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "artifact": "artifact-test",
                        "file": "fab/doa2526.gbr",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "preview_artifact_file",
                    {
                        "path": "/tmp/native-project",
                        "artifact": "artifact-test",
                        "artifact_dir": None,
                        "file": "fab/doa2526.gbr",
                    },
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "artifact_file_preview_v1")
        self.assertEqual(payload["preview_kind"], "gerber_rs274x")
        self.assertEqual(payload["hash_matches_metadata"], True)

    def test_tools_call_dispatches_compare_artifacts(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 92,
                "method": "tools/call",
                "params": {
                    "name": "compare_artifacts",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "before": "artifact-before",
                        "after": "artifact-after",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "compare_artifacts",
                    {
                        "path": "/tmp/native-project",
                        "before": "artifact-before",
                        "after": "artifact-after",
                    },
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "artifact_metadata_compare_v1")
        self.assertEqual(payload["before_artifact_id"], "artifact-before")
        self.assertEqual(payload["after_artifact_id"], "artifact-after")

    def test_tools_call_dispatches_validate_artifact(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 93,
                "method": "tools/call",
                "params": {
                    "name": "validate_artifact",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "artifact": "artifact-test",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "validate_artifact",
                    {"path": "/tmp/native-project", "artifact": "artifact-test"},
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "artifact_metadata_validation_v1")
        self.assertEqual(payload["artifact_id"], "artifact-test")
        self.assertEqual(payload["valid"], True)

    def test_tools_call_dispatches_get_output_jobs(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 91,
                "method": "tools/call",
                "params": {
                    "name": "get_output_jobs",
                    "arguments": {"path": "/tmp/native-project"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_output_jobs", "/tmp/native-project")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["action"], "output_jobs")
        self.assertEqual(payload["output_job_count"], 1)

    def test_tools_call_dispatches_create_gerber_output_job(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 92,
                "method": "tools/call",
                "params": {
                    "name": "create_gerber_output_job",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "prefix": "fab/doa2526",
                        "name": "Fabrication Gerbers",
                        "manufacturing_plan": "fab/doa2526",
                        "output_dir": "/tmp/native-project/fab",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "create_gerber_output_job",
                    {
                        "path": "/tmp/native-project",
                        "prefix": "fab/doa2526",
                        "name": "Fabrication Gerbers",
                        "manufacturing_plan": "fab/doa2526",
                        "output_dir": "/tmp/native-project/fab",
                    },
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["action"], "create_gerber_output_job")
        self.assertEqual(payload["output_job"]["prefix"], "fab/doa2526")
        self.assertEqual(payload["output_job"]["manufacturing_plan"], "fab/doa2526")

    def test_tools_call_dispatches_create_output_job(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 98,
                "method": "tools/call",
                "params": {
                    "name": "create_output_job",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "prefix": "fab/doa2526",
                        "include": "drill",
                        "name": "Fabrication Drill",
                        "manufacturing_plan": "fab/doa2526",
                        "output_dir": "/tmp/native-project/fab",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "create_output_job",
                    {
                        "path": "/tmp/native-project",
                        "prefix": "fab/doa2526",
                        "include": "drill",
                        "name": "Fabrication Drill",
                        "manufacturing_plan": "fab/doa2526",
                        "output_dir": "/tmp/native-project/fab",
                    },
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["action"], "create_output_job")
        self.assertEqual(payload["output_job"]["include"], ["drill"])

    def test_tools_call_dispatches_update_output_job(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 99,
                "method": "tools/call",
                "params": {
                    "name": "update_output_job",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "output_job": "gerber-set-default",
                        "name": "Updated Gerbers",
                        "output_dir": "/tmp/native-project/fab",
                        "manufacturing_plan": None,
                        "clear_manufacturing_plan": True,
                        "clear_output_dir": False,
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "update_output_job",
                    {
                        "path": "/tmp/native-project",
                        "output_job": "gerber-set-default",
                        "name": "Updated Gerbers",
                        "output_dir": "/tmp/native-project/fab",
                        "manufacturing_plan": None,
                        "clear_manufacturing_plan": True,
                        "clear_output_dir": False,
                    },
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["action"], "update_output_job")
        self.assertEqual(payload["output_job"]["name"], "Updated Gerbers")

    def test_tools_call_dispatches_delete_output_job(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 100,
                "method": "tools/call",
                "params": {
                    "name": "delete_output_job",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "output_job": "gerber-set-default",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "delete_output_job",
                    {
                        "path": "/tmp/native-project",
                        "output_job": "gerber-set-default",
                    },
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["action"], "delete_output_job")
        self.assertEqual(payload["output_job"]["id"], "gerber-set-default")

    def test_tools_call_dispatches_manufacturing_set_evidence_tools(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        export_response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 97,
                "method": "tools/call",
                "params": {
                    "name": "export_manufacturing_set",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "output_dir": "/tmp/fab",
                        "prefix": "doa2526",
                    },
                },
            }
        )
        validate_response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 98,
                "method": "tools/call",
                "params": {
                    "name": "validate_manufacturing_set",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "output_dir": "/tmp/fab",
                        "prefix": "doa2526",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "export_manufacturing_set",
                    {
                        "path": "/tmp/native-project",
                        "output_dir": "/tmp/fab",
                        "prefix": "doa2526",
                    },
                ),
                (
                    "validate_manufacturing_set",
                    {
                        "path": "/tmp/native-project",
                        "output_dir": "/tmp/fab",
                        "prefix": "doa2526",
                    },
                ),
            ],
        )
        export_payload = export_response["result"]["content"][0]["json"]
        validate_payload = validate_response["result"]["content"][0]["json"]
        self.assertEqual(export_payload["action"], "export_manufacturing_set")
        self.assertEqual(
            export_payload["artifact_metadata"]["kind"], "manufacturing_set"
        )
        self.assertEqual(
            export_payload["output_job_run"]["output_job_kind"], "manufacturing_set"
        )
        self.assertEqual(validate_payload["action"], "validate_manufacturing_set")
        self.assertEqual(validate_payload["artifact_validation_state"], "valid")

    def test_tools_call_dispatches_panel_projection_tools(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        get_response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 93,
                "method": "tools/call",
                "params": {
                    "name": "get_panel_projections",
                    "arguments": {"path": "/tmp/native-project"},
                },
            }
        )
        create_response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 94,
                "method": "tools/call",
                "params": {
                    "name": "create_panel_projection",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "key": "main-panel",
                        "name": "Main Panel",
                        "board": "main",
                        "x_nm": 1000,
                        "y_nm": 2000,
                        "rotation_deg": 90,
                    },
                },
            }
        )
        update_response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 103,
                "method": "tools/call",
                "params": {
                    "name": "update_panel_projection",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "panel_projection": "main-panel",
                        "name": "Updated Panel",
                        "board": "main",
                        "x_nm": 3000,
                        "y_nm": 4000,
                        "rotation_deg": 180,
                    },
                },
            }
        )
        delete_response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 101,
                "method": "tools/call",
                "params": {
                    "name": "delete_panel_projection",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "panel_projection": "main-panel",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                ("get_panel_projections", "/tmp/native-project"),
                (
                    "create_panel_projection",
                    {
                        "path": "/tmp/native-project",
                        "key": "main-panel",
                        "name": "Main Panel",
                        "board": "main",
                        "x_nm": 1000,
                        "y_nm": 2000,
                        "rotation_deg": 90,
                    },
                ),
                (
                    "update_panel_projection",
                    {
                        "path": "/tmp/native-project",
                        "panel_projection": "main-panel",
                        "name": "Updated Panel",
                        "board": "main",
                        "x_nm": 3000,
                        "y_nm": 4000,
                        "rotation_deg": 180,
                    },
                ),
                (
                    "delete_panel_projection",
                    {
                        "path": "/tmp/native-project",
                        "panel_projection": "main-panel",
                    },
                ),
            ],
        )
        get_payload = get_response["result"]["content"][0]["json"]
        create_payload = create_response["result"]["content"][0]["json"]
        update_payload = update_response["result"]["content"][0]["json"]
        delete_payload = delete_response["result"]["content"][0]["json"]
        self.assertEqual(get_payload["panel_projection_count"], 1)
        self.assertEqual(create_payload["panel_projection"]["key"], "main-panel")
        self.assertEqual(update_payload["action"], "update_panel_projection")
        self.assertEqual(delete_payload["action"], "delete_panel_projection")

    def test_tools_call_dispatches_manufacturing_plan_tools(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        get_response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 95,
                "method": "tools/call",
                "params": {
                    "name": "get_manufacturing_plans",
                    "arguments": {"path": "/tmp/native-project"},
                },
            }
        )
        create_response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 96,
                "method": "tools/call",
                "params": {
                    "name": "create_manufacturing_plan",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "prefix": "fab/doa2526",
                        "name": "Fabrication Plan",
                        "variant": "default",
                        "panel_projection": "main-panel",
                    },
                },
            }
        )
        update_response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 104,
                "method": "tools/call",
                "params": {
                    "name": "update_manufacturing_plan",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "manufacturing_plan": "fab/doa2526",
                        "name": "Updated Fabrication Plan",
                        "prefix": "fab/doa2526-r2",
                        "variant": None,
                        "clear_variant": True,
                        "panel_projection": None,
                        "clear_panel_projection": False,
                    },
                },
            }
        )
        delete_response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 102,
                "method": "tools/call",
                "params": {
                    "name": "delete_manufacturing_plan",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "manufacturing_plan": "fab/doa2526",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                ("get_manufacturing_plans", "/tmp/native-project"),
                (
                    "create_manufacturing_plan",
                    {
                        "path": "/tmp/native-project",
                        "prefix": "fab/doa2526",
                        "name": "Fabrication Plan",
                        "variant": "default",
                        "panel_projection": "main-panel",
                    },
                ),
                (
                    "update_manufacturing_plan",
                    {
                        "path": "/tmp/native-project",
                        "manufacturing_plan": "fab/doa2526",
                        "name": "Updated Fabrication Plan",
                        "prefix": "fab/doa2526-r2",
                        "variant": None,
                        "clear_variant": True,
                        "panel_projection": None,
                        "clear_panel_projection": False,
                    },
                ),
                (
                    "delete_manufacturing_plan",
                    {
                        "path": "/tmp/native-project",
                        "manufacturing_plan": "fab/doa2526",
                    },
                ),
            ],
        )
        get_payload = get_response["result"]["content"][0]["json"]
        create_payload = create_response["result"]["content"][0]["json"]
        update_payload = update_response["result"]["content"][0]["json"]
        delete_payload = delete_response["result"]["content"][0]["json"]
        self.assertEqual(get_payload["manufacturing_plan_count"], 1)
        self.assertEqual(
            create_payload["manufacturing_plan"]["panel_projection"], "main-panel"
        )
        self.assertEqual(update_payload["action"], "update_manufacturing_plan")
        self.assertEqual(delete_payload["action"], "delete_manufacturing_plan")
if __name__ == "__main__":
    unittest.main()
