#!/usr/bin/env python3
"""Durable, secret-free agent invocation and journal provenance."""

from __future__ import annotations

from contextlib import contextmanager
from contextvars import ContextVar
from dataclasses import asdict, dataclass
import json
import os
from pathlib import Path
import time
from typing import Any, Iterator

from discovery_scope import DiscoveryScope


@dataclass(frozen=True)
class AgentInvocationProvenance:
    agent_identity: str
    agent_launch_id: str
    terminal_session_id: str
    context_id: str
    expected_model_revision: str
    accepted_transaction_tip: str | None
    requested_capability: str
    approval_policy: str
    approval_reference: str | None
    tool_name: str

    def environment_json(self) -> str:
        return json.dumps(asdict(self), separators=(",", ":"), sort_keys=True)


_CURRENT_INVOCATION: ContextVar[AgentInvocationProvenance | None] = ContextVar(
    "datum_agent_invocation", default=None
)


@contextmanager
def agent_invocation(
    provenance: AgentInvocationProvenance,
) -> Iterator[None]:
    token = _CURRENT_INVOCATION.set(provenance)
    try:
        yield
    finally:
        _CURRENT_INVOCATION.reset(token)


def current_agent_subprocess_environment() -> dict[str, str] | None:
    provenance = _CURRENT_INVOCATION.get()
    if provenance is None:
        return None
    environment = dict(os.environ)
    environment["DATUM_COMMIT_SOURCE"] = "assistant"
    environment["DATUM_TOOL_SURFACE"] = "mcp"
    environment["DATUM_AGENT_PROVENANCE"] = provenance.environment_json()
    return environment


class AgentAuditWriter:
    def __init__(self, discovery: DiscoveryScope | None) -> None:
        self._path = _audit_path(discovery)

    def record(
        self,
        provenance: AgentInvocationProvenance,
        outcome: str,
        result: Any = None,
        error: Exception | None = None,
    ) -> None:
        if self._path is None:
            return
        commit = _find_mapping(result, "transaction_id")
        batch = _find_mapping(result, "batch_id") or commit
        diff = _find_value(result, "diff")
        record: dict[str, Any] = {
            "event": "agent_tool_audit",
            "schema_version": 1,
            "occurred_unix_ms": time.time_ns() // 1_000_000,
            "agent_provenance": asdict(provenance),
            "outcome": outcome,
            "operation_batch": {
                "batch_id": None if batch is None else batch.get("batch_id"),
                "operation_count": _operation_count(result),
            },
            "diff": diff if isinstance(diff, dict) else None,
            "journal_result": {
                "transaction_id": None if commit is None else commit.get("transaction_id"),
                "before_model_revision": None
                if commit is None
                else commit.get("before_model_revision"),
                "after_model_revision": None
                if commit is None
                else commit.get("after_model_revision"),
                "status": "committed" if commit is not None else "not_applicable",
            },
        }
        if error is not None:
            record["error"] = {
                "code": getattr(error, "code", "tool_call_failed"),
                "reason": getattr(error, "details", {}).get("reason")
                if isinstance(getattr(error, "details", None), dict)
                else None,
            }
        payload = json.dumps(record, separators=(",", ":"), sort_keys=True)
        if len(payload.encode("utf-8")) > 16_384:
            raise RuntimeError("agent audit record exceeded its bounded record size")
        descriptor = os.open(
            self._path,
            os.O_APPEND | os.O_CREAT | os.O_WRONLY | os.O_CLOEXEC,
            0o600,
        )
        try:
            os.write(descriptor, payload.encode("utf-8") + b"\n")
        finally:
            os.close(descriptor)


def build_invocation_provenance(
    discovery: DiscoveryScope,
    agent_identity: str,
    approval_policy: str,
    required_capability: str,
    tool_name: str,
    arguments: dict[str, Any],
) -> AgentInvocationProvenance:
    pinned = discovery.read_pinned_context()
    context_id = arguments.get("context_id") or discovery.context_id
    revision = arguments.get("expected_model_revision") or pinned.get("model_revision")
    if not isinstance(context_id, str) or not context_id:
        raise ValueError("agent audit requires pinned context identity")
    if not isinstance(revision, str) or not revision:
        raise ValueError("agent audit requires expected model revision")
    launch_id = os.environ.get("DATUM_AGENT_LAUNCH_ID") or discovery.agent_launch_id
    if not isinstance(launch_id, str) or not launch_id:
        raise ValueError("agent audit requires agent launch identity")
    tip = arguments.get("accepted_transaction_tip", pinned.get("accepted_transaction_tip"))
    if tip is not None and not isinstance(tip, str):
        raise ValueError("agent audit transaction tip must be a string or null")
    approval_reference = arguments.get("proposal")
    if approval_reference is not None and not isinstance(approval_reference, str):
        approval_reference = None
    return AgentInvocationProvenance(
        agent_identity=agent_identity,
        agent_launch_id=launch_id,
        terminal_session_id=discovery.terminal_session_id,
        context_id=context_id,
        expected_model_revision=revision,
        accepted_transaction_tip=tip,
        requested_capability=required_capability,
        approval_policy=approval_policy,
        approval_reference=approval_reference,
        tool_name=tool_name,
    )


def _audit_path(discovery: DiscoveryScope | None) -> Path | None:
    if discovery is None or discovery.schema != "datum_agent_discovery_v1":
        return None
    pinned = discovery.read_pinned_context()
    storage = pinned.get("storage")
    value = storage.get("event_log_path") if isinstance(storage, dict) else None
    if not isinstance(value, str) or not value:
        raise ValueError("pinned context lacks agent lifecycle event path")
    path = Path(value)
    if not path.is_absolute():
        raise ValueError("agent lifecycle event path must be absolute")
    project_datum = (discovery.project_root / ".datum").resolve()
    resolved_parent = path.parent.resolve()
    if resolved_parent != project_datum and project_datum not in resolved_parent.parents:
        raise ValueError("agent lifecycle event path escapes the scoped project")
    return path


def _find_mapping(value: Any, key: str, depth: int = 0) -> dict[str, Any] | None:
    if depth > 6:
        return None
    if isinstance(value, dict):
        if key in value:
            return value
        for child in value.values():
            found = _find_mapping(child, key, depth + 1)
            if found is not None:
                return found
    elif isinstance(value, list):
        for child in value:
            found = _find_mapping(child, key, depth + 1)
            if found is not None:
                return found
    return None


def _find_value(value: Any, key: str) -> Any:
    mapping = _find_mapping(value, key)
    return None if mapping is None else mapping.get(key)


def _operation_count(value: Any) -> int | None:
    operations = _find_value(value, "operations")
    return len(operations) if isinstance(operations, list) else None
