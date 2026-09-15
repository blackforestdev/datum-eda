#!/usr/bin/env python3
"""MCP dispatch coverage for Datum context session event streams."""
from __future__ import annotations
import unittest
from server_runtime import StdioToolHost
from test_support import FakeDaemonClient


class TestDispatchContextSessionEvents(unittest.TestCase):
    def test_tools_call_dispatches_datum_context_session_events(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 306,
                "method": "tools/call",
                "params": {
                    "name": "datum.context.session_events",
                    "arguments": {
                        "session": "session-test",
                        "path": "/tmp/context.json",
                        "project_root": "/tmp/native-project",
                        "event_kind": "terminal_command_handoff",
                        "origin": "production_terminal_command",
                        "command_id": "datum.artifact.generate",
                        "execution_id": "exec-test",
                        "limit": 1,
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "datum_context_session_events",
                    "session-test",
                    "/tmp/context.json",
                    "/tmp/native-project",
                    "terminal_command_handoff",
                    "production_terminal_command",
                    "datum.artifact.generate",
                    "exec-test",
                    1,
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertTrue(payload["ok"])
        self.assertEqual(
            payload["schema"],
            {"name": "datum.context.session_events", "version": 1},
        )
        self.assertEqual(payload["result"]["contract"], "datum_tool_session_events_v1")
        self.assertEqual(payload["result"]["context_provenance"]["context_id"], "context-test")
        self.assertEqual(payload["result"]["context_provenance"]["session_id"], "session-test")
        self.assertEqual(payload["result"]["event_count"], 1)
        self.assertEqual(payload["result"]["filters"]["event_kind"], "terminal_command_handoff")
        self.assertEqual(payload["result"]["filters"]["execution_id"], "exec-test")
        self.assertEqual(payload["result"]["limit"], 1)
        self.assertEqual(payload["event_count"], 1)

    def test_tools_call_dispatches_datum_context_session_activity(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 307,
                "method": "tools/call",
                "params": {
                    "name": "datum.context.session_activity",
                    "arguments": {
                        "session": "session-test",
                        "path": "/tmp/context.json",
                        "project_root": "/tmp/native-project",
                        "origin": "production_terminal_command",
                        "execution_id": "exec-test",
                        "limit": 1,
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "datum_context_session_activity",
                    "session-test",
                    "/tmp/context.json",
                    "/tmp/native-project",
                    None,
                    "production_terminal_command",
                    None,
                    "exec-test",
                    1,
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertTrue(payload["ok"])
        self.assertEqual(payload["schema"], {"name": "datum.context.session_activity", "version": 1})
        self.assertEqual(payload["result"]["contract"], "datum_tool_session_activity_summary_v1")
        self.assertEqual(payload["result"]["context_provenance"]["context_id"], "context-test")
        self.assertEqual(payload["result"]["activity_event_count"], 4)
        self.assertEqual(payload["result"]["terminal_io"]["input_event_count"], 1)
        self.assertEqual(payload["result"]["terminal_io"]["output_event_count"], 1)
        self.assertEqual(payload["result"]["terminal_io"]["last_output_preview"], "total 8\n")
        self.assertEqual(payload["result"]["executions"][0]["execution_id"], "exec-test")
        self.assertEqual(payload["result"]["executions"][0]["duration_ms"], 3)
        self.assertEqual(payload["result"]["executions"][0]["lifecycle"], "finished")
        self.assertEqual(payload["result"]["executions"][0]["process_exit_code"], 0)
        self.assertEqual(
            payload["result"]["executions"][0]["context_provenance"]["session_id"],
            "session-test",
        )
        self.assertEqual(
            payload["result"]["executions"][0]["terminal_io"]["output_byte_count"],
            12,
        )
        self.assertEqual(payload["result"]["activity_spans"][0]["span_id"], "span-000001")
        self.assertEqual(payload["result"]["activity_spans"][0]["span_kind"], "command")
        self.assertEqual(
            payload["result"]["activity_spans"][0]["handoff"]["command_id"],
            "datum.artifact.generate",
        )
        self.assertEqual(
            payload["result"]["activity_spans"][0]["handoff"]["context_provenance"]["context_id"],
            "context-test",
        )
        self.assertEqual(
            payload["result"]["activity_spans"][0]["terminal_io"]["output_byte_count"],
            12,
        )


if __name__ == "__main__":
    unittest.main()
