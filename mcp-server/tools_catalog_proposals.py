"""Proposal MCP tool schemas."""

PROPOSAL_TOOL_SCHEMAS = {
    "create_proposal": {
        "description": "Create a non-mutating draft proposal from an OperationBatch JSON file.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "batch": {"type": "string"},
                "rationale": {"type": "string"},
                "proposal": {"type": "string"},
                "source": {"type": "string", "enum": ["manual", "cli", "tool", "assistant", "check", "import"]},
                "checks_run": {"type": "array", "items": {"type": "string"}},
                "finding_fingerprints": {"type": "array", "items": {"type": "string"}},
            },
            "required": ["path", "batch", "rationale"],
        },
        "x_dispatch_args": ["path", "batch", "rationale", "proposal", "source", "checks_run", "finding_fingerprints"],
        "x_dispatch_defaults": {"source": "tool", "checks_run": [], "finding_fingerprints": []},
    },
    "create_draw_wire_proposal": {
        "description": "Create a non-mutating draft proposal to draw one native-project schematic wire.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "sheet": {"type": "string"},
                "from_x_nm": {"type": "integer"},
                "from_y_nm": {"type": "integer"},
                "to_x_nm": {"type": "integer"},
                "to_y_nm": {"type": "integer"},
                "proposal": {"type": ["string", "null"]},
                "rationale": {"type": ["string", "null"]},
            },
            "required": ["path", "sheet", "from_x_nm", "from_y_nm", "to_x_nm", "to_y_nm"],
        },
        "x_dispatch_args": ["path", "sheet", "from_x_nm", "from_y_nm", "to_x_nm", "to_y_nm", "proposal", "rationale"],
    },
    "create_place_label_proposal": {
        "description": "Create a non-mutating draft proposal to place one native-project schematic label.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "sheet": {"type": "string"},
                "name": {"type": "string"},
                "x_nm": {"type": "integer"},
                "y_nm": {"type": "integer"},
                "kind": {"type": ["string", "null"]},
                "proposal": {"type": ["string", "null"]},
                "rationale": {"type": ["string", "null"]},
            },
            "required": ["path", "sheet", "name", "x_nm", "y_nm"],
        },
        "x_dispatch_args": ["path", "sheet", "name", "x_nm", "y_nm", "kind", "proposal", "rationale"],
    },
    "create_place_symbol_proposal": {
        "description": "Create a non-mutating draft proposal to place one native-project schematic symbol.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "sheet": {"type": "string"},
                "reference": {"type": "string"},
                "value": {"type": "string"},
                "x_nm": {"type": "integer"},
                "y_nm": {"type": "integer"},
                "lib_id": {"type": ["string", "null"]},
                "rotation_deg": {"type": ["integer", "null"]},
                "mirrored": {"type": ["boolean", "null"]},
                "proposal": {"type": ["string", "null"]},
                "rationale": {"type": ["string", "null"]},
            },
            "required": ["path", "sheet", "reference", "value", "x_nm", "y_nm"],
        },
        "x_dispatch_args": ["path", "sheet", "reference", "value", "x_nm", "y_nm", "lib_id", "rotation_deg", "mirrored", "proposal", "rationale"],
    },
    "get_proposals": {
        "description": "Read resolver-discovered proposal sidecars for a native project.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}},
            "required": ["path"],
        },
    },
    "show_proposal": {
        "description": "Show one persisted native-project proposal plus validation state.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}, "proposal": {"type": "string"}},
            "required": ["path", "proposal"],
        },
    },
    "preview_proposal": {
        "description": "Preview one persisted native-project proposal's classified diff without writing shards.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}, "proposal": {"type": "string"}},
            "required": ["path", "proposal"],
        },
    },
    "validate_proposal": {
        "description": "Validate one persisted native-project proposal against the current model revision.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}, "proposal": {"type": "string"}},
            "required": ["path", "proposal"],
        },
    },
    "defer_proposal": {
        "description": "Defer one draft native-project proposal without applying it.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}, "proposal": {"type": "string"}},
            "required": ["path", "proposal"],
        },
    },
    "reject_proposal": {
        "description": "Reject one draft native-project proposal without applying it.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}, "proposal": {"type": "string"}},
            "required": ["path", "proposal"],
        },
    },
    "review_proposal": {
        "description": "Review one persisted native-project proposal as accepted, deferred, or rejected.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}, "proposal": {"type": "string"}, "status": {"type": "string", "enum": ["accepted", "deferred", "rejected"]}},
            "required": ["path", "proposal", "status"],
        },
    },
    "apply_proposal": {
        "description": "Apply one accepted persisted native-project proposal through the generic proposal gateway.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}, "proposal": {"type": "string"}},
            "required": ["path", "proposal"],
        },
    },
    "accept_apply_proposal": {
        "description": "Accept one draft native-project proposal and apply it through the generic proposal gateway.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}, "proposal": {"type": "string"}},
            "required": ["path", "proposal"],
        },
    },
}
