#!/usr/bin/env python3
"""AI-MCP-02 loopback Streamable HTTP acceptance tests."""

from __future__ import annotations

import http.client
import json
import os
from pathlib import Path
import tempfile
import threading
import unittest

from stdio_tool_host import StdioToolHost
from streamable_http import create_http_server, load_bearer_token
from test_support import FakeDaemonClient


TOKEN = "datum-test-token"
ORIGIN = "http://127.0.0.1:9911"


class TestStreamableHttp(unittest.TestCase):
    def setUp(self) -> None:
        self.server = create_http_server(
            0, TOKEN, [ORIGIN], StdioToolHost(FakeDaemonClient())
        )
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()

    def tearDown(self) -> None:
        self.server.shutdown()
        self.server.server_close()
        self.thread.join(timeout=2)

    def request(
        self,
        body: dict[str, object],
        *,
        token: str = TOKEN,
        origin: str | None = None,
        protocol: str = "2025-06-18",
    ) -> tuple[int, bytes]:
        connection = http.client.HTTPConnection("127.0.0.1", self.server.server_port)
        headers = {
            "Authorization": f"Bearer {token}",
            "Accept": "application/json, text/event-stream",
            "Content-Type": "application/json",
            "MCP-Protocol-Version": protocol,
        }
        if origin is not None:
            headers["Origin"] = origin
        connection.request("POST", "/mcp", json.dumps(body), headers)
        response = connection.getresponse()
        payload = response.read()
        status = response.status
        connection.close()
        return status, payload

    def test_initialize_is_authenticated_json_and_stateless(self) -> None:
        status, payload = self.request(
            {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {"protocolVersion": "2025-06-18"},
            }
        )
        self.assertEqual(status, 200)
        response = json.loads(payload)
        self.assertEqual(response["id"], 1)
        self.assertEqual(response["result"]["serverInfo"]["name"], "datum-eda")

    def test_notification_returns_accepted_without_body(self) -> None:
        status, payload = self.request(
            {"jsonrpc": "2.0", "method": "notifications/initialized"}
        )
        self.assertEqual((status, payload), (202, b""))

    def test_auth_origin_and_protocol_fail_closed(self) -> None:
        request = {"jsonrpc": "2.0", "id": 3, "method": "ping"}
        self.assertEqual(self.request(request, token="wrong")[0], 401)
        self.assertEqual(self.request(request, origin="https://attacker.invalid")[0], 403)
        self.assertEqual(self.request(request, origin=ORIGIN)[0], 200)
        self.assertEqual(self.request(request, protocol="future-v99")[0], 400)

    def test_get_and_delete_decline_unsupported_session_streams(self) -> None:
        for method in ("GET", "DELETE"):
            connection = http.client.HTTPConnection("127.0.0.1", self.server.server_port)
            connection.request(method, "/mcp")
            response = connection.getresponse()
            response.read()
            self.assertEqual(response.status, 405)
            self.assertEqual(response.headers["Allow"], "POST")
            connection.close()


class TestBearerTokenFile(unittest.TestCase):
    def test_token_file_must_be_private(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "token"
            path.write_text(TOKEN, encoding="ascii")
            os.chmod(path, 0o600)
            self.assertEqual(load_bearer_token(path), TOKEN)
            os.chmod(path, 0o644)
            with self.assertRaisesRegex(ValueError, "group or other"):
                load_bearer_token(path)


if __name__ == "__main__":
    unittest.main()
