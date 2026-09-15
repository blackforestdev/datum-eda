#!/usr/bin/env python3
"""Validate the protected Datum discovery scope before MCP startup."""

from __future__ import annotations

from dataclasses import dataclass
import json
import os
from pathlib import Path
from typing import Any


SUPPORTED_DISCOVERY_SCHEMAS = {
    "datum_agent_discovery_v1",
    "datum_terminal_context_v1",
}


@dataclass(frozen=True)
class DiscoveryScope:
    path: Path
    schema: str
    project_root: Path
    terminal_session_id: str
    agent_launch_id: str | None
    context_id: str | None
    document: dict[str, Any]
    live_context_id: str | None = None
    live_context_path: Path | None = None
    pinned_context_path: Path | None = None
    pinned_document: dict[str, Any] | None = None
    credential_descriptor_path: Path | None = None

    def read_live_context(self) -> dict[str, Any]:
        if self.live_context_path is None:
            return self.document
        document = _load_json_object(self.live_context_path, "live context")
        _match_context_identity(
            document,
            self.terminal_session_id,
            self.context_id,
            self.live_context_id,
            "live",
        )
        return document

    def read_pinned_context(self) -> dict[str, Any]:
        return self.pinned_document or self.document


def load_discovery_scope(path: str | os.PathLike[str]) -> DiscoveryScope:
    requested = Path(path)
    try:
        canonical = requested.resolve(strict=True)
    except OSError as exc:
        raise ValueError(f"discovery document is unavailable: {requested}") from exc
    if not canonical.is_file():
        raise ValueError(f"discovery document is not a regular file: {canonical}")
    try:
        document = json.loads(canonical.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        raise ValueError(f"discovery document is not valid UTF-8 JSON: {canonical}") from exc
    if not isinstance(document, dict):
        raise ValueError("discovery document root must be an object")

    schema = document.get("schema") or document.get("contract")
    if schema not in SUPPORTED_DISCOVERY_SCHEMAS:
        raise ValueError(
            f"unsupported discovery schema {schema!r}; run `datum-eda agent doctor`"
        )
    project_root = _required_path(document, "project_root")
    terminal_session_id = _required_string(document, "terminal_session_id")
    agent_launch_id = document.get("agent_launch_id")
    if schema == "datum_agent_discovery_v1":
        agent_launch_id = _required_string(document, "agent_launch_id")
    elif agent_launch_id is not None and (
        not isinstance(agent_launch_id, str) or not agent_launch_id
    ):
        raise ValueError("discovery agent_launch_id must be a non-empty string when present")
    context_id = document.get("pinned_context_id") or document.get("context_id")
    if context_id is not None and (not isinstance(context_id, str) or not context_id):
        raise ValueError("discovery context_id must be a non-empty string when present")

    live_context_id = document.get("live_context_id")
    if live_context_id is not None and (
        not isinstance(live_context_id, str) or not live_context_id
    ):
        raise ValueError("discovery live_context_id must be a non-empty string when present")
    live_context_path = _optional_context_path(document, "live_context_path")
    pinned_context_path = _optional_context_path(document, "pinned_context_path")
    credential_descriptor_path = _optional_context_path(
        document, "credential_descriptor"
    )
    if schema == "datum_agent_discovery_v1" and credential_descriptor_path is None:
        raise ValueError("agent discovery credential_descriptor is required")
    if (live_context_path is None) != (pinned_context_path is None):
        raise ValueError(
            "discovery live_context_path and pinned_context_path must be declared together"
        )
    pinned_document = None
    if live_context_path is not None and pinned_context_path is not None:
        live_document = _load_json_object(live_context_path, "live context")
        pinned_document = _load_json_object(pinned_context_path, "pinned context")
        _match_context_identity(
            live_document, terminal_session_id, context_id, live_context_id, "live"
        )
        _match_context_identity(
            pinned_document, terminal_session_id, context_id, live_context_id, "pinned"
        )
        _match_pinned_authority(document, pinned_document)

    _match_environment("DATUM_PROJECT_ROOT", project_root)
    _match_environment("DATUM_TERMINAL_SESSION_ID", terminal_session_id)
    os.environ.setdefault("DATUM_PROJECT_ROOT", os.fspath(project_root))
    os.environ.setdefault("DATUM_TERMINAL_SESSION_ID", terminal_session_id)
    os.environ["DATUM_AGENT_DISCOVERY"] = os.fspath(canonical)
    return DiscoveryScope(
        path=canonical,
        schema=schema,
        project_root=project_root,
        terminal_session_id=terminal_session_id,
        agent_launch_id=agent_launch_id,
        context_id=context_id,
        document=document,
        live_context_id=live_context_id,
        live_context_path=live_context_path,
        pinned_context_path=pinned_context_path,
        pinned_document=pinned_document,
        credential_descriptor_path=credential_descriptor_path,
    )


def _optional_context_path(document: dict[str, Any], key: str) -> Path | None:
    value = document.get(key)
    if value is None:
        return None
    if not isinstance(value, str) or not value:
        raise ValueError(f"discovery {key} must be a non-empty absolute path")
    path = Path(value)
    if not path.is_absolute():
        raise ValueError(f"discovery {key} must be absolute")
    try:
        canonical = path.resolve(strict=True)
    except OSError as exc:
        raise ValueError(f"discovery {key} is unavailable: {path}") from exc
    if not canonical.is_file():
        raise ValueError(f"discovery {key} is not a regular file: {canonical}")
    return canonical


def _load_json_object(path: Path, label: str) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        raise ValueError(f"discovery {label} is not valid UTF-8 JSON: {path}") from exc
    if not isinstance(value, dict):
        raise ValueError(f"discovery {label} root must be an object")
    return value


def _match_context_identity(
    document: dict[str, Any],
    terminal_session_id: str,
    pinned_context_id: str | None,
    live_context_id: str | None,
    expected_kind: str,
) -> None:
    if document.get("terminal_session_id") != terminal_session_id:
        raise ValueError(f"discovery {expected_kind} context session identity mismatch")
    if pinned_context_id is not None and document.get("pinned_context_id") != pinned_context_id:
        raise ValueError(f"discovery {expected_kind} pinned context identity mismatch")
    if live_context_id is not None and document.get("live_context_id") != live_context_id:
        raise ValueError(f"discovery {expected_kind} live context identity mismatch")
    if document.get("context_kind") != expected_kind:
        raise ValueError(f"discovery {expected_kind} context kind mismatch")


def _match_pinned_authority(
    discovery: dict[str, Any], pinned_document: dict[str, Any]
) -> None:
    for field in ("model_revision", "accepted_transaction_tip"):
        if discovery.get(field) != pinned_document.get(field):
            raise ValueError(f"discovery pinned {field} mismatch")
    for field in (
        "capabilities",
        "capability_profile",
        "approval_policy",
        "unattended_tools",
    ):
        if field in discovery and discovery[field] != pinned_document.get(field):
            raise ValueError(f"discovery pinned {field} mismatch")


def _required_path(document: dict[str, Any], key: str) -> Path:
    value = _required_string(document, key)
    path = Path(value)
    if not path.is_absolute():
        raise ValueError(f"discovery {key} must be absolute")
    try:
        canonical = path.resolve(strict=True)
    except OSError as exc:
        raise ValueError(f"discovery {key} is unavailable: {path}") from exc
    if not canonical.is_dir():
        raise ValueError(f"discovery {key} is not a directory: {canonical}")
    return canonical


def _required_string(document: dict[str, Any], key: str) -> str:
    value = document.get(key)
    if not isinstance(value, str) or not value:
        raise ValueError(f"discovery {key} must be a non-empty string")
    return value


def _match_environment(key: str, expected: Path | str) -> None:
    actual = os.environ.get(key)
    if actual is None:
        return
    if isinstance(expected, Path):
        try:
            matches = Path(actual).resolve(strict=True) == expected
        except OSError:
            matches = False
    else:
        matches = actual == expected
    if not matches:
        raise ValueError(f"discovery scope does not match inherited {key}")
