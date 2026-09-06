"""GP-CM03 canonical Global Preferences MCP tool schemas."""

from __future__ import annotations

from typing import Any


GENERATION_REF: dict[str, Any] = {
    "type": "object",
    "properties": {
        "repository_id": {"type": "string"},
        "generation": {"type": "integer", "minimum": 0},
        "parent_generation": {"type": ["integer", "null"], "minimum": 0},
        "canonical_manifest_digest": {"type": "string"},
        "committed_order": {"type": "integer", "minimum": 0},
        "writer_instance": {"type": "string"},
        "format_version": {"type": "integer", "minimum": 0},
        "canonicalization_version": {"type": "integer", "minimum": 0},
    },
    "required": [
        "repository_id", "generation", "parent_generation",
        "canonical_manifest_digest", "committed_order", "writer_instance",
        "format_version", "canonicalization_version",
    ],
    "additionalProperties": False,
}

HEAD_EXPECTATION: dict[str, Any] = {
    "oneOf": [
        {"type": "object", "properties": {"kind": {"const": "missing"}}, "required": ["kind"], "additionalProperties": False},
        {
            "type": "object",
            "properties": {
                "kind": {"const": "generation"},
                "generation": GENERATION_REF,
            },
            "required": ["kind", "generation"],
            "additionalProperties": False,
        },
    ]
}

MUTATION: dict[str, Any] = {
    "oneOf": [
        {
            "type": "object",
            "properties": {
                "kind": {"const": "set_user"},
                "key": {"type": "string"},
                "value": {},
                "expected": HEAD_EXPECTATION,
                "request_id": {"type": "string", "format": "uuid"},
                "reason": {"type": "string", "minLength": 1},
            },
            "required": ["kind", "key", "value", "expected", "request_id", "reason"],
            "additionalProperties": False,
        },
        {
            "type": "object",
            "properties": {
                "kind": {"const": "reset_user"},
                "key": {"type": "string"},
                "expected": HEAD_EXPECTATION,
                "request_id": {"type": "string", "format": "uuid"},
                "reason": {"type": "string", "minLength": 1},
            },
            "required": ["kind", "key", "expected", "request_id", "reason"],
            "additionalProperties": False,
        },
    ]
}

ACTOR: dict[str, Any] = {
    "type": "object",
    "properties": {
        "kind": {"const": "mcp_agent"},
        "session_id": {"type": "string", "minLength": 1},
        "local_actor_id": {"type": "string", "minLength": 1},
        "invocation_id": {"type": "string", "format": "uuid"},
    },
    "required": ["kind", "session_id", "local_actor_id", "invocation_id"],
    "additionalProperties": False,
}

PROPOSAL: dict[str, Any] = {
    "type": "object",
    "properties": {
        "schema": {
            "type": "object",
            "properties": {
                "name": {"const": "datum.preferences.proposal"},
                "version": {"const": 1},
            },
            "required": ["name", "version"],
            "additionalProperties": False,
        },
        "proposal_id": {"type": "string", "format": "uuid"},
        "proposal_digest": {"type": "string"},
        "prepared_against": HEAD_EXPECTATION,
        "active_catalog_digest": {"type": "string"},
        "mutation": MUTATION,
        "requesting_actor": ACTOR,
        "rationale": {"type": "string"},
        "creation_session": {"type": "string"},
    },
    "required": [
        "schema", "proposal_id", "proposal_digest", "prepared_against",
        "active_catalog_digest", "mutation", "requesting_actor", "rationale",
        "creation_session",
    ],
    "additionalProperties": False,
}


def _tool(name: str, description: str, properties: dict[str, Any], required: list[str]) -> dict[str, Any]:
    spec = {
        "name": name,
        "description": description,
        "inputSchema": {
            "type": "object",
            "properties": properties,
            "required": required,
            "additionalProperties": False,
        },
        "x_dispatch_method": name.replace("datum.preferences.", "preferences_").replace(".", "_"),
        "x_dispatch_args": [*properties, "_transport_actor"],
    }
    if name == "datum.preferences.proposal.accept_apply":
        spec["x_public_write_surface_class"] = "preference_proposal_apply"
        spec["x_write_surface_evidence"] = "GP-CM03 daemon-local human acceptance broker"
    return spec


PREFERENCE_TOOL_SPECS = [
    _tool("datum.preferences.describe", "Describe the active Global Preferences product surface.", {}, []),
    _tool("datum.preferences.list", "List active Global Preference values, optionally by section.", {"section": {"type": ["string", "null"]}}, []),
    _tool("datum.preferences.get", "Get one active Global Preference value.", {"key": {"type": "string"}}, ["key"]),
    _tool("datum.preferences.search", "Search active Global Preferences by label, description, stable key, or registered alias.", {"query": {"type": "string", "minLength": 1}}, ["query"]),
    _tool("datum.preferences.explain", "Explain resolution and provenance for one active Global Preference.", {"key": {"type": "string"}}, ["key"]),
    _tool(
        "datum.preferences.preview_project_units_seed",
        "Preview the exact eight-key Units seed without creating or changing a Project.",
        {
            "source": {
                "oneOf": [
                    {
                        "type": "object",
                        "properties": {
                            "kind": {"const": "global"},
                            "expected_generation": {
                                "oneOf": [GENERATION_REF, {"type": "null"}]
                            },
                        },
                        "required": ["kind"],
                        "additionalProperties": False,
                    },
                    {
                        "type": "object",
                        "properties": {
                            "kind": {"const": "factory"},
                            "profile_id": {"const": "datum.units.factory.v1"},
                        },
                        "required": ["kind", "profile_id"],
                        "additionalProperties": False,
                    },
                ]
            }
        },
        ["source"],
    ),
    _tool(
        "datum.preferences.proposal.prepare",
        "Prepare one portable Global Preference mutation proposal without writing.",
        {"mutation": MUTATION, "rationale": {"type": "string", "minLength": 1}},
        ["mutation", "rationale"],
    ),
    _tool("datum.preferences.proposal.validate", "Validate a Global Preference proposal without writing.", {"proposal": PROPOSAL}, ["proposal"]),
    _tool(
        "datum.preferences.proposal.accept_apply",
        "Apply a matching proposal only when this MCP session holds a daemon-local human acceptance.",
        {"proposal": PROPOSAL},
        ["proposal"],
    ),
    _tool("datum.preferences.proposal.reject", "Reject a Global Preference proposal without writing persistent state.", {"proposal": PROPOSAL}, ["proposal"]),
]
