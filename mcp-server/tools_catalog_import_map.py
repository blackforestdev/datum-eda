"""Import map MCP tool schemas."""

IMPORT_MAP_TOOL_SCHEMAS = {
    "get_import_map": {
        "description": "Read resolver-validated import-key identity mappings for a native project.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}},
            "required": ["path"],
        },
    },
}
