#!/usr/bin/env python3
"""Capability, resource, prompt, and notification contract tests."""

from __future__ import annotations

import io
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from discovery_scope import DiscoveryScope, load_discovery_scope
from resources_catalog import RESOURCE_TEMPLATES
from stdio_tool_host import StdioToolHost
from test_support import FakeDaemonClient


def _scope() -> DiscoveryScope:
    return DiscoveryScope(
        path=Path("/tmp/datum-context.json"),
        schema="datum_terminal_context_v1",
        project_root=Path("/tmp/project"),
        terminal_session_id="session-test",
        context_id="context-test",
        document={
            "contract": "datum_terminal_context_v1",
            "project_root": "/tmp/project",
            "project_id": "project-test",
            "project_name": "Test design",
            "model_revision": "revision-test",
            "context_id": "context-test",
            "terminal_session_id": "session-test",
            "selection_context": {"object_ids": ["object-test"]},
            "latest_check_run_id": "check-run-test",
            "visible_check_run_ids": ["check-run-test"],
            "visible_finding_fingerprints": ["sha256:test"],
            "visible_proposal_ids": ["proposal-test"],
            "visible_artifact_ids": ["artifact-test"],
            "check_status": {"status": "failed"},
            "capability_profile": "datum_agent_capability_v1",
            "capabilities": ["inspect", "propose"],
            "approval_policy": "owner-review-required",
            "unattended_tools": [],
        },
    )


class TestMcpResourcesPrompts(unittest.TestCase):
    def test_live_context_refreshes_while_pinned_context_stays_immutable(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve()
            live_path = root / "live.json"
            pinned_path = root / "pinned.json"
            discovery_path = root / "discovery.json"
            common = {
                "contract": "datum_terminal_context_v1",
                "project_root": os.fspath(root),
                "terminal_session_id": "terminal-split",
                "context_id": "context-pinned",
                "live_context_id": "live-terminal-split",
                "pinned_context_id": "context-pinned",
                "model_revision": "revision-pinned",
                "capability_profile": "datum_agent_capability_v1",
                "capabilities": ["inspect", "propose"],
                "approval_policy": "owner-review-required",
                "unattended_tools": [],
            }
            live_path.write_text(
                json.dumps(
                    common
                    | {
                        "context_kind": "live",
                        "selection_context": {"id": "live-before"},
                    }
                ),
                encoding="utf-8",
            )
            pinned_path.write_text(
                json.dumps(
                    common
                    | {
                        "context_kind": "pinned",
                        "selection_context": {"id": "pinned-selection"},
                    }
                ),
                encoding="utf-8",
            )
            discovery_path.write_text(
                json.dumps(
                    {
                        "schema": "datum_agent_discovery_v1",
                        "project_root": os.fspath(root),
                        "terminal_session_id": "terminal-split",
                        "live_context_id": "live-terminal-split",
                        "live_context_path": os.fspath(live_path),
                        "pinned_context_id": "context-pinned",
                        "pinned_context_path": os.fspath(pinned_path),
                        "model_revision": "revision-pinned",
                        "capability_profile": "datum_agent_capability_v1",
                        "capabilities": ["inspect", "propose"],
                        "approval_policy": "owner-review-required",
                        "unattended_tools": [],
                    }
                ),
                encoding="utf-8",
            )
            with patch.dict(os.environ, {}, clear=True):
                host = StdioToolHost(
                    FakeDaemonClient(), load_discovery_scope(discovery_path)
                )
            live_path.write_text(
                json.dumps(
                    common
                    | {
                        "context_kind": "live",
                        "selection_context": {"id": "live-after"},
                    }
                ),
                encoding="utf-8",
            )
            pinned_path.write_text(
                json.dumps(
                    common
                    | {
                        "context_kind": "pinned",
                        "selection_context": {"id": "tampered-after-pin"},
                    }
                ),
                encoding="utf-8",
            )

            live = host.handle_message(
                {
                    "jsonrpc": "2.0",
                    "id": 20,
                    "method": "resources/read",
                    "params": {"uri": "datum://context/live"},
                }
            )
            pinned = host.handle_message(
                {
                    "jsonrpc": "2.0",
                    "id": 21,
                    "method": "resources/read",
                    "params": {"uri": "datum://context/pinned/context-pinned"},
                }
            )
            self.assertEqual(
                json.loads(live["result"]["contents"][0]["text"])[
                    "selection_context"
                ]["id"],
                "live-after",
            )
            self.assertEqual(
                json.loads(pinned["result"]["contents"][0]["text"])[
                    "selection_context"
                ]["id"],
                "pinned-selection",
            )

    def test_initialize_declares_typed_protocol_capabilities(self) -> None:
        response = StdioToolHost(FakeDaemonClient(), _scope()).handle_message(
            {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {"protocolVersion": "2025-06-18"},
            }
        )
        capabilities = response["result"]["capabilities"]
        self.assertEqual(capabilities["tools"], {"listChanged": False})
        self.assertEqual(capabilities["resources"], {"subscribe": True, "listChanged": True})
        self.assertEqual(capabilities["prompts"], {"listChanged": False})

    def test_resources_list_stable_current_and_visible_identities(self) -> None:
        host = StdioToolHost(FakeDaemonClient(), _scope())
        response = host.handle_message({"jsonrpc": "2.0", "id": 2, "method": "resources/list"})
        uris = {resource["uri"] for resource in response["result"]["resources"]}
        self.assertTrue(
            {
                "datum://project/current",
                "datum://context/live",
                "datum://context/pinned/context-test",
                "datum://model/revision/revision-test",
                "datum://selection/context-test",
                "datum://checks/current",
                "datum://check/sha256:test",
                "datum://proposal/proposal-test",
                "datum://artifact/artifact-test",
            }.issubset(uris)
        )

    def test_templates_cover_render_and_paginated_stable_objects(self) -> None:
        host = StdioToolHost(FakeDaemonClient(), _scope())
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 3, "method": "resources/templates/list"}
        )
        self.assertEqual(response["result"]["resourceTemplates"], RESOURCE_TEMPLATES)
        templates = {item["uriTemplate"] for item in RESOURCE_TEMPLATES}
        self.assertIn("datum://render/board/{revision}.svg", templates)
        self.assertIn("datum://render/schematic/{revision}.svg", templates)
        self.assertIn("datum://objects/{kind}{?cursor,limit}", templates)

    def test_read_is_scoped_and_unknown_identity_is_rejected(self) -> None:
        host = StdioToolHost(FakeDaemonClient(), _scope())
        project = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 4,
                "method": "resources/read",
                "params": {"uri": "datum://project/current"},
            }
        )
        payload = json.loads(project["result"]["contents"][0]["text"])
        self.assertEqual(payload["project_id"], "project-test")
        rejected = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 5,
                "method": "resources/read",
                "params": {"uri": "datum://artifact/not-visible"},
            }
        )
        self.assertEqual(rejected["error"]["code"], -32002)

    def test_prompts_are_user_invoked_and_preserve_review_gates(self) -> None:
        host = StdioToolHost(FakeDaemonClient(), _scope())
        listed = host.handle_message({"jsonrpc": "2.0", "id": 6, "method": "prompts/list"})
        names = {prompt["name"] for prompt in listed["result"]["prompts"]}
        self.assertIn("datum.prepare-proposal", names)
        result = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 7,
                "method": "prompts/get",
                "params": {
                    "name": "datum.prepare-proposal",
                    "arguments": {"objective": "route the selected net"},
                },
            }
        )
        text = result["result"]["messages"][0]["content"]["text"]
        self.assertIn("datum://context/pinned/context-test", text)
        self.assertIn("do not apply it", text)

    def test_resource_notifications_require_subscription_and_stdio_support(self) -> None:
        host = StdioToolHost(FakeDaemonClient(), _scope())
        host._publish_resource_changes()
        self.assertEqual(host.drain_notifications(), [])
        subscribed = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 8,
                "method": "resources/subscribe",
                "params": {"uri": "datum://context/live"},
            }
        )
        self.assertEqual(subscribed["result"], {})
        host._publish_resource_changes()
        self.assertEqual(
            host.drain_notifications(),
            [
                {
                    "jsonrpc": "2.0",
                    "method": "notifications/resources/updated",
                    "params": {"uri": "datum://context/live"},
                }
            ],
        )
        http_host = StdioToolHost(
            FakeDaemonClient(), _scope(), notifications_enabled=False
        )
        initialized = http_host.handle_message(
            {"jsonrpc": "2.0", "id": 9, "method": "initialize", "params": {}}
        )
        self.assertFalse(initialized["result"]["capabilities"]["resources"]["subscribe"])

    def test_successful_declared_write_notifies_only_subscribed_resources(self) -> None:
        host = StdioToolHost(FakeDaemonClient(), _scope())
        host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 10,
                "method": "resources/subscribe",
                "params": {"uri": "datum://context/live"},
            }
        )
        with patch.object(host, "_call_tool", return_value={"content": []}):
            response = host.handle_message(
                {
                    "jsonrpc": "2.0",
                    "id": 11,
                    "method": "tools/call",
                    "params": {"name": "datum.proposal.create", "arguments": {}},
                }
            )
        self.assertEqual(response["result"], {"content": []})
        self.assertEqual(
            host.drain_notifications()[0]["method"],
            "notifications/resources/updated",
        )

    def test_stdio_flushes_negotiated_notifications_after_response(self) -> None:
        host = StdioToolHost(FakeDaemonClient(), _scope())
        host._subscriptions.add("datum://context/live")
        host._publish_resource_changes()
        stdout = io.StringIO()
        host.run_stdio(io.StringIO('{"jsonrpc":"2.0","id":12,"method":"ping"}\n'), stdout)
        messages = [json.loads(line) for line in stdout.getvalue().splitlines()]
        self.assertEqual(messages[0]["id"], 12)
        self.assertEqual(messages[1]["method"], "notifications/resources/updated")


if __name__ == "__main__":
    unittest.main()
