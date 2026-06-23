"""Check and standards-repair MCP tool schemas."""

CHECK_TOOL_SCHEMAS = {
    "get_check_run": {
        "description": "Run a native project CheckRun profile and persist a resolver-owned CheckRun evidence artifact.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "profile": {"type": ["string", "null"]},
            },
            "required": ["path"],
        },
    },
    "get_check_runs": {
        "description": "List resolver-discovered persisted CheckRun evidence artifacts.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}},
            "required": ["path"],
        },
    },
    "show_check_run": {
        "description": "Show one resolver-discovered persisted CheckRun evidence artifact by UUID.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "check_run": {"type": "string"},
            },
            "required": ["path", "check_run"],
        },
    },
    "get_check_profiles": {
        "description": "List supported native-project CheckRun profiles.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}},
            "required": ["path"],
        },
    },
    "get_zone_fills": {
        "description": "Return resolver-derived native board zone-fill state without pretending unfilled zones are copper.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}},
            "required": ["path"],
        },
    },
    "fill_zones": {
        "description": "Persist honest generated ZoneFill evidence for native board zones. The bounded solver fills closed same-net zones, supports one rectangular foreign pad/via cutout with positive netclass clearance, and records Unsupported evidence for thermals, keepouts, unresolved pads, tracks, or general pour cases.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "zone": {"type": ["string", "null"]},
                "net": {"type": ["string", "null"]},
            },
            "required": ["path"],
        },
    },
    "generate_standards_repair_proposals": {
        "description": "Generate draft standards-repair proposals from persisted process-aperture check findings.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}},
            "required": ["path"],
        },
    },
    "waive_finding": {
        "description": "Author a fingerprint-scoped check finding waiver through the native project journal.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "fingerprint": {"type": "string"},
                "rationale": {"type": "string"},
                "created_by": {"type": ["string", "null"]},
            },
            "required": ["path", "fingerprint", "rationale"],
        },
    },
    "accept_deviation": {
        "description": "Accept a fingerprint-scoped check finding as a deviation through the native project journal.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "fingerprint": {"type": "string"},
                "rationale": {"type": "string"},
                "accepted_by": {"type": ["string", "null"]},
            },
            "required": ["path", "fingerprint", "rationale"],
        },
    },
}
