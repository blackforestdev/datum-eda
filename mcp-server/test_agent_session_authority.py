#!/usr/bin/env python3
"""AI-CTX-04 session revocation, secret isolation, and audit acceptance."""

from __future__ import annotations

import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from agent_audit import (
    AgentAuditWriter,
    agent_invocation,
    build_invocation_provenance,
    current_agent_subprocess_environment,
)
from agent_authority_test_support import write_agent_authority_fixture
from discovery_scope import load_discovery_scope
from library_authoring_methods import cli_run_kwargs_for_method
from stdio_tool_host import StdioToolHost
from test_support import FakeDaemonClient


class TestAgentSessionAuthority(unittest.TestCase):
    def setUp(self) -> None:
        self._temp = tempfile.TemporaryDirectory()
        self.root = Path(self._temp.name).resolve()
        self.live_path = self.root / "live.json"
        self.pinned_path = self.root / "pinned.json"
        self.discovery_path = self.root / "discovery.json"
        context_authority, discovery_authority, self.environment = (
            write_agent_authority_fixture(self.root, "terminal-revocation")
        )
        common = {
            "contract": "datum_terminal_context_v1",
            "project_root": os.fspath(self.root),
            "project_id": "project-revocation",
            "terminal_session_id": "terminal-revocation",
            "context_id": "context-revocation",
            "live_context_id": "live-terminal-revocation",
            "pinned_context_id": "context-revocation",
            "model_revision": "revision-revocation",
            "accepted_transaction_tip": "transaction-revocation",
            "capability_profile": "datum_agent_capability_v1",
            "capabilities": ["inspect", "propose"],
            "approval_policy": "owner-review-required",
            "unattended_tools": [],
        } | context_authority
        self._write(self.live_path, common | {"context_kind": "live"})
        self._write(self.pinned_path, common | {"context_kind": "pinned"})
        self._write(
            self.discovery_path,
            {
                "schema": "datum_agent_discovery_v1",
                "project_root": os.fspath(self.root),
                "terminal_session_id": "terminal-revocation",
                "live_context_id": "live-terminal-revocation",
                "live_context_path": os.fspath(self.live_path),
                "pinned_context_id": "context-revocation",
                "pinned_context_path": os.fspath(self.pinned_path),
                "model_revision": "revision-revocation",
                "accepted_transaction_tip": "transaction-revocation",
                "capability_profile": "datum_agent_capability_v1",
                "capabilities": ["inspect", "propose"],
                "approval_policy": "owner-review-required",
                "unattended_tools": [],
            }
            | discovery_authority,
        )
        with patch.dict(os.environ, self.environment, clear=True):
            self.scope = load_discovery_scope(self.discovery_path)
            self.daemon = FakeDaemonClient()
            self.host = StdioToolHost(self.daemon, self.scope)

    def tearDown(self) -> None:
        self._temp.cleanup()

    def test_same_broker_is_refused_after_descriptor_revocation(self) -> None:
        descriptor_path = Path(
            json.loads(self.discovery_path.read_text())["credential_descriptor"]
        )
        descriptor = json.loads(descriptor_path.read_text())
        descriptor["state"] = "revoked"
        self._write(descriptor_path, descriptor)
        response = self.host.handle_message(
            {"jsonrpc": "2.0", "id": 7, "method": "tools/list"}
        )
        self.assertEqual(response["error"]["code"], -32020)
        self.assertNotIn("secret", json.dumps(response))

    def test_terminal_lifecycle_change_revokes_tool_call_before_dispatch(self) -> None:
        live = json.loads(self.live_path.read_text())
        live["session_lifecycle"] = "terminating"
        self._write(self.live_path, live)
        response = self.host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 8,
                "method": "tools/call",
                "params": {
                    "name": "datum.proposal.show",
                    "arguments": {
                        "path": os.fspath(self.root),
                        "proposal": "proposal-revoked",
                    },
                },
            }
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["error"]["code"], "session_authority_revoked")
        self.assertEqual(self.daemon.calls, [])

    def test_secret_is_absent_from_discovery_context_and_audit(self) -> None:
        credential = json.loads(
            Path(self.environment["DATUM_AGENT_CREDENTIAL_FILE"]).read_text()
        )
        secret = credential["secret"]
        response = self.host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 9,
                "method": "tools/call",
                "params": {
                    "name": "datum.proposal.show",
                    "arguments": {
                        "path": os.fspath(self.root),
                        "proposal": "proposal-audit",
                    },
                },
            }
        )
        self.assertTrue(response["result"]["content"][0]["json"]["ok"])
        event_log = Path(
            json.loads(self.pinned_path.read_text())["storage"]["event_log_path"]
        )
        exposed = "\n".join(
            path.read_text()
            for path in [self.discovery_path, self.live_path, self.pinned_path, event_log]
        )
        self.assertNotIn(secret, exposed)
        record = json.loads(event_log.read_text().splitlines()[-1])
        self.assertEqual(record["agent_provenance"]["agent_identity"], "test-agent")
        self.assertEqual(record["agent_provenance"]["terminal_session_id"], "terminal-revocation")
        self.assertEqual(record["agent_provenance"]["context_id"], "context-revocation")
        self.assertEqual(record["agent_provenance"]["requested_capability"], "inspect")

    def test_commit_environment_and_audit_record_preserve_full_provenance(self) -> None:
        provenance = build_invocation_provenance(
            self.scope,
            "test-agent",
            "owner-review-required",
            "apply-approved",
            "datum.proposal.apply",
            {
                "context_id": "context-revocation",
                "expected_model_revision": "revision-revocation",
                "accepted_transaction_tip": "transaction-revocation",
                "proposal": "proposal-approved",
            },
        )
        with agent_invocation(provenance):
            environment = current_agent_subprocess_environment()
            run_environment = cli_run_kwargs_for_method("accept_apply_proposal")["env"]
        self.assertEqual(environment["DATUM_COMMIT_SOURCE"], "assistant")
        self.assertEqual(run_environment["DATUM_COMMIT_SOURCE"], "assistant")
        self.assertEqual(
            run_environment["DATUM_AGENT_PROVENANCE"],
            environment["DATUM_AGENT_PROVENANCE"],
        )
        serialized = json.loads(environment["DATUM_AGENT_PROVENANCE"])
        self.assertEqual(serialized["approval_reference"], "proposal-approved")

        AgentAuditWriter(self.scope).record(
            provenance,
            "succeeded",
            result={
                "batch_id": "batch-approved",
                "operations": [{"op": "set"}],
                "transaction_id": "transaction-committed",
                "before_model_revision": "revision-revocation",
                "after_model_revision": "revision-after",
                "diff": {"modified": [{"uuid": "object-1"}]},
            },
        )
        event_log = Path(
            json.loads(self.pinned_path.read_text())["storage"]["event_log_path"]
        )
        record = json.loads(event_log.read_text().splitlines()[-1])
        self.assertEqual(record["operation_batch"]["batch_id"], "batch-approved")
        self.assertEqual(record["operation_batch"]["operation_count"], 1)
        self.assertEqual(
            record["journal_result"]["transaction_id"], "transaction-committed"
        )
        self.assertEqual(record["diff"]["modified"][0]["uuid"], "object-1")

    @staticmethod
    def _write(path: Path, value: dict[str, object]) -> None:
        path.write_text(json.dumps(value), encoding="utf-8")
        if "authority" in path.name or "credential" in path.name:
            path.chmod(0o600)


if __name__ == "__main__":
    unittest.main()
