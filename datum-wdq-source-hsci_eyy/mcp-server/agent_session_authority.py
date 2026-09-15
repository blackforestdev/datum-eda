#!/usr/bin/env python3
"""Session-bound Datum agent credential validation and revocation."""

from __future__ import annotations

from dataclasses import dataclass
import json
import os
from pathlib import Path
import stat
from typing import Any

from discovery_scope import DiscoveryScope


class SessionAuthorityError(RuntimeError):
    """A scoped broker no longer owns live terminal-session authority."""

    def __init__(self, reason: str, discovery: DiscoveryScope) -> None:
        super().__init__("Datum agent session authority is unavailable or revoked")
        self.code = "session_authority_revoked"
        self.details = {
            "reason": reason,
            "terminal_session_id": discovery.terminal_session_id,
            "context_id": discovery.context_id,
            "agent_launch_id": discovery.agent_launch_id,
        }


@dataclass(frozen=True)
class SessionAuthority:
    discovery: DiscoveryScope
    credential_file: Path
    credential_id: str
    agent_identity: str

    def assert_active(self) -> None:
        descriptor = _read_json(self.discovery.credential_descriptor_path, "authority descriptor")
        if descriptor.get("schema") != "datum_agent_authority_v1":
            raise SessionAuthorityError("unsupported_authority_schema", self.discovery)
        if descriptor.get("credential_id") != self.credential_id:
            raise SessionAuthorityError("credential_rotated", self.discovery)
        if descriptor.get("terminal_session_id") != self.discovery.terminal_session_id:
            raise SessionAuthorityError("session_identity_mismatch", self.discovery)
        if descriptor.get("project_root") != str(self.discovery.project_root):
            raise SessionAuthorityError("project_identity_mismatch", self.discovery)
        if descriptor.get("state") != "active":
            raise SessionAuthorityError("authority_revoked", self.discovery)
        try:
            _validate_private_file(self.credential_file, "credential")
            credential = _read_json(self.credential_file, "credential")
        except ValueError as exc:
            raise SessionAuthorityError("credential_unavailable", self.discovery) from exc
        if credential.get("schema") != "datum_agent_credential_v1":
            raise SessionAuthorityError("unsupported_credential_schema", self.discovery)
        if credential.get("credential_id") != self.credential_id:
            raise SessionAuthorityError("credential_identity_mismatch", self.discovery)
        if credential.get("terminal_session_id") != self.discovery.terminal_session_id:
            raise SessionAuthorityError("credential_session_mismatch", self.discovery)
        if credential.get("project_root") != str(self.discovery.project_root):
            raise SessionAuthorityError("credential_project_mismatch", self.discovery)
        secret = credential.get("secret")
        if not isinstance(secret, str) or len(secret) < 64:
            raise SessionAuthorityError("credential_secret_invalid", self.discovery)
        try:
            live = self.discovery.read_live_context()
        except ValueError as exc:
            raise SessionAuthorityError("live_context_unavailable", self.discovery) from exc
        if live.get("session_lifecycle") != "running":
            raise SessionAuthorityError("terminal_session_not_running", self.discovery)


def load_session_authority(discovery: DiscoveryScope | None) -> SessionAuthority | None:
    if discovery is None or discovery.schema != "datum_agent_discovery_v1":
        return None
    descriptor_path = discovery.credential_descriptor_path
    if descriptor_path is None:
        raise ValueError("agent discovery lacks credential_descriptor")
    _validate_private_file(descriptor_path, "authority descriptor")
    descriptor = _read_json(descriptor_path, "authority descriptor")
    credential_id = _required_string(descriptor, "credential_id", "authority descriptor")
    raw_credential_path = os.environ.get("DATUM_AGENT_CREDENTIAL_FILE")
    if not raw_credential_path:
        raise ValueError("DATUM_AGENT_CREDENTIAL_FILE is required for scoped agent discovery")
    credential_path = Path(raw_credential_path)
    if not credential_path.is_absolute():
        raise ValueError("DATUM_AGENT_CREDENTIAL_FILE must be absolute")
    credential_path = credential_path.resolve(strict=True)
    if credential_path.parent != descriptor_path.parent:
        raise ValueError("agent credential and authority descriptor must share one runtime scope")
    authority = SessionAuthority(
        discovery=discovery,
        credential_file=credential_path,
        credential_id=credential_id,
        agent_identity=os.environ.get("DATUM_AGENT_ADAPTER_ID", "direct-agent"),
    )
    authority.assert_active()
    return authority


def _validate_private_file(path: Path | None, label: str) -> None:
    if path is None:
        raise ValueError(f"agent {label} is unavailable")
    metadata = path.stat()
    if not stat.S_ISREG(metadata.st_mode):
        raise ValueError(f"agent {label} is not a regular file")
    if stat.S_IMODE(metadata.st_mode) & 0o077:
        raise ValueError(f"agent {label} must not be accessible to group or other users")


def _read_json(path: Path | None, label: str) -> dict[str, Any]:
    if path is None:
        raise ValueError(f"agent {label} is unavailable")
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        raise ValueError(f"agent {label} is not valid UTF-8 JSON") from exc
    if not isinstance(value, dict):
        raise ValueError(f"agent {label} root must be an object")
    return value


def _required_string(document: dict[str, Any], key: str, label: str) -> str:
    value = document.get(key)
    if not isinstance(value, str) or not value:
        raise ValueError(f"agent {label} {key} must be a non-empty string")
    return value
