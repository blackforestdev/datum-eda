"""Static MCP tool catalog exposed by tools/list."""

from __future__ import annotations

from typing import Any

TOOLS: list[dict[str, Any]] = [
    {
        "name": "open_project",
        "description": "Import a KiCad or Eagle design into the engine session.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}},
            "required": ["path"],
        },
    },
    {
        "name": "close_project",
        "description": "Close the current in-memory project session.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "save",
        "description": "Save the current imported design to a path or back to its original file.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": ["string", "null"]}},
        },
    },
    {
        "name": "delete_track",
        "description": "Delete one board track by UUID in the current M3 slice.",
        "inputSchema": {
            "type": "object",
            "properties": {"uuid": {"type": "string"}},
            "required": ["uuid"],
        },
    },
    {
        "name": "delete_component",
        "description": "Delete one board component by UUID in the current M3 slice.",
        "inputSchema": {
            "type": "object",
            "properties": {"uuid": {"type": "string"}},
            "required": ["uuid"],
        },
    },
    {
        "name": "move_component",
        "description": "Move one board component by UUID in the current M3 slice.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "uuid": {"type": "string"},
                "x_mm": {"type": "number"},
                "y_mm": {"type": "number"},
                "rotation_deg": {"type": ["number", "null"]},
            },
            "required": ["uuid", "x_mm", "y_mm"],
        },
    },
    {
        "name": "rotate_component",
        "description": "Rotate one board component by UUID in the current M3 slice.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "uuid": {"type": "string"},
                "rotation_deg": {"type": "number"},
            },
            "required": ["uuid", "rotation_deg"],
        },
    },
    {
        "name": "set_value",
        "description": "Set one board component value by UUID in the current M3 slice.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "uuid": {"type": "string"},
                "value": {"type": "string"},
            },
            "required": ["uuid", "value"],
        },
    },
    {
        "name": "assign_part",
        "description": "Assign one pool part to a board component by UUID in the current M3 slice.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "uuid": {"type": "string"},
                "part_uuid": {"type": "string"},
            },
            "required": ["uuid", "part_uuid"],
        },
    },
    {
        "name": "set_package",
        "description": "Assign one pool package to a board component by UUID in the current M3 slice.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "uuid": {"type": "string"},
                "package_uuid": {"type": "string"},
            },
            "required": ["uuid", "package_uuid"],
        },
    },
    {
        "name": "set_package_with_part",
        "description": "Assign one pool package plus an explicit compatible pool part to a board component by UUID in the current M3 slice.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "uuid": {"type": "string"},
                "package_uuid": {"type": "string"},
                "part_uuid": {"type": "string"},
            },
            "required": ["uuid", "package_uuid", "part_uuid"],
        },
    },
    {
        "name": "replace_component",
        "description": "Replace one board component with an explicit compatible pool part+package in the current M3 slice.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "uuid": {"type": "string"},
                "package_uuid": {"type": "string"},
                "part_uuid": {"type": "string"},
            },
            "required": ["uuid", "package_uuid", "part_uuid"],
        },
    },
    {
        "name": "replace_components",
        "description": "Replace multiple board components in one transaction using explicit compatible pool part+package selections in the current M3 slice.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "replacements": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "uuid": {"type": "string"},
                            "package_uuid": {"type": "string"},
                            "part_uuid": {"type": "string"},
                        },
                        "required": ["uuid", "package_uuid", "part_uuid"],
                    },
                }
            },
            "required": ["replacements"],
        },
    },
    {
        "name": "set_reference",
        "description": "Set one board component reference by UUID in the current M3 slice.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "uuid": {"type": "string"},
                "reference": {"type": "string"},
            },
            "required": ["uuid", "reference"],
        },
    },
    {
        "name": "set_net_class",
        "description": "Assign one board net to a concrete net class in the current M3 slice.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "net_uuid": {"type": "string"},
                "class_name": {"type": "string"},
                "clearance": {"type": "integer"},
                "track_width": {"type": "integer"},
                "via_drill": {"type": "integer"},
                "via_diameter": {"type": "integer"},
                "diffpair_width": {"type": "integer"},
                "diffpair_gap": {"type": "integer"},
            },
            "required": [
                "net_uuid",
                "class_name",
                "clearance",
                "track_width",
                "via_drill",
                "via_diameter",
            ],
        },
    },
    {
        "name": "delete_via",
        "description": "Delete one board via by UUID in the current M3 slice.",
        "inputSchema": {
            "type": "object",
            "properties": {"uuid": {"type": "string"}},
            "required": ["uuid"],
        },
    },
    {
        "name": "set_design_rule",
        "description": "Create or update one board design rule in the current M3 slice.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "rule_type": {"type": "string"},
                "scope": {"type": ["object", "string"]},
                "parameters": {"type": "object"},
                "priority": {"type": "integer"},
                "name": {"type": ["string", "null"]},
            },
            "required": ["rule_type", "scope", "parameters", "priority"],
        },
    },
    {
        "name": "undo",
        "description": "Undo the last board transaction in the current session.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "redo",
        "description": "Redo the last undone board transaction in the current session.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "search_pool",
        "description": "Search imported pool parts by keyword.",
        "inputSchema": {
            "type": "object",
            "properties": {"query": {"type": "string"}},
            "required": ["query"],
        },
    },
    {
        "name": "get_part",
        "description": "Return detailed pool part metadata for a UUID.",
        "inputSchema": {
            "type": "object",
            "properties": {"uuid": {"type": "string"}},
            "required": ["uuid"],
        },
    },
    {
        "name": "get_package",
        "description": "Return detailed package geometry/footprint metadata for a UUID.",
        "inputSchema": {
            "type": "object",
            "properties": {"uuid": {"type": "string"}},
            "required": ["uuid"],
        },
    },
    {
        "name": "get_package_change_candidates",
        "description": "Return compatible target-package candidates for a board component UUID.",
        "inputSchema": {
            "type": "object",
            "properties": {"uuid": {"type": "string"}},
            "required": ["uuid"],
        },
    },
    {
        "name": "get_part_change_candidates",
        "description": "Return compatible target-part candidates for a board component UUID.",
        "inputSchema": {
            "type": "object",
            "properties": {"uuid": {"type": "string"}},
            "required": ["uuid"],
        },
    },
    {
        "name": "get_component_replacement_plan",
        "description": "Return a unified replacement-planning report for a board component UUID.",
        "inputSchema": {
            "type": "object",
            "properties": {"uuid": {"type": "string"}},
            "required": ["uuid"],
        },
    },
    {
        "name": "get_components",
        "description": "Return the imported board component list for the open project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_netlist",
        "description": "Return canonical netlist view for the open board or schematic project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_check_report",
        "description": "Return the unified board/schematic checking report.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_net_info",
        "description": "Return the current imported board net list and routing metrics.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_unrouted",
        "description": "Return unrouted airwires for the current imported board.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_schematic_net_info",
        "description": "Return the current imported schematic net list and connectivity counts.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_board_summary",
        "description": "Return the imported board summary for the open project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_schematic_summary",
        "description": "Return the imported schematic summary for the open project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_sheets",
        "description": "Return imported schematic sheets for the open project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_labels",
        "description": "Return the imported schematic labels for the open project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_symbols",
        "description": "Return the imported schematic symbols for the open project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_symbol_fields",
        "description": "Return authored fields for a specific schematic symbol UUID.",
        "inputSchema": {
            "type": "object",
            "properties": {"symbol_uuid": {"type": "string"}},
            "required": ["symbol_uuid"],
        },
    },
    {
        "name": "get_ports",
        "description": "Return the imported schematic interface ports for the open project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_buses",
        "description": "Return the imported schematic buses for the open project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_bus_entries",
        "description": "Return the imported schematic bus-entry objects for the open project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_hierarchy",
        "description": "Return the imported schematic hierarchy for the open project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_noconnects",
        "description": "Return the imported schematic no-connect markers for the open project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_connectivity_diagnostics",
        "description": "Return raw connectivity diagnostics for the open project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_design_rules",
        "description": "Return authored design-rule entries for the open board project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "run_erc",
        "description": "Return raw ERC findings for the open schematic project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "run_drc",
        "description": "Return DRC report for the open board project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "explain_violation",
        "description": "Explain a specific ERC/DRC finding by domain and index.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "domain": {"type": "string", "enum": ["erc", "drc"]},
                "index": {"type": "integer"},
            },
            "required": ["domain", "index"],
        },
    },
]
