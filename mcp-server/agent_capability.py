#!/usr/bin/env python3
"""Project/session-scoped Datum capability grants for the MCP broker."""

from __future__ import annotations

from copy import deepcopy
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from discovery_scope import DiscoveryScope


CAPABILITY_PROFILE = "datum_agent_capability_v1"
INSPECT = "inspect"
PROPOSE = "propose"
APPLY_APPROVED = "apply-approved"
UNATTENDED = "unattended"
_CAPABILITY_ORDER = (INSPECT, PROPOSE, APPLY_APPROVED, UNATTENDED)
_CAPABILITY_LEVEL = {name: index for index, name in enumerate(_CAPABILITY_ORDER)}
_WRITE_CLASS_CAPABILITY = {
    "proposal_metadata_write": PROPOSE,
    "proposal_review_state_write": UNATTENDED,
    "journaled_route_apply": UNATTENDED,
    "proposal_artifact_apply": UNATTENDED,
}
_PROPOSE_READ_TOOLS = {
    "datum.proposal.preview",
    "datum.proposal.validate",
}
_UNATTENDED_TOOLS = {
    "datum.journal.redo",
    "datum.journal.undo",
    "datum.route.capture_strategy_baseline",
    "datum.route.write_strategy_fixture_suite",
}


@dataclass(frozen=True)
class AgentCapabilityGrant:
    capabilities: tuple[str, ...]
    approval_policy: str
    unattended_tools: frozenset[str]

    def allows(self, capability: str) -> bool:
        return capability in self.capabilities


class AgentCapabilityError(RuntimeError):
    """Structured refusal raised before a scoped tool can cross its grant."""

    def __init__(self, code: str, message: str, details: dict[str, Any]) -> None:
        super().__init__(message)
        self.code = code
        self.details = details


def load_agent_capability_grant(
    discovery: DiscoveryScope | None,
    catalog: dict[str, dict[str, Any]],
) -> AgentCapabilityGrant | None:
    """Load one immutable cumulative grant from the pinned work context."""

    if discovery is None:
        return None
    pinned = discovery.read_pinned_context()
    if pinned.get("capability_profile") != CAPABILITY_PROFILE:
        raise ValueError(
            "pinned context lacks the supported Datum agent capability profile; refresh the session"
        )
    raw_capabilities = pinned.get("capabilities")
    if not isinstance(raw_capabilities, list) or not all(
        isinstance(value, str) for value in raw_capabilities
    ):
        raise ValueError("pinned context capabilities must be a string array")
    capabilities = tuple(raw_capabilities)
    if capabilities not in tuple(
        _CAPABILITY_ORDER[: index + 1] for index in range(len(_CAPABILITY_ORDER))
    ):
        raise ValueError("pinned context capabilities must be one exact cumulative profile")
    approval_policy = pinned.get("approval_policy")
    expected_policy = (
        "owner-enabled-unattended"
        if UNATTENDED in capabilities
        else "owner-review-required"
    )
    if approval_policy != expected_policy:
        raise ValueError("pinned context approval policy does not match its capability profile")
    raw_unattended_tools = pinned.get("unattended_tools", [])
    if not isinstance(raw_unattended_tools, list) or not all(
        isinstance(value, str) and value.startswith("datum.")
        for value in raw_unattended_tools
    ):
        raise ValueError("pinned unattended tools must be canonical datum.* names")
    if len(set(raw_unattended_tools)) != len(raw_unattended_tools):
        raise ValueError("pinned unattended tools must be unique")
    if UNATTENDED in capabilities and not raw_unattended_tools:
        raise ValueError("unattended authority requires at least one exact tool grant")
    if UNATTENDED not in capabilities and raw_unattended_tools:
        raise ValueError("unattended tool grants require unattended authority")
    for name in raw_unattended_tools:
        spec = catalog.get(name)
        if spec is None:
            raise ValueError(f"unattended tool grant names unknown canonical tool {name!r}")
        if required_capability(name, spec, catalog) != UNATTENDED:
            raise ValueError(
                f"unattended tool grant names non-unattended tool {name!r}"
            )
    return AgentCapabilityGrant(
        capabilities=capabilities,
        approval_policy=approval_policy,
        unattended_tools=frozenset(raw_unattended_tools),
    )


def scoped_authorized_tool_catalog(
    tools: list[dict[str, Any]],
    catalog: dict[str, dict[str, Any]],
    grant: AgentCapabilityGrant | None,
) -> list[dict[str, Any]]:
    """Project only the tools the immutable scoped grant can actually call."""

    if grant is None:
        return tools
    visible: list[dict[str, Any]] = []
    for tool in tools:
        name = tool.get("name")
        spec = catalog.get(name) if isinstance(name, str) else None
        if spec is None:
            continue
        required = required_capability(name, spec, catalog)
        if not grant.allows(required):
            continue
        if required == UNATTENDED and name not in grant.unattended_tools:
            continue
        projected = deepcopy(tool)
        projected["x_datum_required_capability"] = required
        visible.append(projected)
    return visible


def authorize_tool_call(
    grant: AgentCapabilityGrant | None,
    discovery: DiscoveryScope | None,
    name: Any,
    arguments: Any,
    catalog: dict[str, dict[str, Any]],
) -> None:
    """Enforce capability and project scope before any daemon dispatch."""

    if grant is None or discovery is None:
        return
    if not isinstance(name, str) or not isinstance(arguments, dict):
        return
    spec = catalog.get(name)
    if spec is None:
        return
    required = required_capability(name, spec, catalog)
    if not grant.allows(required):
        raise _denied(
            discovery,
            grant,
            name,
            required,
            "capability_not_granted",
        )
    if required == UNATTENDED:
        if not name.startswith("datum."):
            raise _denied(
                discovery,
                grant,
                name,
                required,
                "compatibility_alias_not_authorized",
            )
        if name not in grant.unattended_tools:
            raise _denied(
                discovery,
                grant,
                name,
                required,
                "tool_not_in_unattended_allowlist",
            )
    requested_path = arguments.get("path")
    if requested_path is not None:
        if not isinstance(requested_path, str):
            raise _denied(
                discovery, grant, name, required, "project_path_not_a_string"
            )
        try:
            requested_root = Path(requested_path).resolve(strict=True)
        except OSError:
            raise _denied(
                discovery, grant, name, required, "project_path_unavailable"
            ) from None
        if requested_root != discovery.project_root:
            raise _denied(
                discovery, grant, name, required, "project_scope_mismatch"
            )


def required_capability(
    name: str,
    spec: dict[str, Any],
    catalog: dict[str, dict[str, Any]],
    _seen: frozenset[str] = frozenset(),
) -> str:
    if name in _seen:
        raise ValueError(f"cyclic capability replacement metadata for {name}")
    if name in _UNATTENDED_TOOLS:
        return UNATTENDED
    if name in _PROPOSE_READ_TOOLS:
        return PROPOSE
    write_class = spec.get("x_public_write_surface_class")
    if write_class == "proposal_gateway_apply":
        return UNATTENDED if "accept_apply" in name else APPLY_APPROVED
    if write_class is not None:
        required = _WRITE_CLASS_CAPABILITY.get(write_class)
        if required is None:
            raise ValueError(f"unclassified Datum write capability {write_class!r}")
        return required
    replacements = spec.get("x_canonical_replacements", [])
    replacement_requirements = []
    for replacement in replacements:
        canonical = catalog.get(replacement)
        if canonical is not None:
            replacement_requirements.append(
                required_capability(
                    replacement,
                    canonical,
                    catalog,
                    _seen | {name},
                )
            )
    if replacement_requirements:
        return max(replacement_requirements, key=_CAPABILITY_LEVEL.__getitem__)
    return INSPECT


def _denied(
    discovery: DiscoveryScope,
    grant: AgentCapabilityGrant,
    name: str,
    required: str,
    reason: str,
) -> AgentCapabilityError:
    return AgentCapabilityError(
        "capability_denied",
        f"Datum session authority does not permit {name}",
        {
            "reason": reason,
            "tool": name,
            "required_capability": required,
            "granted_capabilities": list(grant.capabilities),
            "approval_policy": grant.approval_policy,
            "unattended_tools": sorted(grant.unattended_tools),
            "project_root": str(discovery.project_root),
            "terminal_session_id": discovery.terminal_session_id,
            "context_id": discovery.context_id,
        },
    )
