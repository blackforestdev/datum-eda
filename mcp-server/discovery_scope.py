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
    context_id: str | None


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
    context_id = document.get("context_id")
    if context_id is not None and (not isinstance(context_id, str) or not context_id):
        raise ValueError("discovery context_id must be a non-empty string when present")

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
        context_id=context_id,
    )


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
