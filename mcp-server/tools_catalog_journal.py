"""Native project journal MCP tool schemas."""

JOURNAL_TOOL_SCHEMAS = {
    "get_journal_list": {
        "description": "Return the resolver-backed native project transaction journal summary.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}},
            "required": ["path"],
        },
    },
    "get_journal_transaction": {
        "description": "Return one full native project transaction journal record by transaction UUID.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "transaction": {"type": "string"},
            },
            "required": ["path", "transaction"],
        },
    },
    "journal_undo": {
        "description": "Apply one native project journal undo as a compensating transaction, optionally guarded by model revision or journal tip.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "expected_model_revision": {"type": "string"},
                "expected_tip_transaction": {"type": "string"},
            },
            "required": ["path"],
        },
    },
    "journal_redo": {
        "description": "Apply one native project journal redo as a compensating transaction, optionally guarded by model revision or journal tip.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "expected_model_revision": {"type": "string"},
                "expected_tip_transaction": {"type": "string"},
            },
            "required": ["path"],
        },
    },
}
