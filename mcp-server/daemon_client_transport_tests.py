#!/usr/bin/env python3
"""Engine daemon client response decoding and socket transport tests."""

from __future__ import annotations

import json
import os
import socket
import tempfile
import threading
import unittest

from server_runtime import EngineDaemonClient, JsonRpcResponse


class TestDaemonClientTransport(unittest.TestCase):
    def test_response_decodes_success_payload(self) -> None:
        response = JsonRpcResponse.from_json(
            json.dumps(
                {
                    "jsonrpc": "2.0",
                    "id": 7,
                    "result": {
                        "domain": "board",
                        "summary": {"status": "warning"},
                        "diagnostics": [],
                    },
                    "error": None,
                }
            )
        )
        self.assertEqual(response.id, 7)
        self.assertIsNone(response.error)
        assert isinstance(response.result, dict)
        self.assertEqual(response.result["domain"], "board")

    def test_response_decodes_error_payload(self) -> None:
        response = JsonRpcResponse.from_json(
            json.dumps(
                {
                    "jsonrpc": "2.0",
                    "id": 9,
                    "result": None,
                    "error": {"code": -32001, "message": "no project open"},
                }
            )
        )
        self.assertIsNone(response.result)
        self.assertIsNotNone(response.error)
        assert response.error is not None
        self.assertEqual(response.error.code, -32001)
        self.assertEqual(response.error.message, "no project open")

    def test_call_requires_socket_configuration(self) -> None:
        client = EngineDaemonClient(socket_path=None)
        with self.assertRaisesRegex(RuntimeError, "EDA_ENGINE_SOCKET is not configured"):
            client.get_check_report()

    def test_get_check_report_round_trips_over_unix_socket(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            socket_path = os.path.join(tmp, "eda.sock")
            probe = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            try:
                probe.bind(socket_path)
            except PermissionError as exc:
                self.skipTest(f"unix socket bind not permitted in this environment: {exc}")
            finally:
                probe.close()
                if os.path.exists(socket_path):
                    os.unlink(socket_path)
            ready = threading.Event()

            def serve_once() -> None:
                with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as server:
                    server.bind(socket_path)
                    server.listen(1)
                    ready.set()
                    conn, _ = server.accept()
                    with conn:
                        data = b""
                        while not data.endswith(b"\n"):
                            chunk = conn.recv(4096)
                            if not chunk:
                                break
                            data += chunk
                        request = json.loads(data.decode("utf-8").strip())
                        self.assertEqual(request["method"], "get_check_report")
                        response = json.dumps(
                            {
                                "jsonrpc": "2.0",
                                "id": request["id"],
                                "result": {
                                    "domain": "board",
                                    "summary": {
                                        "status": "warning",
                                        "errors": 0,
                                        "warnings": 1,
                                        "infos": 1,
                                        "waived": 0,
                                        "by_code": [
                                            {"code": "partially_routed_net", "count": 1},
                                            {"code": "net_without_copper", "count": 1},
                                        ],
                                    },
                                    "diagnostics": [
                                        {
                                            "kind": "partially_routed_net",
                                            "severity": "warning",
                                        },
                                        {"kind": "net_without_copper", "severity": "info"},
                                    ],
                                },
                                "error": None,
                            }
                        )
                        conn.sendall(response.encode("utf-8") + b"\n")

            thread = threading.Thread(target=serve_once)
            thread.start()
            ready.wait(timeout=2)

            client = EngineDaemonClient(socket_path=socket_path)
            response = client.get_check_report()
            self.assertIsNone(response.error)
            assert isinstance(response.result, dict)
            self.assertEqual(response.result["domain"], "board")
            self.assertEqual(response.result["summary"]["status"], "warning")
            self.assertEqual(
                response.result["summary"]["by_code"][0]["code"], "partially_routed_net"
            )
            thread.join(timeout=2)
            self.assertFalse(thread.is_alive())
