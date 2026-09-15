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
        "description": "Create a journaled ComponentInstance binding between one or more schematic symbols and one board package.",
        "x_dispatch_args": ["path", "symbol", "package", "component_instance", "symbols", "part", "symbol_roles", "package_roles"],
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "symbol": {"type": "string"},
                "package": {"type": "string"},
                "part": {"type": ["string", "null"]},
                "symbol_roles": {"type": ["object", "array", "null"]},
                "package_roles": {"type": ["object", "array", "null"]},
                "component_instance": {"type": ["string", "null"]},
                "symbols": {
                    "type": "array",
                    "items": {"type": "string"},
                    "minItems": 1,
                },
            },
            "required": ["path", "package"],
            "anyOf": [{"required": ["symbol"]}, {"required": ["symbols"]}],
        },
    },
    "set_component_instance": {
        "description": "Update a journaled ComponentInstance symbol/package binding, preserving multi-symbol authored instances when `symbols` is supplied.",
        "x_dispatch_args": ["path", "component_instance", "symbol", "package", "symbols", "part", "symbol_roles", "package_roles"],
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "component_instance": {"type": "string"},
                "symbol": {"type": "string"},
                "package": {"type": "string"},
                "part": {"type": ["string", "null"]},
                "symbol_roles": {"type": ["object", "array", "null"]},
                "package_roles": {"type": ["object", "array", "null"]},
                "symbols": {
                    "type": "array",
                    "items": {"type": "string"},
                    "minItems": 1,
                },
            },
            "required": ["path", "component_instance", "package"],
            "anyOf": [{"required": ["symbol"]}, {"required": ["symbols"]}],
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
