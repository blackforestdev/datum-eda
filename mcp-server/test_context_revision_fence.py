#!/usr/bin/env python3
"""AI-CTX-02 pinned-context and revision-fence acceptance tests."""

from __future__ import annotations

import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from discovery_scope import load_discovery_scope
from stdio_tool_host import StdioToolHost
from test_support import FakeDaemonClient
from tools_catalog_data import TOOL_BY_NAME


class TestContextRevisionFence(unittest.TestCase):
    def setUp(self) -> None:
        self._temp = tempfile.TemporaryDirectory()
        self.root = Path(self._temp.name).resolve()
        self.live_path = self.root / "live.json"
        self.pinned_path = self.root / "pinned.json"
        self.discovery_path = self.root / "discovery.json"
        common = {
            "contract": "datum_terminal_context_v1",
            "project_root": os.fspath(self.root),
            "project_id": "project-fenced",
            "terminal_session_id": "terminal-fenced",
            "context_id": "context-pinned",
            "live_context_id": "live-terminal-fenced",
            "pinned_context_id": "context-pinned",
            "model_revision": "revision-pinned",
            "accepted_transaction_tip": "transaction-pinned",
            "capability_profile": "datum_agent_capability_v1",
            "capabilities": ["inspect", "propose", "apply-approved", "unattended"],
            "approval_policy": "owner-enabled-unattended",
            "unattended_tools": ["datum.proposal.accept_apply"],
            "selection_context": {
                "id": "component-selected",
                "object_ids": ["net-selected"],
            },
        }
        self.live_document = common | {"context_kind": "live"}
        self.pinned_document = common | {"context_kind": "pinned"}
        self._write_json(self.live_path, self.live_document)
        self._write_json(self.pinned_path, self.pinned_document)
        self._write_json(
            self.discovery_path,
            {
                "schema": "datum_agent_discovery_v1",
                "project_root": os.fspath(self.root),
                "terminal_session_id": "terminal-fenced",
                "live_context_id": "live-terminal-fenced",
                "live_context_path": os.fspath(self.live_path),
                "pinned_context_id": "context-pinned",
                "pinned_context_path": os.fspath(self.pinned_path),
                "model_revision": "revision-pinned",
                "accepted_transaction_tip": "transaction-pinned",
                "capability_profile": "datum_agent_capability_v1",
                "capabilities": [
                    "inspect",
                    "propose",
                    "apply-approved",
                    "unattended",
                ],
                "approval_policy": "owner-enabled-unattended",
                "unattended_tools": ["datum.proposal.accept_apply"],
            },
        )
        with patch.dict(os.environ, {}, clear=True):
            scope = load_discovery_scope(self.discovery_path)
        self.daemon = FakeDaemonClient()
        self.host = StdioToolHost(self.daemon, scope)

    def tearDown(self) -> None:
        self._temp.cleanup()

    def test_scoped_catalog_requires_fence_on_proposal_and_apply_envelopes(self) -> None:
        response = self.host.handle_message(
            {"jsonrpc": "2.0", "id": 1, "method": "tools/list"}
        )
        tools = {tool["name"]: tool for tool in response["result"]["tools"]}
        write_tools = {
            name
            for name, spec in TOOL_BY_NAME.items()
            if name in tools and spec.get("x_public_write_surface_class")
        }
        self.assertIn("datum.proposal.create", write_tools)
        self.assertIn("datum.proposal.accept_apply", write_tools)
        for name in write_tools:
            required = tools[name]["inputSchema"]["required"]
            self.assertIn("context_id", required)
            self.assertIn("expected_model_revision", required)
            self.assertIn("accepted_transaction_tip", required)
        self.assertNotIn(
            "context_id", tools["datum.proposal.show"]["inputSchema"]["properties"]
        )

    def test_missing_fence_fails_closed_without_daemon_dispatch(self) -> None:
        payload = self._call({})
        self.assertFalse(payload["ok"])
        self.assertEqual(payload["error"]["code"], "context_fence_required")
        self.assertEqual(
            payload["error"]["details"]["missing_fields"],
            ["accepted_transaction_tip", "context_id", "expected_model_revision"],
        )
        self.assertEqual(self.daemon.calls, [])

    def test_discovery_cannot_relabel_the_immutable_pinned_revision(self) -> None:
        discovery = json.loads(self.discovery_path.read_text(encoding="utf-8"))
        discovery["model_revision"] = "revision-tampered"
        self._write_json(self.discovery_path, discovery)
        with patch.dict(os.environ, {}, clear=True):
            with self.assertRaisesRegex(ValueError, "pinned model_revision mismatch"):
                load_discovery_scope(self.discovery_path)

    def test_discovery_cannot_relabel_the_accepted_transaction_tip(self) -> None:
        discovery = json.loads(self.discovery_path.read_text(encoding="utf-8"))
        discovery["accepted_transaction_tip"] = "transaction-tampered"
        self._write_json(self.discovery_path, discovery)
        with patch.dict(os.environ, {}, clear=True):
            with self.assertRaisesRegex(
                ValueError, "pinned accepted_transaction_tip mismatch"
            ):
                load_discovery_scope(self.discovery_path)

    def test_wrong_context_returns_structured_refresh_and_rebase_options(self) -> None:
        payload = self._call(self._fence(context_id="context-other"))
        error = payload["error"]
        self.assertEqual(error["code"], "stale_context")
        self.assertEqual(error["details"]["reason"], "context_id_mismatch")
        self.assertEqual(
            error["details"]["affected_ids"],
            ["component-selected", "net-selected"],
        )
        self.assertEqual(
            error["details"]["options"]["refresh"]["resource"],
            "datum://context/live",
        )
        self.assertEqual(
            error["details"]["options"]["rebase"]["resource"],
            "datum://context/pinned/context-pinned",
        )
        self.assertEqual(self.daemon.calls, [])

    def test_live_revision_advance_refuses_stale_pinned_request(self) -> None:
        self._write_json(
            self.live_path,
            self.live_document
            | {
                "model_revision": "revision-current",
                "accepted_transaction_tip": "transaction-current",
            },
        )
        payload = self._call(self._fence())
        details = payload["error"]["details"]
        self.assertEqual(payload["error"]["code"], "stale_context")
        self.assertEqual(details["reason"], "live_revision_advanced")
        self.assertEqual(details["pinned"]["model_revision"], "revision-pinned")
        self.assertEqual(details["current"]["model_revision"], "revision-current")
        self.assertEqual(self.daemon.calls, [])

    def test_live_selection_change_does_not_retarget_the_pinned_request(self) -> None:
        self._write_json(
            self.live_path,
            self.live_document
            | {
                "selection_context": {
                    "id": "component-live-other",
                    "object_ids": ["net-live-other"],
                }
            },
        )
        payload = self._call(self._fence())
        self.assertTrue(payload["ok"])
        self.assertEqual(len(self.daemon.calls), 1)

    def test_expected_revision_and_transaction_tip_must_match_the_pin(self) -> None:
        wrong_revision = self._fence() | {"expected_model_revision": "revision-other"}
        revision_payload = self._call(wrong_revision)
        self.assertEqual(
            revision_payload["error"]["details"]["reason"],
            "pinned_revision_mismatch",
        )
        wrong_tip = self._fence() | {
            "accepted_transaction_tip": "transaction-other"
        }
        tip_payload = self._call(wrong_tip)
        self.assertEqual(
            tip_payload["error"]["details"]["reason"],
            "pinned_transaction_tip_mismatch",
        )
        self.assertEqual(self.daemon.calls, [])

    def test_valid_fence_is_consumed_before_dispatch(self) -> None:
        payload = self._call(self._fence())
        self.assertTrue(payload["ok"])
        self.assertEqual(
            self.daemon.calls,
            [
                (
                    "create_proposal",
                    os.fspath(self.root),
                    "/tmp/batch.json",
                    "review batch",
                    "proposal-fenced",
                    "assistant",
                    [],
                    [],
                )
            ],
        )

    def test_valid_apply_envelope_is_fenced_then_dispatched(self) -> None:
        response = self.host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 10,
                "method": "tools/call",
                "params": {
                    "name": "datum.proposal.accept_apply",
                    "arguments": {
                        "path": os.fspath(self.root),
                        "proposal": "proposal-fenced",
                    }
                    | self._fence(),
                },
            }
        )
        payload = response["result"]["content"][0]["json"]
        self.assertTrue(payload["ok"])
        self.assertEqual(
            self.daemon.calls,
            [
                (
                    "accept_apply_proposal",
                    os.fspath(self.root),
                    "proposal-fenced",
                )
            ],
        )

    def test_unreadable_live_authority_returns_typed_unavailable_refusal(self) -> None:
        self.live_path.write_text("{broken", encoding="utf-8")
        payload = self._call(self._fence())
        self.assertEqual(payload["error"]["code"], "context_fence_unavailable")
        self.assertEqual(
            payload["error"]["details"]["reason"],
            "authority_document_unavailable",
        )
        self.assertIn("live context", payload["error"]["details"]["authority_error"])
        self.assertEqual(self.daemon.calls, [])

    def test_live_identity_change_returns_typed_unavailable_refusal(self) -> None:
        self._write_json(
            self.live_path,
            self.live_document | {"terminal_session_id": "terminal-other"},
        )
        payload = self._call(self._fence())
        self.assertEqual(payload["error"]["code"], "context_fence_unavailable")
        self.assertEqual(
            payload["error"]["details"]["reason"],
            "authority_document_unavailable",
        )
        self.assertIn(
            "session identity mismatch",
            payload["error"]["details"]["authority_error"],
        )
        self.assertEqual(self.daemon.calls, [])

    def test_hidden_compatibility_alias_cannot_bypass_fence(self) -> None:
        response = self.host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 9,
                "method": "tools/call",
                "params": {
                    "name": "create_proposal",
                    "arguments": self._proposal_arguments(),
                },
            }
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["error"]["code"], "context_fence_required")
        self.assertEqual(self.daemon.calls, [])

    def _call(self, fence: dict[str, object]) -> dict:
        response = self.host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 8,
                "method": "tools/call",
                "params": {
                    "name": "datum.proposal.create",
                    "arguments": self._proposal_arguments() | fence,
                },
            }
        )
        return response["result"]["content"][0]["json"]

    def _proposal_arguments(self) -> dict[str, object]:
        return {
            "path": os.fspath(self.root),
            "batch": "/tmp/batch.json",
            "rationale": "review batch",
            "proposal": "proposal-fenced",
            "source": "assistant",
            "checks_run": [],
            "finding_fingerprints": [],
        }

    @staticmethod
    def _fence(
        *, context_id: str = "context-pinned"
    ) -> dict[str, object]:
        return {
            "context_id": context_id,
            "expected_model_revision": "revision-pinned",
            "accepted_transaction_tip": "transaction-pinned",
        }

    @staticmethod
    def _write_json(path: Path, value: dict[str, object]) -> None:
        path.write_text(json.dumps(value), encoding="utf-8")


if __name__ == "__main__":
    unittest.main()
