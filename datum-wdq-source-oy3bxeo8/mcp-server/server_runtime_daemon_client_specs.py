#!/usr/bin/env python3
"""Daemon-client method spec table (extracted from server_runtime.py to keep
it within the file-size budget). The _REQUIRED sentinel identity is shared by
importing it back into server_runtime, where _build_client_params checks it."""
from __future__ import annotations
from typing import Any

_REQUIRED = object()
DAEMON_CLIENT_METHOD_SPECS: list[dict[str, Any]] = [
    {"name": "open_project", "params": [("path", _REQUIRED)]},
    {"name": "close_project", "params": []},
    {"name": "save", "params": [("path", None)]},
    {"name": "create_board_component_replacement_proposal", "params": [("path", _REQUIRED), ("component", _REQUIRED), ("package", None), ("part", None), ("value", None), ("proposal", None), ("rationale", None)]},
    {"name": "create_board_component_replacements_proposal", "params": [("path", _REQUIRED), ("replacements", _REQUIRED), ("proposal", None), ("rationale", None)]},
    {"name": "create_board_component_replacement_plan_proposal", "params": [("path", _REQUIRED), ("selections", _REQUIRED), ("proposal", None), ("rationale", None)]},
    {
        "name": "move_component",
        "params": [
            ("uuid", _REQUIRED),
            ("x_mm", _REQUIRED),
            ("y_mm", _REQUIRED),
            ("rotation_deg", None),
        ],
    },
    {
        "name": "rotate_component",
        "params": [("uuid", _REQUIRED), ("rotation_deg", _REQUIRED)],
        "fixed": {"x_mm": 0.0, "y_mm": 0.0},
    },
    {"name": "flip_component", "params": [("uuid", _REQUIRED), ("layer", _REQUIRED)]},
    {"name": "set_value", "params": [("uuid", _REQUIRED), ("value", _REQUIRED)]},
    {
        "name": "set_reference",
        "params": [("uuid", _REQUIRED), ("reference", _REQUIRED)],
    },
    {"name": "assign_part", "params": [("uuid", _REQUIRED), ("part_uuid", _REQUIRED)]},
    {"name": "set_package", "params": [("uuid", _REQUIRED), ("package_uuid", _REQUIRED)]},
    {
        "name": "set_package_with_part",
        "params": [
            ("uuid", _REQUIRED),
            ("package_uuid", _REQUIRED),
            ("part_uuid", _REQUIRED),
        ],
    },
    {
        "name": "replace_component",
        "params": [
            ("uuid", _REQUIRED),
            ("package_uuid", _REQUIRED),
            ("part_uuid", _REQUIRED),
        ],
    },
    {"name": "replace_components", "params": [("replacements", _REQUIRED)]},
    {
        "name": "apply_component_replacement_plan",
        "params": [("replacements", _REQUIRED)],
    },
    {
        "name": "apply_component_replacement_policy",
        "params": [("replacements", _REQUIRED)],
    },
    {
        "name": "apply_scoped_component_replacement_policy",
        "params": [("scope", _REQUIRED), ("policy", _REQUIRED)],
    },
    {
        "name": "apply_scoped_component_replacement_plan",
        "params": [("plan", _REQUIRED)],
    },
    {
        "name": "set_net_class",
        "params": [
            ("net_uuid", _REQUIRED),
            ("class_name", _REQUIRED),
            ("clearance", _REQUIRED),
            ("track_width", _REQUIRED),
            ("via_drill", _REQUIRED),
            ("via_diameter", _REQUIRED),
            ("diffpair_width", 0),
            ("diffpair_gap", 0),
        ],
    },
    {"name": "undo", "params": []},
    {"name": "redo", "params": []},
    {"name": "search_pool", "params": [("query", _REQUIRED)]},
    {"name": "get_part", "params": [("uuid", _REQUIRED)]},
    {"name": "get_package", "params": [("uuid", _REQUIRED)]},
    {"name": "get_package_change_candidates", "params": [("uuid", _REQUIRED)]},
    {"name": "get_part_change_candidates", "params": [("uuid", _REQUIRED)]},
    {"name": "get_component_replacement_plan", "params": [("uuid", _REQUIRED)]},
    {"name": "get_scoped_component_replacement_plan", "params": [("scope", _REQUIRED), ("policy", _REQUIRED)]},
    {
        "name": "edit_scoped_component_replacement_plan",
        "params": [
            ("plan", _REQUIRED),
            ("exclude_component_uuids", _REQUIRED),
            ("overrides", _REQUIRED),
        ],
    },
    {"name": "get_board_summary", "params": []},
    {"name": "get_components", "params": []},
    {"name": "get_netlist", "params": []},
    {"name": "get_schematic_summary", "params": []},
    {"name": "get_sheets", "params": []},
    {"name": "get_labels", "params": []},
    {"name": "get_symbols", "params": []},
    {"name": "get_symbol_fields", "params": [("symbol_uuid", _REQUIRED)]},
    {"name": "get_ports", "params": []},
    {"name": "get_buses", "params": []},
    {"name": "get_bus_entries", "params": []},
    {"name": "get_noconnects", "params": []},
    {"name": "get_hierarchy", "params": []},
    {"name": "get_net_info", "params": []},
    {"name": "get_unrouted", "params": []},
    {"name": "get_schematic_net_info", "params": []},
    {"name": "get_check_report", "params": []},
    {"name": "get_connectivity_diagnostics", "params": []},
    {"name": "get_design_rules", "params": []},
    {"name": "run_erc", "params": []},
    {"name": "run_drc", "params": [("rules", None)], "omit_none": True},
    {
        "name": "explain_violation",
        "params": [("domain", _REQUIRED), ("index", None), ("fingerprint", None)],
        "omit_none": True,
    },
]
