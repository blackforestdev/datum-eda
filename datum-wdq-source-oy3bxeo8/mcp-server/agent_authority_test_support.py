#!/usr/bin/env python3
"""Datum-authored test fixture for session-scoped agent authority."""

from __future__ import annotations

import json
import os
from pathlib import Path
from typing import Any


def write_agent_authority_fixture(
    root: Path,
    terminal_session_id: str,
    *,
    agent_launch_id: str = "agent-launch-test",
) -> tuple[dict[str, Any], dict[str, Any], dict[str, str]]:
    credential_id = "credential-test"
    secret = "a" * 64
    credential_path = root / ".agent-credential.json"
    descriptor_path = root / "agent-authority.json"
    event_log_path = root / ".datum" / "tool-sessions" / "terminal-events.jsonl"
    event_log_path.parent.mkdir(parents=True, exist_ok=True)
    _write_private(
        credential_path,
        {
            "schema": "datum_agent_credential_v1",
            "credential_id": credential_id,
            "terminal_session_id": terminal_session_id,
            "project_root": os.fspath(root),
            "secret": secret,
        },
    )
    _write_private(
        descriptor_path,
        {
            "schema": "datum_agent_authority_v1",
            "credential_id": credential_id,
            "terminal_session_id": terminal_session_id,
            "project_root": os.fspath(root),
            "state": "active",
            "issued_unix_ms": 1,
            "revoked_unix_ms": None,
        },
    )
    context_fields = {
        "agent_launch_id": agent_launch_id,
        "session_lifecycle": "running",
        "storage": {"event_log_path": os.fspath(event_log_path)},
    }
    discovery_fields = {
        "agent_launch_id": agent_launch_id,
        "credential_descriptor": os.fspath(descriptor_path),
    }
    environment = {
        "DATUM_AGENT_CREDENTIAL_FILE": os.fspath(credential_path),
        "DATUM_AGENT_LAUNCH_ID": agent_launch_id,
        "DATUM_AGENT_ADAPTER_ID": "test-agent",
    }
    return context_fields, discovery_fields, environment


def _write_private(path: Path, document: dict[str, Any]) -> None:
    path.write_text(json.dumps(document), encoding="utf-8")
    path.chmod(0o600)
