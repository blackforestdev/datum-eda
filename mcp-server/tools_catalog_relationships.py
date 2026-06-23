"""Relationship and variant MCP tool schemas."""

RELATIONSHIP_TOOL_SCHEMAS = {
    "get_component_instances": {
        "description": "Read authored ComponentInstance records plus resolver-bound symbol/package refs for a native project.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}},
            "required": ["path"],
        },
    },
    "bind_component_instance": {
        "description": "Create a journaled ComponentInstance binding between one schematic symbol and one board package.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "symbol": {"type": "string"},
                "package": {"type": "string"},
                "component_instance": {"type": ["string", "null"]},
            },
            "required": ["path", "symbol", "package"],
        },
    },
    "set_component_instance": {
        "description": "Update a journaled ComponentInstance symbol/package binding.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "component_instance": {"type": "string"},
                "symbol": {"type": "string"},
                "package": {"type": "string"},
            },
            "required": ["path", "component_instance", "symbol", "package"],
        },
    },
    "delete_component_instance": {
        "description": "Delete one journaled ComponentInstance binding by UUID.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "component_instance": {"type": "string"},
            },
            "required": ["path", "component_instance"],
        },
    },
    "get_relationships": {
        "description": "Read authored relationship records plus derived resolver status for a native project.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}},
            "required": ["path"],
        },
    },
    "get_variants": {
        "description": "Read authored variant overlays plus derived population/applicability for a native project.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}},
            "required": ["path"],
        },
    },
}
