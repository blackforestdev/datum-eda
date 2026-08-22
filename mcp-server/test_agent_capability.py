#!/usr/bin/env python3
"""AI-CTX-03 scoped capability-grant acceptance tests."""

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


class ScopedHostFixture:
    def __init__(
        self,
        capabilities: list[str],
        *,
        unattended_tools: list[str] | None = None,
    ) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name).resolve()
        self.live_path = self.root / "live.json"
        self.pinned_path = self.root / "pinned.json"
        self.discovery_path = self.root / "discovery.json"
        unattended_tools = unattended_tools or []
        approval_policy = (
            "owner-enabled-unattended"
            if "unattended" in capabilities
            else "owner-review-required"
        )
        authority = {
            "capability_profile": "datum_agent_capability_v1",
            "capabilities": capabilities,
            "approval_policy": approval_policy,
            "unattended_tools": unattended_tools,
        }
        common = {
            "contract": "datum_terminal_context_v1",
            "project_root": os.fspath(self.root),
            "project_id": "project-authority",
            "terminal_session_id": "terminal-authority",
            "context_id": "context-authority",
            "live_context_id": "live-terminal-authority",
            "pinned_context_id": "context-authority",
            "model_revision": "revision-authority",
            "accepted_transaction_tip": "transaction-authority",
        } | authority
        self._write(self.live_path, common | {"context_kind": "live"})
        self._write(self.pinned_path, common | {"context_kind": "pinned"})
        self._write(
            self.discovery_path,
            {
                "schema": "datum_agent_discovery_v1",
                "project_root": os.fspath(self.root),
                "terminal_session_id": "terminal-authority",
                "live_context_id": "live-terminal-authority",
                "live_context_path": os.fspath(self.live_path),
                "pinned_context_id": "context-authority",
                "pinned_context_path": os.fspath(self.pinned_path),
                "model_revision": "revision-authority",
                "accepted_transaction_tip": "transaction-authority",
            }
            | authority,
        )
        with patch.dict(os.environ, {}, clear=True):
            self.scope = load_discovery_scope(self.discovery_path)
        self.daemon = FakeDaemonClient()
        try:
            self.host = StdioToolHost(self.daemon, self.scope)
        except Exception:
            self.temp.cleanup()
            raise

    def close(self) -> None:
        self.temp.cleanup()

    def fence(self) -> dict[str, object]:
        return {
            "context_id": "context-authority",
            "expected_model_revision": "revision-authority",
            "accepted_transaction_tip": "transaction-authority",
        }

    def call(self, name: str, arguments: dict[str, object]) -> dict[str, object]:
        response = self.host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {"name": name, "arguments": arguments},
            }
        )
        return response["result"]["content"][0]["json"]

    @staticmethod
    def _write(path: Path, value: dict[str, object]) -> None:
        path.write_text(json.dumps(value), encoding="utf-8")


class TestAgentCapability(unittest.TestCase):
    def setUp(self) -> None:
        self.fixtures: list[ScopedHostFixture] = []

    def tearDown(self) -> None:
        for fixture in self.fixtures:
            fixture.close()

    def fixture(
        self,
        capabilities: list[str],
        *,
        unattended_tools: list[str] | None = None,
    ) -> ScopedHostFixture:
        fixture = ScopedHostFixture(
            capabilities, unattended_tools=unattended_tools
        )
        self.fixtures.append(fixture)
        return fixture

    def test_default_propose_grant_advertises_no_apply_or_direct_mutation(self) -> None:
        fixture = self.fixture(["inspect", "propose"])
        tools = self._tools(fixture)
        self.assertEqual(
            tools["datum.proposal.show"]["x_datum_required_capability"],
            "inspect",
        )
        self.assertEqual(
            tools["datum.proposal.create"]["x_datum_required_capability"],
            "propose",
        )
        self.assertIn("datum.proposal.preview", tools)
        self.assertNotIn("datum.proposal.apply", tools)
        self.assertNotIn("datum.proposal.accept_apply", tools)
        self.assertNotIn("datum.route.apply", tools)
        self.assertNotIn("journal_undo", tools)

    def test_inspect_only_grant_hides_proposal_creation(self) -> None:
        fixture = self.fixture(["inspect"])
        tools = self._tools(fixture)
        self.assertIn("datum.proposal.show", tools)
        self.assertNotIn("datum.proposal.create", tools)
        self.assertNotIn("datum.proposal.preview", tools)

    def test_apply_approved_dispatches_only_owner_reviewed_apply(self) -> None:
        fixture = self.fixture(["inspect", "propose", "apply-approved"])
        tools = self._tools(fixture)
        self.assertIn("datum.proposal.apply", tools)
        self.assertNotIn("datum.proposal.accept_apply", tools)
        payload = fixture.call(
            "datum.proposal.apply",
            {
                "path": os.fspath(fixture.root),
                "proposal": "proposal-authority",
            }
            | fixture.fence(),
        )
        self.assertTrue(payload["ok"])
        self.assertEqual(len(fixture.daemon.calls), 1)

    def test_ungranted_apply_is_refused_before_revision_or_daemon_work(self) -> None:
        fixture = self.fixture(["inspect", "propose"])
        payload = fixture.call(
            "datum.proposal.apply",
            {"path": os.fspath(fixture.root), "proposal": "proposal-authority"},
        )
        self.assertFalse(payload["ok"])
        self.assertEqual(payload["error"]["code"], "capability_denied")
        self.assertEqual(
            payload["error"]["details"]["reason"], "capability_not_granted"
        )
        self.assertEqual(fixture.daemon.calls, [])

    def test_unattended_grant_is_exact_tool_scoped(self) -> None:
        fixture = self.fixture(
            ["inspect", "propose", "apply-approved", "unattended"],
            unattended_tools=["datum.proposal.accept_apply"],
        )
        tools = self._tools(fixture)
        self.assertIn("datum.proposal.accept_apply", tools)
        self.assertNotIn("datum.route.apply", tools)
        allowed = fixture.call(
            "datum.proposal.accept_apply",
            {
                "path": os.fspath(fixture.root),
                "proposal": "proposal-authority",
            }
            | fixture.fence(),
        )
        self.assertTrue(allowed["ok"])
        denied = fixture.call(
            "datum.route.apply",
            {"path": os.fspath(fixture.root)},
        )
        self.assertFalse(denied["ok"])
        self.assertEqual(
            denied["error"]["details"]["reason"],
            "tool_not_in_unattended_allowlist",
        )

    def test_hidden_alias_cannot_bypass_canonical_unattended_grant(self) -> None:
        fixture = self.fixture(
            ["inspect", "propose", "apply-approved", "unattended"],
            unattended_tools=["datum.proposal.accept_apply"],
        )
        payload = fixture.call(
            "accept_apply_proposal",
            {"path": os.fspath(fixture.root), "proposal": "proposal-authority"},
        )
        self.assertFalse(payload["ok"])
        self.assertEqual(
            payload["error"]["details"]["reason"],
            "compatibility_alias_not_authorized",
        )
        self.assertEqual(len(fixture.daemon.calls), 0)

    def test_project_grant_cannot_read_or_write_another_project(self) -> None:
        fixture = self.fixture(["inspect", "propose"])
        other = fixture.root / "other-project"
        other.mkdir()
        payload = fixture.call(
            "datum.proposal.preview",
            {"path": os.fspath(other), "proposal": "proposal-other"},
        )
        self.assertFalse(payload["ok"])
        self.assertEqual(
            payload["error"]["details"]["reason"], "project_scope_mismatch"
        )
        self.assertEqual(fixture.daemon.calls, [])

    def test_non_cumulative_grant_fails_host_startup(self) -> None:
        fixture = self.fixture(["inspect", "propose"])
        pinned = json.loads(fixture.pinned_path.read_text(encoding="utf-8"))
        pinned["capabilities"] = ["inspect", "apply-approved"]
        fixture._write(fixture.pinned_path, pinned)
        discovery = json.loads(fixture.discovery_path.read_text(encoding="utf-8"))
        discovery["capabilities"] = ["inspect", "apply-approved"]
        fixture._write(fixture.discovery_path, discovery)
        with patch.dict(os.environ, {}, clear=True):
            scope = load_discovery_scope(fixture.discovery_path)
        with self.assertRaisesRegex(ValueError, "exact cumulative profile"):
            StdioToolHost(FakeDaemonClient(), scope)

    def test_unattended_allowlist_rejects_unknown_or_non_mutating_tools(self) -> None:
        for tool, expected in [
            ("datum.unknown.action", "unknown canonical tool"),
            ("datum.proposal.show", "non-unattended tool"),
        ]:
            with self.subTest(tool=tool):
                with self.assertRaisesRegex(ValueError, expected):
                    self.fixture(
                        ["inspect", "propose", "apply-approved", "unattended"],
                        unattended_tools=[tool],
                    )

    def test_discovery_cannot_relabel_pinned_capability_authority(self) -> None:
        fixture = self.fixture(["inspect", "propose"])
        discovery = json.loads(fixture.discovery_path.read_text(encoding="utf-8"))
        discovery["capabilities"] = ["inspect", "propose", "apply-approved"]
        fixture._write(fixture.discovery_path, discovery)
        with patch.dict(os.environ, {}, clear=True):
            with self.assertRaisesRegex(ValueError, "pinned capabilities mismatch"):
                load_discovery_scope(fixture.discovery_path)

    @staticmethod
    def _tools(fixture: ScopedHostFixture) -> dict[str, dict[str, object]]:
        response = fixture.host.handle_message(
            {"jsonrpc": "2.0", "id": 2, "method": "tools/list"}
        )
        return {tool["name"]: tool for tool in response["result"]["tools"]}


if __name__ == "__main__":
    unittest.main()
