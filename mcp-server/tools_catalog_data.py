"""Shared MCP tool registration data."""

from __future__ import annotations

from typing import Any


TOOL_SPECS: list[dict[str, Any]] = [
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
        "name": "validate_project",
        "description": "Validate one native project directory for required files, supported schema versions, duplicate UUID consistency, and non-dangling persisted references.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}},
            "required": ["path"],
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
        "name": "apply_component_replacement_plan",
        "description": "Apply one or more replacement-plan selections in one transaction, resolving the missing side of each component replacement from the current compatibility plan.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "replacements": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "uuid": {"type": "string"},
                            "package_uuid": {"type": ["string", "null"]},
                            "part_uuid": {"type": ["string", "null"]},
                        },
                        "required": ["uuid"],
                    },
                }
            },
            "required": ["replacements"],
        },
    },
    {
        "name": "apply_component_replacement_policy",
        "description": "Apply one or more deterministic replacement policies in one transaction, choosing the best compatible package or part from the current replacement plan.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "replacements": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "uuid": {"type": "string"},
                            "policy": {
                                "type": "string",
                                "enum": ["best_compatible_package", "best_compatible_part"],
                            },
                        },
                        "required": ["uuid", "policy"],
                    },
                }
            },
            "required": ["replacements"],
        },
    },
    {
        "name": "apply_scoped_component_replacement_policy",
        "description": "Apply a deterministic replacement policy to all components matching a scoped filter in one transaction.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "scope": {
                    "type": "object",
                    "properties": {
                        "reference_prefix": {"type": ["string", "null"]},
                        "value_equals": {"type": ["string", "null"]},
                        "current_package_uuid": {"type": ["string", "null"]},
                        "current_part_uuid": {"type": ["string", "null"]},
                    },
                },
                "policy": {
                    "type": "string",
                    "enum": ["best_compatible_package", "best_compatible_part"],
                },
            },
            "required": ["scope", "policy"],
        },
    },
    {
        "name": "apply_scoped_component_replacement_plan",
        "description": "Apply a previously previewed scoped component replacement plan without re-resolving policy.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "plan": {
                    "type": "object",
                    "properties": {
                        "scope": {"type": "object"},
                        "policy": {
                            "type": "string",
                            "enum": ["best_compatible_package", "best_compatible_part"],
                        },
                        "replacements": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "component_uuid": {"type": "string"},
                                    "current_reference": {"type": "string"},
                                    "current_value": {"type": "string"},
                                    "current_part_uuid": {"type": ["string", "null"]},
                                    "current_package_uuid": {"type": "string"},
                                    "target_part_uuid": {"type": "string"},
                                    "target_package_uuid": {"type": "string"},
                                    "target_value": {"type": "string"},
                                    "target_package_name": {"type": "string"},
                                },
                                "required": [
                                    "component_uuid",
                                    "current_reference",
                                    "current_value",
                                    "current_part_uuid",
                                    "current_package_uuid",
                                    "target_part_uuid",
                                    "target_package_uuid",
                                    "target_value",
                                    "target_package_name",
                                ],
                            },
                        },
                    },
                    "required": ["scope", "policy", "replacements"],
                }
            },
            "required": ["plan"],
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
        "x_dispatch_defaults": {
            "diffpair_width": 0,
            "diffpair_gap": 0,
        },
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
        "name": "get_scoped_component_replacement_plan",
        "description": "Preview the exact replacements a scoped compatibility policy would choose before mutation.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "scope": {
                    "type": "object",
                    "properties": {
                        "reference_prefix": {"type": ["string", "null"]},
                        "value_equals": {"type": ["string", "null"]},
                        "current_package_uuid": {"type": ["string", "null"]},
                        "current_part_uuid": {"type": ["string", "null"]},
                    },
                },
                "policy": {
                    "type": "string",
                    "enum": ["best_compatible_package", "best_compatible_part"],
                },
            },
            "required": ["scope", "policy"],
        },
    },
    {
        "name": "edit_scoped_component_replacement_plan",
        "description": "Exclude or override items in a scoped replacement preview without hand-editing raw JSON.",
        "x_dispatch_defaults": {
            "exclude_component_uuids": [],
            "overrides": [],
        },
        "inputSchema": {
            "type": "object",
            "properties": {
                "plan": {"type": "object"},
                "exclude_component_uuids": {
                    "type": "array",
                    "items": {"type": "string"},
                },
                "overrides": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "component_uuid": {"type": "string"},
                            "target_package_uuid": {"type": "string"},
                            "target_part_uuid": {"type": "string"},
                        },
                        "required": [
                            "component_uuid",
                            "target_package_uuid",
                            "target_part_uuid",
                        ],
                    },
                },
            },
            "required": ["plan"],
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
    {
        "name": "export_route_path_proposal",
        "description": "Export one deterministic route proposal artifact from an accepted current route-path candidate family.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "net_uuid": {"type": "string"},
                "from_anchor_pad_uuid": {"type": "string"},
                "to_anchor_pad_uuid": {"type": "string"},
                "candidate": {
                    "type": "string",
                    "enum": [
                        "route-path-candidate",
                        "route-path-candidate-via",
                        "route-path-candidate-two-via",
                        "route-path-candidate-three-via",
                        "route-path-candidate-four-via",
                        "route-path-candidate-five-via",
                        "route-path-candidate-six-via",
                        "route-path-candidate-authored-via-chain",
                        "route-path-candidate-orthogonal-dogleg",
                        "route-path-candidate-orthogonal-two-bend",
                        "route-path-candidate-orthogonal-graph",
                        "route-path-candidate-orthogonal-graph-via",
                        "route-path-candidate-orthogonal-graph-two-via",
                        "route-path-candidate-orthogonal-graph-three-via",
                        "route-path-candidate-orthogonal-graph-four-via",
                        "route-path-candidate-orthogonal-graph-five-via",
                        "route-path-candidate-orthogonal-graph-six-via",
                        "authored-copper-plus-one-gap",
                        "authored-copper-graph",
                    ],
                },
                "policy": {
                    "type": "string",
                    "enum": [
                        "plain",
                        "zone_aware",
                        "obstacle_aware",
                        "zone_obstacle_aware",
                        "zone_obstacle_topology_aware",
                        "zone_obstacle_topology_layer_balance_aware",
                    ],
                },
                "out": {"type": "string"},
            },
            "required": [
                "path",
                "net_uuid",
                "from_anchor_pad_uuid",
                "to_anchor_pad_uuid",
                "candidate",
                "out",
            ],
        },
    },
    {
        "name": "route_proposal",
        "description": "Select the current deterministic route proposal for one net and anchor pair.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "net_uuid": {"type": "string"},
                "from_anchor_pad_uuid": {"type": "string"},
                "to_anchor_pad_uuid": {"type": "string"},
                "profile": {
                    "type": "string",
                    "enum": ["default", "authored-copper-priority"],
                },
            },
            "required": [
                "path",
                "net_uuid",
                "from_anchor_pad_uuid",
                "to_anchor_pad_uuid",
            ],
        },
    },
    {
        "name": "route_strategy_report",
        "description": "Report which accepted selector profile should be used for one deterministic routing objective and show the current live selector outcome under that profile.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "net_uuid": {"type": "string"},
                "from_anchor_pad_uuid": {"type": "string"},
                "to_anchor_pad_uuid": {"type": "string"},
                "objective": {
                    "type": "string",
                    "enum": ["default", "authored-copper-priority"],
                },
            },
            "required": [
                "path",
                "net_uuid",
                "from_anchor_pad_uuid",
                "to_anchor_pad_uuid",
            ],
        },
    },
    {
        "name": "route_strategy_compare",
        "description": "Compare the accepted deterministic routing objectives/profiles, report the current live selector outcome for each, and recommend one profile under the approved comparison rule.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "net_uuid": {"type": "string"},
                "from_anchor_pad_uuid": {"type": "string"},
                "to_anchor_pad_uuid": {"type": "string"},
            },
            "required": [
                "path",
                "net_uuid",
                "from_anchor_pad_uuid",
                "to_anchor_pad_uuid",
            ],
        },
    },
    {
        "name": "route_strategy_delta",
        "description": "Report the bounded decision delta between the accepted deterministic routing objectives/profiles using the current live selector outcomes and one explicit delta classification.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "net_uuid": {"type": "string"},
                "from_anchor_pad_uuid": {"type": "string"},
                "to_anchor_pad_uuid": {"type": "string"},
            },
            "required": [
                "path",
                "net_uuid",
                "from_anchor_pad_uuid",
                "to_anchor_pad_uuid",
            ],
        },
    },
    {
        "name": "write_route_strategy_curated_fixture_suite",
        "description": "Write one deterministic curated native-project fixture suite plus a versioned batch request manifest for repeated route-strategy batch evidence runs.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "out_dir": {"type": "string"},
                "manifest": {"type": ["string", "null"]},
            },
            "required": ["out_dir"],
        },
    },
    {
        "name": "capture_route_strategy_curated_baseline",
        "description": "Materialize the curated route-strategy fixture suite, evaluate it, and save one reusable versioned batch-result baseline artifact.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "out_dir": {"type": "string"},
                "manifest": {"type": ["string", "null"]},
                "result": {"type": ["string", "null"]},
            },
            "required": ["out_dir"],
        },
    },
    {
        "name": "route_strategy_batch_evaluate",
        "description": "Evaluate the current accepted M6 strategy surfaces across a versioned batch request manifest and return per-request evidence plus aggregate summary counts.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "requests": {"type": "string"},
            },
            "required": ["requests"],
        },
    },
    {
        "name": "inspect_route_strategy_batch_result",
        "description": "Inspect one saved versioned route-strategy batch result artifact and report summary counts, per-request outcomes, and malformed entries.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "artifact": {"type": "string"},
            },
            "required": ["artifact"],
        },
    },
    {
        "name": "validate_route_strategy_batch_result",
        "description": "Validate one saved versioned route-strategy batch result artifact for supported version, required fields, and summary/result count integrity.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "artifact": {"type": "string"},
            },
            "required": ["artifact"],
        },
    },
    {
        "name": "compare_route_strategy_batch_result",
        "description": "Compare two saved versioned route-strategy batch result artifacts by request_id and aggregate summary counts without live re-evaluation.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "before": {"type": "string"},
                "after": {"type": "string"},
            },
            "required": ["before", "after"],
        },
    },
    {
        "name": "gate_route_strategy_batch_result",
        "description": "Evaluate two saved versioned route-strategy batch result artifacts against one explicit deterministic CI gate policy.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "before": {"type": "string"},
                "after": {"type": "string"},
                "policy": {
                    "type": "string",
                    "enum": [
                        "strict_identical",
                        "allow_aggregate_only",
                        "fail_on_recommendation_change",
                    ],
                },
            },
            "required": ["before", "after"],
        },
    },
    {
        "name": "summarize_route_strategy_batch_results",
        "description": "Summarize saved route-strategy batch result artifacts from one directory or explicit list, with optional baseline gate summary.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "dir": {"type": "string"},
                "artifacts": {
                    "type": "array",
                    "items": {"type": "string"},
                },
                "baseline": {"type": "string"},
                "policy": {
                    "type": "string",
                    "enum": [
                        "strict_identical",
                        "allow_aggregate_only",
                        "fail_on_recommendation_change",
                    ],
                },
            },
        },
    },
    {
        "name": "route_proposal_explain",
        "description": "Explain family-level selection and rejection for the current deterministic route proposal.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "net_uuid": {"type": "string"},
                "from_anchor_pad_uuid": {"type": "string"},
                "to_anchor_pad_uuid": {"type": "string"},
                "profile": {
                    "type": "string",
                    "enum": ["default", "authored-copper-priority"],
                },
            },
            "required": [
                "path",
                "net_uuid",
                "from_anchor_pad_uuid",
                "to_anchor_pad_uuid",
            ],
        },
    },
    {
        "name": "export_route_proposal",
        "description": "Export the currently selected deterministic route proposal as a native route proposal artifact.",
        "x_dispatch_args": [
            "path",
            "net_uuid",
            "from_anchor_pad_uuid",
            "to_anchor_pad_uuid",
            "out",
            "profile",
        ],
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "net_uuid": {"type": "string"},
                "from_anchor_pad_uuid": {"type": "string"},
                "to_anchor_pad_uuid": {"type": "string"},
                "profile": {
                    "type": "string",
                    "enum": ["default", "authored-copper-priority"],
                },
                "out": {"type": "string"},
            },
            "required": [
                "path",
                "net_uuid",
                "from_anchor_pad_uuid",
                "to_anchor_pad_uuid",
                "out",
            ],
        },
    },
    {
        "name": "route_apply",
        "description": "Apply one accepted current deterministic route candidate directly into native board copper.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "net_uuid": {"type": "string"},
                "from_anchor_pad_uuid": {"type": "string"},
                "to_anchor_pad_uuid": {"type": "string"},
                "candidate": {
                    "type": "string",
                    "enum": [
                        "route-path-candidate",
                        "route-path-candidate-via",
                        "route-path-candidate-two-via",
                        "route-path-candidate-three-via",
                        "route-path-candidate-four-via",
                        "route-path-candidate-five-via",
                        "route-path-candidate-six-via",
                        "route-path-candidate-authored-via-chain",
                        "route-path-candidate-orthogonal-dogleg",
                        "route-path-candidate-orthogonal-two-bend",
                        "route-path-candidate-orthogonal-graph",
                        "route-path-candidate-orthogonal-graph-via",
                        "route-path-candidate-orthogonal-graph-two-via",
                        "route-path-candidate-orthogonal-graph-three-via",
                        "route-path-candidate-orthogonal-graph-four-via",
                        "route-path-candidate-orthogonal-graph-five-via",
                        "route-path-candidate-orthogonal-graph-six-via",
                        "authored-copper-plus-one-gap",
                        "authored-copper-graph",
                    ],
                },
                "policy": {
                    "type": "string",
                    "enum": [
                        "plain",
                        "zone_aware",
                        "obstacle_aware",
                        "zone_obstacle_aware",
                        "zone_obstacle_topology_aware",
                        "zone_obstacle_topology_layer_balance_aware",
                    ],
                },
            },
            "required": [
                "path",
                "net_uuid",
                "from_anchor_pad_uuid",
                "to_anchor_pad_uuid",
                "candidate",
            ],
        },
    },
    {
        "name": "route_apply_selected",
        "description": "Apply the currently selected deterministic route proposal directly to the current native project.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "net_uuid": {"type": "string"},
                "from_anchor_pad_uuid": {"type": "string"},
                "to_anchor_pad_uuid": {"type": "string"},
                "profile": {
                    "type": "string",
                    "enum": ["default", "authored-copper-priority"],
                },
            },
            "required": [
                "path",
                "net_uuid",
                "from_anchor_pad_uuid",
                "to_anchor_pad_uuid",
            ],
        },
    },
    {
        "name": "inspect_route_proposal_artifact",
        "description": "Inspect one native route proposal artifact without consulting live project state.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "artifact": {"type": "string"},
            },
            "required": ["artifact"],
        },
    },
    {
        "name": "revalidate_route_proposal_artifact",
        "description": "Revalidate one native route proposal artifact against the current live project state without applying it.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "artifact": {"type": "string"},
            },
            "required": ["path", "artifact"],
        },
    },
    {
        "name": "apply_route_proposal_artifact",
        "description": "Apply one native route proposal artifact when it still matches the current live project state.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "artifact": {"type": "string"},
            },
            "required": ["path", "artifact"],
        },
    },
]


def _catalog_tool(spec: dict[str, Any]) -> dict[str, Any]:
    return {key: value for key, value in spec.items() if not key.startswith("x_")}


TOOLS: list[dict[str, Any]] = [_catalog_tool(spec) for spec in TOOL_SPECS]
TOOL_BY_NAME: dict[str, dict[str, Any]] = {spec["name"]: spec for spec in TOOL_SPECS}
