"""Canonical GP-CM03 Project-genesis MCP schema."""

from tools_catalog_preferences import GENERATION_REF

PROJECT_GENESIS_TOOL_SPEC = {
    "name": "datum.project.new",
    "description": "Create one native Datum Project from an explicit Global or factory Units snapshot.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "request_id": {"type": "string", "format": "uuid"},
            "destination": {"type": "string", "minLength": 1},
            "project_name": {"type": "string", "minLength": 1},
            "project_id": {"oneOf": [{"type": "string", "format": "uuid"}, {"type": "null"}]},
            "units_source": {
                "oneOf": [
                    {
                        "type": "object",
                        "properties": {
                            "kind": {"const": "global"},
                            "expected_generation": {"oneOf": [GENERATION_REF, {"type": "null"}]},
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
            },
        },
        "required": ["request_id", "destination", "project_name", "units_source"],
        "additionalProperties": False,
    },
    "x_dispatch_method": "project_new",
    "x_dispatch_args": [
        "request_id", "destination", "project_name", "project_id", "units_source",
        "_transport_actor",
    ],
    "x_public_write_surface_class": "project_genesis",
    "x_write_surface_evidence": "GP-CM03 atomic Project publication",
}
