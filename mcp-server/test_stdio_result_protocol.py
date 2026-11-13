"""Receiver checks for MCP tools/call text results across negotiated versions.

Contract: modelcontextprotocol.io/specification/{version}/server/tools.
No host serializer helpers are used by the receiver.
"""
from io import StringIO
import json
import unittest
from unittest.mock import patch

from server_runtime import JsonRpcError, JsonRpcResponse
from stdio_tool_host import StdioToolHost
from test_support import FakeDaemonClient
from test_agent_capability import ScopedHostFixture

VERSIONS = ("2024-11-05", "2025-03-26", "2025-06-18")


def receive_tool_result(response, *, failed=False):
    assert response["jsonrpc"] == "2.0"
    assert "error" not in response
    result = response["result"]
    assert isinstance(result["content"], list) and result["content"]
    # Datum uses the text alternative supported by all three MCP schemas.
    for block in result["content"]:
        assert block["type"] == "text", block
        assert isinstance(block["text"], str)
        assert "json" not in block
    assert result.get("isError", False) is failed
    return json.loads(result["content"][0]["text"])


class TestStdioResultProtocol(unittest.TestCase):
    def exchange(self, host, version, name, arguments, *, initialize=True):
        messages = (
            {"jsonrpc": "2.0", "id": 1, "method": "initialize",
             "params": {"protocolVersion": version}},
            {"jsonrpc": "2.0", "method": "notifications/initialized"},
            {"jsonrpc": "2.0", "id": 2, "method": "tools/call",
             "params": {"name": name, "arguments": arguments}},
        )
        source = StringIO("\n".join(json.dumps(message) for message in (messages if initialize else messages[2:])) + "\n")
        output = StringIO()
        host.run_stdio(source, output)
        responses = list(map(json.loads, output.getvalue().splitlines()))
        response = responses[-1]
        if initialize:
            self.assertEqual(responses[0]["result"]["protocolVersion"], version)
        self.assertEqual(response["id"], 2)
        return response

    def test_success_preserves_complete_envelope_and_context(self):
        original = {"ok": True, "schema": {"name": "datum.query.board_summary", "version": 1},
                    "context": {"project_id": "p", "model_revision": "r"},
                    "result": {"text": "µΩ\nboard", "items": [1, None]},
                    "provenance": {"invocation": "original"}}
        for version in VERSIONS:
            with self.subTest(version=version):
                daemon = FakeDaemonClient()
                with patch.object(daemon, "get_board_summary", return_value=JsonRpcResponse("2.0", 1, original, None)):
                    payload = receive_tool_result(self.exchange(StdioToolHost(daemon), version, "datum.query.board_summary", {}))
                self.assertEqual(payload, original)

    def test_execution_error_is_tool_result(self):
        for version in VERSIONS:
            with self.subTest(version=version):
                daemon = FakeDaemonClient()
                with patch.object(daemon, "get_board_summary", return_value=JsonRpcResponse("2.0", 1, None, JsonRpcError(-1, "design unavailable"))):
                    payload = receive_tool_result(self.exchange(StdioToolHost(daemon), version, "datum.query.board_summary", {}), failed=True)
                self.assertFalse(payload["ok"])
                self.assertEqual(payload["error"]["message"], "design unavailable")
                self.assertEqual(payload["schema"]["name"], "datum.query.board_summary")

    def test_compatibility_array_and_embedded_failure_payloads(self):
        for version in VERSIONS:
            with self.subTest(version=version):
                host = StdioToolHost(FakeDaemonClient())
                payload = receive_tool_result(self.exchange(host, version, "get_labels", {}))
                self.assertIsInstance(payload, list)
                self.assertEqual(payload[0]["name"], "SCL")
                original = {"ok": False, "schema": {"name": "datum.query.board_summary", "version": 1},
                            "context": {"model_revision": "r"},
                            "error": {"code": "domain_failure", "details": {"untouched": [1, 2]}}}
                with patch.object(host._daemon, "get_board_summary", return_value=JsonRpcResponse("2.0", 1, original, None)):
                    failed = receive_tool_result(self.exchange(host, version, "datum.query.board_summary", {}), failed=True)
                self.assertEqual(failed, original)

    def test_authority_and_revocation_failures_are_tool_results(self):
        for version in VERSIONS:
            fixture = ScopedHostFixture(["inspect"])
            try:
                with self.subTest(version=version, error="capability"):
                    payload = receive_tool_result(self.exchange(fixture.host, version, "datum.proposal.apply", {}), failed=True)
                    self.assertFalse(payload["ok"])
                    self.assertEqual(fixture.daemon.calls, [])
                with self.subTest(version=version, error="revocation"):
                    live = json.loads(fixture.live_path.read_text())
                    live["session_lifecycle"] = "terminating"
                    fixture.live_path.write_text(json.dumps(live))
                    payload = receive_tool_result(self.exchange(fixture.host, version, "datum.query.board_summary", {}, initialize=False), failed=True)
                    self.assertEqual(payload["error"]["code"], "session_authority_revoked")
            finally:
                fixture.close()

    def test_unknown_and_malformed_calls_are_protocol_errors(self):
        for params in ({"name": "datum.not_a_tool"}, {}, [],
                       {"name": []}, {"name": "datum.query.board_summary", "arguments": []}):
            with self.subTest(params=params):
                response = StdioToolHost(FakeDaemonClient()).handle_message(
                    {"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": params})
                self.assertEqual(response["error"]["code"], -32602)
                self.assertNotIn("result", response)

    def test_receiver_rejects_original_nonstandard_json_block(self):
        with self.assertRaises(AssertionError):
            receive_tool_result({"jsonrpc": "2.0", "result": {"content": [{"type": "json", "json": {"ok": True}}]}})


if __name__ == "__main__":
    unittest.main()
