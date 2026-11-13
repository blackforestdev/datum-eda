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
from tools_catalog_data import TOOL_BY_NAME
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

    def test_revision_query_stays_quarantined_at_active_admission(self):
        self.assertNotIn("datum.revision.query", TOOL_BY_NAME)
        host = StdioToolHost(FakeDaemonClient())
        for version in VERSIONS:
            response = self.exchange(host, version, "datum.revision.query",
                                     {"project_root": "p", "query": "release_readiness"})
            self.assertEqual(response["error"]["code"], -32602)
            self.assertIn("unknown tool", response["error"]["message"])
            self.assertNotIn("result", response)

    def test_revision_refusal_adapter_preserves_complete_current_and_history_packet(self):
        # Adapter-only unit: this test descriptor is removed after each case.
        # It does not restore the quarantined production catalog or run Rust.
        spec = {"name": "datum.revision.query", "x_dispatch_method": "revision_query",
                "x_dispatch_args": ["project_root", "query", "as_of_sequence", "expected_model_revision"],
                "inputSchema": {"properties": {"project_root": {}, "query": {},
                    "as_of_sequence": {}, "expected_model_revision": {}},
                    "required": ["project_root", "query"]}}
        for version in VERSIONS:
            for sequence in (None, 0, 99):
                original = {"schema_version": 1, "project_id": "p", "model_revision": "r",
                    "transaction_tip": "t", "query": "release_readiness", "as_of_sequence": sequence,
                    "result": None, "refusal": {"code": "unsupported_algorithm",
                        "affected_identities": ["p"], "controlling_policy_or_requirement": "PM-037 public revision service",
                        "expected_facts": ["engineering evaluation for release_readiness"],
                        "current_facts": ["no public evaluator"], "witness": ["engine revision public service"],
                        "remediation": "use revision.catalog for discovery"}}
                before = json.loads(json.dumps(original))
                daemon = FakeDaemonClient()
                with self.subTest(version=version, sequence=sequence), patch.dict(TOOL_BY_NAME, {"datum.revision.query": spec}), patch.object(
                    daemon, "revision_query", create=True, return_value=JsonRpcResponse("2.0", 1, original, None)
                ) as call:
                    payload = receive_tool_result(self.exchange(StdioToolHost(daemon), version,
                        "datum.revision.query", {"project_root": "p", "query": "release_readiness",
                        "as_of_sequence": sequence, "expected_model_revision": "r"}), failed=True)
                    call.assert_called_once_with("p", "release_readiness", sequence, "r")
                self.assertFalse(payload["ok"])
                self.assertEqual(payload["result"], before)
                self.assertEqual(payload["error"]["details"], before["refusal"])
                self.assertEqual(payload["error"]["code"], "unsupported_algorithm")
                self.assertEqual(payload["context"]["project_id"], "p")
                self.assertEqual(payload["context"]["model_revision"], "r")
                self.assertEqual(original, before)
        self.assertNotIn("datum.revision.query", TOOL_BY_NAME)

    def test_revision_adapter_rejects_selectors_before_dispatch_drops_them(self):
        spec = {"name": "datum.revision.query", "x_dispatch_method": "revision_query",
                "x_dispatch_args": ["project_root", "query"],
                "inputSchema": {"properties": {"project_root": {}, "query": {}},
                    "required": ["project_root", "query"]}}
        for selector in ("subject", "subject_id", "baseline_id", "release_id"):
            daemon = FakeDaemonClient()
            with self.subTest(selector=selector), patch.dict(TOOL_BY_NAME, {"datum.revision.query": spec}), patch.object(
                daemon, "revision_query", create=True
            ) as call:
                payload = receive_tool_result(self.exchange(StdioToolHost(daemon), VERSIONS[-1],
                    "datum.revision.query", {"project_root": "p", "query": "release_readiness",
                    selector: "unsupported"}), failed=True)
                call.assert_not_called()
            self.assertFalse(payload["ok"])
            self.assertIn(selector, payload["error"]["message"])
        self.assertNotIn("datum.revision.query", TOOL_BY_NAME)

    def test_receiver_rejects_original_nonstandard_json_block(self):
        with self.assertRaises(AssertionError):
            receive_tool_result({"jsonrpc": "2.0", "result": {"content": [{"type": "json", "json": {"ok": True}}]}})


if __name__ == "__main__":
    unittest.main()
