#!/usr/bin/env python3
"""Pinned-context and model-revision fences for scoped MCP writes."""

from __future__ import annotations

from copy import deepcopy
from typing import Any

from discovery_scope import DiscoveryScope


CONTEXT_ID_FIELD = "context_id"
EXPECTED_MODEL_REVISION_FIELD = "expected_model_revision"
ACCEPTED_TRANSACTION_TIP_FIELD = "accepted_transaction_tip"
_FENCE_FIELDS = {
    CONTEXT_ID_FIELD,
    EXPECTED_MODEL_REVISION_FIELD,
    ACCEPTED_TRANSACTION_TIP_FIELD,
}


class ContextFenceError(RuntimeError):
    """Structured refusal raised before a scoped proposal/apply dispatch."""

    def __init__(self, code: str, message: str, details: dict[str, Any]) -> None:
        super().__init__(message)
        self.code = code
        self.details = details


def scoped_tool_catalog(
    tools: list[dict[str, Any]],
    catalog: dict[str, dict[str, Any]],
    discovery: DiscoveryScope | None,
) -> list[dict[str, Any]]:
    """Advertise required fence fields only for discovery-scoped write tools."""

    if discovery is None:
        return tools
    scoped = deepcopy(tools)
    for tool in scoped:
        name = tool.get("name")
        spec = catalog.get(name) if isinstance(name, str) else None
        if spec is None or not tool_requires_context_fence(spec, catalog):
            continue
        schema = tool.setdefault("inputSchema", {"type": "object"})
        properties = schema.setdefault("properties", {})
        properties[CONTEXT_ID_FIELD] = {
            "type": "string",
            "description": "Immutable pinned Datum work-context identity.",
        }
        properties[EXPECTED_MODEL_REVISION_FIELD] = {
            "type": "string",
            "description": "Model revision against which this request was prepared.",
        }
        properties[ACCEPTED_TRANSACTION_TIP_FIELD] = {
            "type": ["string", "null"],
            "description": "Accepted journal transaction tip from the pinned context.",
        }
        required = schema.setdefault("required", [])
        for field in (
            CONTEXT_ID_FIELD,
            EXPECTED_MODEL_REVISION_FIELD,
            ACCEPTED_TRANSACTION_TIP_FIELD,
        ):
            if field not in required:
                required.append(field)
    return scoped


def validate_context_fence(
    discovery: DiscoveryScope | None,
    name: str,
    arguments: Any,
    catalog: dict[str, dict[str, Any]],
) -> dict[str, Any]:
    """Validate and remove MCP-only fence fields before daemon dispatch."""

    if not isinstance(arguments, dict):
        raise ValueError("tool arguments must be an object")
    spec = catalog.get(name)
    if discovery is None or spec is None or not tool_requires_context_fence(spec, catalog):
        return arguments

    missing = [field for field in _FENCE_FIELDS if field not in arguments]
    if missing:
        raise ContextFenceError(
            "context_fence_required",
            "proposal/apply request is missing its pinned context fence",
            _error_details(discovery, name, arguments, reason="missing_fields")
            | {"missing_fields": sorted(missing)},
        )

    provided_context = _non_empty_string(arguments.get(CONTEXT_ID_FIELD))
    provided_revision = _non_empty_string(arguments.get(EXPECTED_MODEL_REVISION_FIELD))
    provided_tip = arguments.get(ACCEPTED_TRANSACTION_TIP_FIELD)
    if provided_tip is not None and not isinstance(provided_tip, str):
        provided_tip = _INVALID

    try:
        pinned = discovery.read_pinned_context()
        live = discovery.read_live_context()
    except ValueError as exc:
        raise ContextFenceError(
            "context_fence_unavailable",
            "Datum cannot read the pinned/current context fence",
            _error_details(
                discovery,
                name,
                arguments,
                reason="authority_document_unavailable",
                pinned=discovery.pinned_document or discovery.document,
                live={},
            )
            | {"authority_error": str(exc)},
        ) from exc
    pinned_context = _context_id(discovery, pinned)
    pinned_revision = _non_empty_string(pinned.get("model_revision"))
    current_revision = _non_empty_string(live.get("model_revision"))
    pinned_tip = _transaction_tip(pinned)
    current_tip = _transaction_tip(live)
    if pinned_context is None or pinned_revision is None or current_revision is None:
        raise ContextFenceError(
            "context_fence_unavailable",
            "Datum cannot establish the pinned/current revision fence",
            _error_details(
                discovery,
                name,
                arguments,
                reason="missing_authority_state",
                pinned=pinned,
                live=live,
            ),
        )

    reason = None
    if provided_context != pinned_context:
        reason = "context_id_mismatch"
    elif provided_revision != pinned_revision:
        reason = "pinned_revision_mismatch"
    elif current_revision != pinned_revision:
        reason = "live_revision_advanced"
    elif provided_tip is _INVALID:
        reason = "transaction_tip_invalid"
    elif provided_tip != pinned_tip:
        reason = "pinned_transaction_tip_mismatch"
    elif current_tip != pinned_tip:
        reason = "live_transaction_tip_advanced"
    if reason is not None:
        raise ContextFenceError(
            "stale_context",
            "proposal/apply request no longer matches its pinned Datum context",
            _error_details(
                discovery, name, arguments, reason=reason, pinned=pinned, live=live
            ),
        )

    return {key: value for key, value in arguments.items() if key not in _FENCE_FIELDS}


def tool_requires_context_fence(
    spec: dict[str, Any], catalog: dict[str, dict[str, Any]]
) -> bool:
    if spec.get("x_public_write_surface_class"):
        return True
    for replacement in spec.get("x_canonical_replacements", []):
        canonical = catalog.get(replacement)
        if canonical is not None and canonical.get("x_public_write_surface_class"):
            return True
    return False


def _error_details(
    discovery: DiscoveryScope,
    name: str,
    arguments: dict[str, Any],
    *,
    reason: str,
    pinned: dict[str, Any] | None = None,
    live: dict[str, Any] | None = None,
) -> dict[str, Any]:
    pinned = discovery.read_pinned_context() if pinned is None else pinned
    if live is None:
        try:
            live = discovery.read_live_context()
        except ValueError:
            live = {}
    context_id = _context_id(discovery, pinned)
    pinned_uri = (
        f"datum://context/pinned/{context_id}"
        if context_id
        else "datum://context/pinned/{context_id}"
    )
    return {
        "reason": reason,
        "tool": name,
        "provided": {
            "context_id": arguments.get(CONTEXT_ID_FIELD),
            "expected_model_revision": arguments.get(EXPECTED_MODEL_REVISION_FIELD),
            "accepted_transaction_tip": arguments.get(ACCEPTED_TRANSACTION_TIP_FIELD),
        },
        "pinned": {
            "context_id": context_id,
            "model_revision": pinned.get("model_revision"),
            "accepted_transaction_tip": _transaction_tip(pinned),
        },
        "current": {
            "live_context_id": discovery.live_context_id,
            "model_revision": live.get("model_revision"),
            "accepted_transaction_tip": _transaction_tip(live),
        },
        "affected_ids": _affected_ids(pinned),
        "options": {
            "refresh": {
                "resource": "datum://context/live",
                "instruction": "Read the live context and inspect the current revision before retrying.",
            },
            "rebase": {
                "resource": pinned_uri,
                "instruction": "Start a new agent work unit to pin the refreshed live context before retrying.",
            },
        },
    }


def _context_id(discovery: DiscoveryScope, document: dict[str, Any]) -> str | None:
    return (
        discovery.context_id
        or _non_empty_string(document.get("pinned_context_id"))
        or _non_empty_string(document.get("context_id"))
    )


def _transaction_tip(document: dict[str, Any]) -> str | None:
    value = document.get("accepted_transaction_tip")
    return value if isinstance(value, str) and value else None


def _affected_ids(document: dict[str, Any]) -> list[str]:
    values: set[str] = set()
    for key in ("board_id", "scene_id", "focused_artifact_id", "latest_proposal_id"):
        value = document.get(key)
        if isinstance(value, str) and value:
            values.add(value)
    selection = document.get("selection_context")
    if isinstance(selection, dict):
        selected = selection.get("id")
        if isinstance(selected, str) and selected:
            values.add(selected)
        for key in ("object_ids", "selected_ids"):
            items = selection.get(key)
            if isinstance(items, list):
                values.update(item for item in items if isinstance(item, str) and item)
    return sorted(values)


def _non_empty_string(value: Any) -> str | None:
    return value if isinstance(value, str) and value else None


_INVALID = object()
