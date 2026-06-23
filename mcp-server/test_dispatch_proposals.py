#!/usr/bin/env python3
"""Proposal MCP tools/call dispatch tests."""

from __future__ import annotations

import unittest

from server_runtime import StdioToolHost
from test_support import FakeDaemonClient


class TestDispatchProposals(unittest.TestCase):
    def test_tools_call_dispatches_create_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 203, "method": "tools/call", "params": {"name": "create_proposal", "arguments": {"path": "/tmp/native-project", "batch": "/tmp/batch.json", "rationale": "review batch", "proposal": "proposal-test", "source": "assistant", "checks_run": ["check-test"], "finding_fingerprints": ["sha256:test"]}}}
        )
        self.assertEqual(daemon.calls, [("create_proposal", "/tmp/native-project", "/tmp/batch.json", "review batch", "proposal-test", "assistant", ["check-test"], ["sha256:test"])])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(
            payload["validation"]["policy"],
            "accepted_revision_guarded_source_policy_v1",
        )

    def test_tools_call_dispatches_create_draw_wire_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 212, "method": "tools/call", "params": {"name": "create_draw_wire_proposal", "arguments": {"path": "/tmp/native-project", "sheet": "sheet-uuid", "from_x_nm": 0, "from_y_nm": 10, "to_x_nm": 100, "to_y_nm": 110, "proposal": "proposal-wire", "rationale": "review wire"}}}
        )
        self.assertEqual(daemon.calls, [("create_draw_wire_proposal", "/tmp/native-project", "sheet-uuid", 0, 10, 100, 110, "proposal-wire", "review wire")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(payload["action"], "propose_draw_wire")

    def test_tools_call_dispatches_create_place_label_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 213, "method": "tools/call", "params": {"name": "create_place_label_proposal", "arguments": {"path": "/tmp/native-project", "sheet": "sheet-uuid", "name": "VCC", "x_nm": 100, "y_nm": 200, "kind": "power", "proposal": "proposal-label", "rationale": "review label"}}}
        )
        self.assertEqual(daemon.calls, [("create_place_label_proposal", "/tmp/native-project", "sheet-uuid", "VCC", 100, 200, "power", "proposal-label", "review label")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(payload["action"], "propose_place_label")

    def test_tools_call_dispatches_create_place_symbol_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 214, "method": "tools/call", "params": {"name": "create_place_symbol_proposal", "arguments": {"path": "/tmp/native-project", "sheet": "sheet-uuid", "reference": "U1", "value": "OPA", "x_nm": 100, "y_nm": 200, "lib_id": "Device:R", "rotation_deg": 90, "mirrored": True, "proposal": "proposal-symbol", "rationale": "review symbol"}}}
        )
        self.assertEqual(daemon.calls, [("create_place_symbol_proposal", "/tmp/native-project", "sheet-uuid", "U1", "OPA", 100, 200, "Device:R", 90, True, "proposal-symbol", "review symbol")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(payload["action"], "propose_place_symbol")

    def test_tools_call_dispatches_datum_alias_for_place_symbol_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 215, "method": "tools/call", "params": {"name": "datum.proposal.create_place_symbol", "arguments": {"path": "/tmp/native-project", "sheet": "sheet-uuid", "reference": "U2", "value": "BUF", "x_nm": 300, "y_nm": 400}}}
        )
        self.assertEqual(daemon.calls, [("create_place_symbol_proposal", "/tmp/native-project", "sheet-uuid", "U2", "BUF", 300, 400, None, None, None, "proposal-symbol-test", None)])
        self.assertEqual(response["result"]["content"][0]["json"]["action"], "propose_place_symbol")

    def test_tools_call_dispatches_get_proposals(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 204,
                "method": "tools/call",
                "params": {
                    "name": "get_proposals",
                    "arguments": {"path": "/tmp/native-project"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_proposals", "/tmp/native-project")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposals_query_v1")
        self.assertEqual(payload["proposal_count"], 1)

    def test_tools_call_dispatches_review_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 205, "method": "tools/call", "params": {"name": "review_proposal", "arguments": {"path": "/tmp/native-project", "proposal": "proposal-test", "status": "accepted"}}}
        )
        self.assertEqual(daemon.calls, [("review_proposal", "/tmp/native-project", "proposal-test", "accepted")])
        self.assertEqual(response["result"]["content"][0]["json"]["status"], "accepted")

    def test_tools_call_dispatches_show_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 207, "method": "tools/call", "params": {"name": "show_proposal", "arguments": {"path": "/tmp/native-project", "proposal": "proposal-test"}}}
        )
        self.assertEqual(daemon.calls, [("show_proposal", "/tmp/native-project", "proposal-test")])
        self.assertEqual(response["result"]["content"][0]["json"]["contract"], "proposal_show_v1")

    def test_tools_call_dispatches_preview_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 216, "method": "tools/call", "params": {"name": "preview_proposal", "arguments": {"path": "/tmp/native-project", "proposal": "proposal-test"}}}
        )
        self.assertEqual(daemon.calls, [("preview_proposal", "/tmp/native-project", "proposal-test")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_preview_v1")
        self.assertEqual(payload["diff"]["created"], ["object-test"])
        self.assertEqual(
            payload["validation"]["approval_path"],
            "draft_review_accept_then_apply",
        )

    def test_tools_call_dispatches_validate_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 208, "method": "tools/call", "params": {"name": "validate_proposal", "arguments": {"path": "/tmp/native-project", "proposal": "proposal-test"}}}
        )
        self.assertEqual(daemon.calls, [("validate_proposal", "/tmp/native-project", "proposal-test")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_validation_v1")
        self.assertEqual(payload["policy"], "accepted_revision_guarded_source_policy_v1")
        self.assertEqual(payload["approval_path"], "draft_review_accept_then_apply")
        self.assertEqual(payload["acceptance_required"], True)
        self.assertEqual(payload["current_revision_required"], True)
        self.assertEqual(payload["revision_guard_required"], True)
        self.assertEqual(payload["check_source_evidence_required"], True)

    def test_tools_call_dispatches_defer_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 209, "method": "tools/call", "params": {"name": "defer_proposal", "arguments": {"path": "/tmp/native-project", "proposal": "proposal-test"}}}
        )
        self.assertEqual(daemon.calls, [("defer_proposal", "/tmp/native-project", "proposal-test")])
        self.assertEqual(response["result"]["content"][0]["json"]["status"], "deferred")

    def test_tools_call_dispatches_reject_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 210, "method": "tools/call", "params": {"name": "reject_proposal", "arguments": {"path": "/tmp/native-project", "proposal": "proposal-test"}}}
        )
        self.assertEqual(daemon.calls, [("reject_proposal", "/tmp/native-project", "proposal-test")])
        self.assertEqual(response["result"]["content"][0]["json"]["status"], "rejected")

    def test_tools_call_dispatches_accept_apply_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 211, "method": "tools/call", "params": {"name": "accept_apply_proposal", "arguments": {"path": "/tmp/native-project", "proposal": "proposal-test"}}}
        )
        self.assertEqual(daemon.calls, [("accept_apply_proposal", "/tmp/native-project", "proposal-test")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["status"], "applied")
        self.assertEqual(payload["policy"], "accepted_revision_guarded_source_policy_v1")
        self.assertEqual(payload["validation"]["status"], "accepted")
        self.assertEqual(payload["validation"]["can_apply"], True)

    def test_tools_call_dispatches_apply_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 206, "method": "tools/call", "params": {"name": "apply_proposal", "arguments": {"path": "/tmp/native-project", "proposal": "proposal-test"}}}
        )
        self.assertEqual(daemon.calls, [("apply_proposal", "/tmp/native-project", "proposal-test")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["status"], "applied")
        self.assertEqual(payload["approval_path"], "draft_review_accept_then_apply")
        self.assertEqual(payload["validation"]["status"], "accepted")
        self.assertEqual(payload["validation"]["can_apply"], True)
