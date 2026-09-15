#!/usr/bin/env python3
"""AI-MCP-01 discovery and protocol-clean stdio acceptance tests."""

from __future__ import annotations

from io import StringIO
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from agent_authority_test_support import write_agent_authority_fixture
from discovery_scope import load_discovery_scope
from stdio_tool_host import StdioToolHost
from test_support import FakeDaemonClient


class TestDiscoveryScope(unittest.TestCase):
    def test_valid_scope_is_loaded_and_exported(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve()
            discovery = root / "discovery.json"
            _, authority, environment = write_agent_authority_fixture(root, "terminal-7")
            discovery.write_text(
                json.dumps(
                    {
                        "schema": "datum_agent_discovery_v1",
                        "project_root": os.fspath(root),
                        "terminal_session_id": "terminal-7",
                        "context_id": "context-9",
                    }
                    | authority
                ),
                encoding="utf-8",
            )
            with patch.dict(os.environ, environment, clear=True):
                scope = load_discovery_scope(discovery)
                self.assertEqual(scope.project_root, root)
                self.assertEqual(scope.terminal_session_id, "terminal-7")
                self.assertEqual(os.environ["DATUM_AGENT_DISCOVERY"], os.fspath(discovery))

    def test_scope_mismatch_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve()
            discovery = root / "discovery.json"
            discovery.write_text(
                json.dumps(
                    {
                        "contract": "datum_terminal_context_v1",
                        "project_root": os.fspath(root),
                        "terminal_session_id": "terminal-real",
                    }
                ),
                encoding="utf-8",
            )
            with patch.dict(
                os.environ,
                {"DATUM_TERMINAL_SESSION_ID": "terminal-other"},
                clear=True,
            ):
                with self.assertRaisesRegex(ValueError, "does not match"):
                    load_discovery_scope(discovery)

    def test_unknown_schema_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve()
            discovery = root / "discovery.json"
            discovery.write_text(
                json.dumps(
                    {
                        "schema": "future_schema_v99",
                        "project_root": os.fspath(root),
                        "terminal_session_id": "terminal-1",
                    }
                ),
                encoding="utf-8",
            )
            with self.assertRaisesRegex(ValueError, "unsupported discovery schema"):
                load_discovery_scope(discovery)


class TestProtocolCleanStdio(unittest.TestCase):
    def test_only_json_rpc_messages_are_written_to_stdout(self) -> None:
        source = StringIO(
            "\n".join(
                [
                    '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}',
                    '{"jsonrpc":"2.0","method":"notifications/initialized"}',
                    '{"jsonrpc":"2.0","id":2,"method":"ping"}',
                    '{broken',
                    '[]',
                    '{"jsonrpc":"1.0","id":9,"method":"ping"}',
                    '{"jsonrpc":"2.0","id":3,"method":"unknown"}',
                ]
            )
            + "\n"
        )
        output = StringIO()
        StdioToolHost(FakeDaemonClient()).run_stdio(source, output)
        messages = [json.loads(line) for line in output.getvalue().splitlines()]
        self.assertEqual(
            [message["id"] for message in messages], [1, 2, None, None, None, 3]
        )
        self.assertEqual(messages[0]["result"]["serverInfo"]["name"], "datum-eda")
        self.assertEqual(messages[1]["result"], {})
        self.assertEqual(messages[2]["error"]["code"], -32700)
        self.assertEqual(messages[3]["error"]["code"], -32600)
        self.assertEqual(messages[4]["error"]["code"], -32600)
        self.assertEqual(messages[5]["error"]["code"], -32601)


if __name__ == "__main__":
    unittest.main()
