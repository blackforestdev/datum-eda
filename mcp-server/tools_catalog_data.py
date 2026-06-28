from __future__ import annotations
from typing import Any
from tools_catalog_checks import CHECK_TOOL_SCHEMAS
from tools_catalog_drc import RUN_DRC_INPUT_SCHEMA
from tools_catalog_datum import DATUM_TOOL_SPECS
from tools_catalog_journal import JOURNAL_TOOL_SCHEMAS
from tools_catalog_legacy_aliases import _LEGACY_CANONICAL_ALIAS_NAMES
from tools_catalog_retirement import DEFAULT_HIDDEN_ALIAS_RETIREMENT_CRITERIA, canonical_replacements_for_hidden_alias, retirement_override_for_hidden_alias
from tools_catalog_import_map import IMPORT_MAP_TOOL_SCHEMAS; from tools_catalog_library import LIBRARY_TOOL_SCHEMAS; from tools_catalog_output_jobs import OUTPUT_JOB_TOOL_SCHEMAS; from tools_catalog_proposals import PROPOSAL_TOOL_SCHEMAS; from tools_catalog_relationships import RELATIONSHIP_TOOL_SCHEMAS
COMPATIBILITY_TOOL_SPECS: list[dict[str, object]] = [
    {"name": "get_check_run", **CHECK_TOOL_SCHEMAS["get_check_run"]}, {"name": "get_check_runs", **CHECK_TOOL_SCHEMAS["get_check_runs"]}, {"name": "show_check_run", **CHECK_TOOL_SCHEMAS["show_check_run"]}, {"name": "get_check_profiles", **CHECK_TOOL_SCHEMAS["get_check_profiles"]}, {"name": "fill_zones", **CHECK_TOOL_SCHEMAS["fill_zones"]}, {"name": "generate_standards_repair_proposals", **CHECK_TOOL_SCHEMAS["generate_standards_repair_proposals"]}, {"name": "waive_finding", **CHECK_TOOL_SCHEMAS["waive_finding"]}, {"name": "accept_deviation", **CHECK_TOOL_SCHEMAS["accept_deviation"]},
    {"name": "get_zone_fills", **CHECK_TOOL_SCHEMAS["get_zone_fills"]},
    {"name": "get_check_report", "description": "Legacy unified checking report compatibility alias; canonical check evidence is datum.check.run.", "inputSchema": {"type": "object", "properties": {}}},
    {"name": "run_erc", "description": "ERC compatibility alias returning live check_run_v1 with raw ERC findings under raw_report.erc; canonical persisted check evidence is datum.check.run.", "inputSchema": {"type": "object", "properties": {}}},
    {"name": "run_drc", "description": "DRC compatibility alias returning live check_run_v1 with raw DRC report under raw_report.drc; canonical persisted check evidence is datum.check.run.", "inputSchema": RUN_DRC_INPUT_SCHEMA},
    {"name": "explain_violation", "description": "Finding explanation compatibility alias. Prefer fingerprint; index remains accepted for legacy positional callers.", "inputSchema": {"type": "object", "properties": {"domain": {"type": "string", "enum": ["erc", "drc"]}, "index": {"type": ["integer", "null"]}, "fingerprint": {"type": ["string", "null"]}}, "required": ["domain"]}, "x_dispatch_args": ["domain", "index", "fingerprint"]},
    {"name": "get_journal_list", **JOURNAL_TOOL_SCHEMAS["get_journal_list"]}, {"name": "get_journal_transaction", **JOURNAL_TOOL_SCHEMAS["get_journal_transaction"]}, {"name": "journal_undo", **JOURNAL_TOOL_SCHEMAS["journal_undo"]}, {"name": "journal_redo", **JOURNAL_TOOL_SCHEMAS["journal_redo"]}, {"name": "undo", "description": "Legacy in-session board undo compatibility alias; canonical project undo is datum.journal.undo.", "inputSchema": {"type": "object", "properties": {}}}, {"name": "redo", "description": "Legacy in-session board redo compatibility alias; canonical project redo is datum.journal.redo.", "inputSchema": {"type": "object", "properties": {}}},
    {"name": "generate_artifacts", **OUTPUT_JOB_TOOL_SCHEMAS["generate_artifacts"]}, {"name": "get_artifacts", **OUTPUT_JOB_TOOL_SCHEMAS["get_artifacts"]}, {"name": "show_artifact", **OUTPUT_JOB_TOOL_SCHEMAS["show_artifact"]}, {"name": "get_artifact_files", **OUTPUT_JOB_TOOL_SCHEMAS["get_artifact_files"]}, {"name": "preview_artifact_file", **OUTPUT_JOB_TOOL_SCHEMAS["preview_artifact_file"]}, {"name": "compare_artifacts", **OUTPUT_JOB_TOOL_SCHEMAS["compare_artifacts"]}, {"name": "validate_artifact", **OUTPUT_JOB_TOOL_SCHEMAS["validate_artifact"]}, {"name": "start_output_job_run", **OUTPUT_JOB_TOOL_SCHEMAS["start_output_job_run"]}, {"name": "cancel_output_job_run", **OUTPUT_JOB_TOOL_SCHEMAS["cancel_output_job_run"]}, {"name": "export_manufacturing_set", **OUTPUT_JOB_TOOL_SCHEMAS["export_manufacturing_set"]}, {"name": "validate_manufacturing_set", **OUTPUT_JOB_TOOL_SCHEMAS["validate_manufacturing_set"]},
    {"name": "get_panel_projections", **OUTPUT_JOB_TOOL_SCHEMAS["get_panel_projections"]}, {"name": "get_manufacturing_plans", **OUTPUT_JOB_TOOL_SCHEMAS["get_manufacturing_plans"]}, {"name": "get_output_jobs", **OUTPUT_JOB_TOOL_SCHEMAS["get_output_jobs"]},
    {"name": "create_panel_projection_proposal", **OUTPUT_JOB_TOOL_SCHEMAS["create_panel_projection_proposal"]}, {"name": "update_panel_projection_proposal", **OUTPUT_JOB_TOOL_SCHEMAS["update_panel_projection_proposal"]}, {"name": "delete_panel_projection_proposal", **OUTPUT_JOB_TOOL_SCHEMAS["delete_panel_projection_proposal"]},
    {"name": "create_manufacturing_plan_proposal", **OUTPUT_JOB_TOOL_SCHEMAS["create_manufacturing_plan_proposal"]}, {"name": "update_manufacturing_plan_proposal", **OUTPUT_JOB_TOOL_SCHEMAS["update_manufacturing_plan_proposal"]}, {"name": "delete_manufacturing_plan_proposal", **OUTPUT_JOB_TOOL_SCHEMAS["delete_manufacturing_plan_proposal"]},
    {"name": "create_output_job_proposal", **OUTPUT_JOB_TOOL_SCHEMAS["create_output_job_proposal"]}, {"name": "update_output_job_proposal", **OUTPUT_JOB_TOOL_SCHEMAS["update_output_job_proposal"]}, {"name": "delete_output_job_proposal", **OUTPUT_JOB_TOOL_SCHEMAS["delete_output_job_proposal"]},
    {"name": "create_panel_projection", **OUTPUT_JOB_TOOL_SCHEMAS["create_panel_projection"]}, {"name": "update_panel_projection", **OUTPUT_JOB_TOOL_SCHEMAS["update_panel_projection"]}, {"name": "delete_panel_projection", **OUTPUT_JOB_TOOL_SCHEMAS["delete_panel_projection"]},
    {"name": "create_manufacturing_plan", **OUTPUT_JOB_TOOL_SCHEMAS["create_manufacturing_plan"]}, {"name": "update_manufacturing_plan", **OUTPUT_JOB_TOOL_SCHEMAS["update_manufacturing_plan"]}, {"name": "delete_manufacturing_plan", **OUTPUT_JOB_TOOL_SCHEMAS["delete_manufacturing_plan"]},
    {"name": "create_gerber_output_job", **OUTPUT_JOB_TOOL_SCHEMAS["create_gerber_output_job"]}, {"name": "create_output_job", **OUTPUT_JOB_TOOL_SCHEMAS["create_output_job"]}, {"name": "update_output_job", **OUTPUT_JOB_TOOL_SCHEMAS["update_output_job"]}, {"name": "run_output_job", **OUTPUT_JOB_TOOL_SCHEMAS["run_output_job"]}, {"name": "delete_output_job", **OUTPUT_JOB_TOOL_SCHEMAS["delete_output_job"]},
    {"name": "get_component_instances", **RELATIONSHIP_TOOL_SCHEMAS["get_component_instances"]}, {"name": "bind_component_instance", **RELATIONSHIP_TOOL_SCHEMAS["bind_component_instance"]}, {"name": "set_component_instance", **RELATIONSHIP_TOOL_SCHEMAS["set_component_instance"]}, {"name": "delete_component_instance", **RELATIONSHIP_TOOL_SCHEMAS["delete_component_instance"]},
    {"name": "get_relationships", **RELATIONSHIP_TOOL_SCHEMAS["get_relationships"]}, {"name": "get_variants", **RELATIONSHIP_TOOL_SCHEMAS["get_variants"]}, {"name": "get_import_map", **IMPORT_MAP_TOOL_SCHEMAS["get_import_map"]},
    {"name": "get_components", "description": "Return the imported board component list for the open project.", "inputSchema": {"type": "object", "properties": {}}},
    {"name": "get_netlist", "description": "Return canonical netlist view for the open board or schematic project.", "inputSchema": {"type": "object", "properties": {}}},
    {"name": "get_schematic_net_info", "description": "Return the current imported schematic net list and connectivity counts.", "inputSchema": {"type": "object", "properties": {}}},
    {"name": "get_board_summary", "description": "Return the imported board summary for the open project.", "inputSchema": {"type": "object", "properties": {}}},
    {"name": "get_schematic_summary", "description": "Return the imported schematic summary for the open project.", "inputSchema": {"type": "object", "properties": {}}},
    {"name": "get_sheets", "description": "Return imported schematic sheets for the open project.", "inputSchema": {"type": "object", "properties": {}}},
    {"name": "get_labels", "description": "Return the imported schematic labels for the open project.", "inputSchema": {"type": "object", "properties": {}}},
    {"name": "get_symbols", "description": "Return the imported schematic symbols for the open project.", "inputSchema": {"type": "object", "properties": {}}},
    {"name": "get_symbol_fields", "description": "Return authored fields for a specific schematic symbol UUID.", "inputSchema": {"type": "object", "properties": {"symbol_uuid": {"type": "string"}}, "required": ["symbol_uuid"]}},
    {"name": "get_ports", "description": "Return the imported schematic interface ports for the open project.", "inputSchema": {"type": "object", "properties": {}}},
    {"name": "get_buses", "description": "Return the imported schematic buses for the open project.", "inputSchema": {"type": "object", "properties": {}}},
    {"name": "get_bus_entries", "description": "Return the imported schematic bus-entry objects for the open project.", "inputSchema": {"type": "object", "properties": {}}},
    {"name": "get_noconnects", "description": "Return the imported schematic no-connect markers for the open project.", "inputSchema": {"type": "object", "properties": {}}},
    {"name": "get_connectivity_diagnostics", "description": "Return raw connectivity diagnostics for the open project.", "inputSchema": {"type": "object", "properties": {}}},
    {"name": "get_design_rules", "description": "Return authored design-rule entries for the open board project.", "inputSchema": {"type": "object", "properties": {}}},
    {"name": "create_proposal", **PROPOSAL_TOOL_SCHEMAS["create_proposal"]}, {"name": "create_draw_wire_proposal", **PROPOSAL_TOOL_SCHEMAS["create_draw_wire_proposal"]}, {"name": "create_place_label_proposal", **PROPOSAL_TOOL_SCHEMAS["create_place_label_proposal"]}, {"name": "create_place_symbol_proposal", **PROPOSAL_TOOL_SCHEMAS["create_place_symbol_proposal"]}, {"name": "create_board_component_replacement_proposal", **PROPOSAL_TOOL_SCHEMAS["create_board_component_replacement_proposal"]}, {"name": "create_board_component_replacements_proposal", **PROPOSAL_TOOL_SCHEMAS["create_board_component_replacements_proposal"]}, {"name": "create_board_component_replacement_plan_proposal", **PROPOSAL_TOOL_SCHEMAS["create_board_component_replacement_plan_proposal"]}, {"name": "get_proposals", **PROPOSAL_TOOL_SCHEMAS["get_proposals"]}, {"name": "show_proposal", **PROPOSAL_TOOL_SCHEMAS["show_proposal"]}, {"name": "preview_proposal", **PROPOSAL_TOOL_SCHEMAS["preview_proposal"]}, {"name": "validate_proposal", **PROPOSAL_TOOL_SCHEMAS["validate_proposal"]}, {"name": "defer_proposal", **PROPOSAL_TOOL_SCHEMAS["defer_proposal"]}, {"name": "reject_proposal", **PROPOSAL_TOOL_SCHEMAS["reject_proposal"]}, {"name": "review_proposal", **PROPOSAL_TOOL_SCHEMAS["review_proposal"]}, {"name": "accept_apply_proposal", **PROPOSAL_TOOL_SCHEMAS["accept_apply_proposal"]}, {"name": "apply_proposal", **PROPOSAL_TOOL_SCHEMAS["apply_proposal"]},
    {"name": "get_pool_library_objects", **LIBRARY_TOOL_SCHEMAS["get_pool_library_objects"]}, {"name": "show_pool_library_object", **LIBRARY_TOOL_SCHEMAS["show_pool_library_object"]}, {"name": "get_pool_model_blobs", **LIBRARY_TOOL_SCHEMAS["get_pool_model_blobs"]},
    {"name": "gc_pool_model_blobs", **LIBRARY_TOOL_SCHEMAS["gc_pool_model_blobs"]},
    {"name": "create_pool_library_object", **LIBRARY_TOOL_SCHEMAS["create_pool_library_object"]}, {"name": "delete_pool_library_object", **LIBRARY_TOOL_SCHEMAS["delete_pool_library_object"]}, {"name": "create_pool_unit", **LIBRARY_TOOL_SCHEMAS["create_pool_unit"]}, {"name": "set_pool_unit_pin", **LIBRARY_TOOL_SCHEMAS["set_pool_unit_pin"]}, {"name": "create_pool_symbol", **LIBRARY_TOOL_SCHEMAS["create_pool_symbol"]}, {"name": "add_pool_symbol_line", **LIBRARY_TOOL_SCHEMAS["add_pool_symbol_line"]}, {"name": "add_pool_symbol_rect", **LIBRARY_TOOL_SCHEMAS["add_pool_symbol_rect"]}, {"name": "add_pool_symbol_circle", **LIBRARY_TOOL_SCHEMAS["add_pool_symbol_circle"]}, {"name": "add_pool_symbol_arc", **LIBRARY_TOOL_SCHEMAS["add_pool_symbol_arc"]}, {"name": "add_pool_symbol_polygon", **LIBRARY_TOOL_SCHEMAS["add_pool_symbol_polygon"]}, {"name": "add_pool_symbol_text", **LIBRARY_TOOL_SCHEMAS["add_pool_symbol_text"]}, {"name": "set_pool_symbol_pin_anchor", **LIBRARY_TOOL_SCHEMAS["set_pool_symbol_pin_anchor"]},
    {"name": "create_pool_entity", **LIBRARY_TOOL_SCHEMAS["create_pool_entity"]}, {"name": "create_pool_padstack", **LIBRARY_TOOL_SCHEMAS["create_pool_padstack"]}, {"name": "create_pool_package", **LIBRARY_TOOL_SCHEMAS["create_pool_package"]}, {"name": "set_pool_package_pad", **LIBRARY_TOOL_SCHEMAS["set_pool_package_pad"]}, {"name": "set_pool_package_courtyard_rect", **LIBRARY_TOOL_SCHEMAS["set_pool_package_courtyard_rect"]}, {"name": "set_pool_package_courtyard_polygon", **LIBRARY_TOOL_SCHEMAS["set_pool_package_courtyard_polygon"]}, {"name": "add_pool_package_silkscreen_line", **LIBRARY_TOOL_SCHEMAS["add_pool_package_silkscreen_line"]}, {"name": "add_pool_package_silkscreen_rect", **LIBRARY_TOOL_SCHEMAS["add_pool_package_silkscreen_rect"]}, {"name": "add_pool_package_silkscreen_polygon", **LIBRARY_TOOL_SCHEMAS["add_pool_package_silkscreen_polygon"]}, {"name": "add_pool_package_silkscreen_circle", **LIBRARY_TOOL_SCHEMAS["add_pool_package_silkscreen_circle"]}, {"name": "add_pool_package_silkscreen_arc", **LIBRARY_TOOL_SCHEMAS["add_pool_package_silkscreen_arc"]}, {"name": "add_pool_package_silkscreen_text", **LIBRARY_TOOL_SCHEMAS["add_pool_package_silkscreen_text"]}, {"name": "add_pool_package_model_3d", **LIBRARY_TOOL_SCHEMAS["add_pool_package_model_3d"]}, {"name": "set_pool_package_body_heights", **LIBRARY_TOOL_SCHEMAS["set_pool_package_body_heights"]},
    {"name": "create_pool_part", **LIBRARY_TOOL_SCHEMAS["create_pool_part"]}, {"name": "set_pool_part_metadata", **LIBRARY_TOOL_SCHEMAS["set_pool_part_metadata"]}, {"name": "set_pool_part_parametric", **LIBRARY_TOOL_SCHEMAS["set_pool_part_parametric"]}, {"name": "set_pool_part_orderable_mpns", **LIBRARY_TOOL_SCHEMAS["set_pool_part_orderable_mpns"]}, {"name": "set_pool_part_tags", **LIBRARY_TOOL_SCHEMAS["set_pool_part_tags"]}, {"name": "set_pool_part_packaging_options", **LIBRARY_TOOL_SCHEMAS["set_pool_part_packaging_options"]}, {"name": "set_pool_part_supply_chain", **LIBRARY_TOOL_SCHEMAS["set_pool_part_supply_chain"]}, {"name": "set_pool_part_behavioural_models", **LIBRARY_TOOL_SCHEMAS["set_pool_part_behavioural_models"]}, {"name": "attach_pool_part_model", **LIBRARY_TOOL_SCHEMAS["attach_pool_part_model"]}, {"name": "detach_pool_part_model", **LIBRARY_TOOL_SCHEMAS["detach_pool_part_model"]}, {"name": "set_pool_part_thermal", **LIBRARY_TOOL_SCHEMAS["set_pool_part_thermal"]}, {"name": "set_pool_part_pad_map_entry", **LIBRARY_TOOL_SCHEMAS["set_pool_part_pad_map_entry"]}, {"name": "set_pool_part_pad_map", **LIBRARY_TOOL_SCHEMAS["set_pool_part_pad_map"]}, {"name": "set_pool_library_object", **LIBRARY_TOOL_SCHEMAS["set_pool_library_object"]},
]
_LEGACY_FLAT_TOOL_SPECS: list[dict[str, object]] = [
    {
        "name": "open_project",
        "description": "Import a KiCad or Eagle design into the engine session.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}},
            "required": ["path"],
        },
    },
    {"name": "close_project", "description": "Close the current in-memory project session.", "inputSchema": {"type": "object", "properties": {}}},
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
        "name": "get_hierarchy",
        "description": "Return the imported schematic hierarchy for the open project.",
        "inputSchema": {"type": "object", "properties": {}},
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
        "name": "review_route_proposal",
        "description": "Review one selected deterministic route proposal or one saved route proposal artifact without mutating project state.",
        "x_dispatch_args": [
            "path",
            "net_uuid",
            "from_anchor_pad_uuid",
            "to_anchor_pad_uuid",
            "profile",
            "artifact",
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
                "artifact": {"type": "string"},
            },
            "oneOf": [
                {
                    "required": [
                        "path",
                        "net_uuid",
                        "from_anchor_pad_uuid",
                        "to_anchor_pad_uuid",
                    ]
                },
                {"required": ["artifact"]},
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
        "name": "write_route_strategy_curated_fixture_suite", "authoring_boundary": "generated_fixture_only", "write_path_policy": "direct project-shard writes are restricted to deterministic regression fixture generation",
        "description": "Write one deterministic generated-regression-fixture native-project suite plus a versioned batch request manifest for repeated route-strategy batch evidence runs; not a user design-authoring path.",
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
        "name": "capture_route_strategy_curated_baseline", "authoring_boundary": "generated_fixture_only", "write_path_policy": "direct project-shard writes are restricted to deterministic regression fixture generation",
        "description": "Materialize the generated-regression-fixture route-strategy suite, evaluate it, and save one reusable versioned batch-result baseline artifact; not a user design-authoring path.",
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
        "x_public_write_surface_class": "journaled_route_apply",
        "x_write_surface_evidence": "routes through route-apply proposal/journal gateway",
        "description": "Apply one accepted deterministic route candidate through the proposal journal gateway.",
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
        "x_public_write_surface_class": "journaled_route_apply",
        "x_write_surface_evidence": "routes through route-apply proposal/journal gateway",
        "description": "Apply the currently selected deterministic route proposal through the proposal journal gateway.",
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
        "x_public_write_surface_class": "proposal_artifact_apply",
        "x_write_surface_evidence": "applies embedded route proposal artifact through substrate apply path",
        "description": "Apply one native route proposal artifact through the proposal journal gateway when it still matches the current live project state.",
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
# _LEGACY_CANONICAL_ALIAS_NAMES is imported from tools_catalog_legacy_aliases.py
# (extracted to keep this module within its file-size budget).


def _legacy_canonical_alias(flat_spec: dict[str, Any]) -> dict[str, Any]:
    flat_name = flat_spec["name"]
    alias = {key: value for key, value in flat_spec.items() if key != "name"}
    alias["name"] = _LEGACY_CANONICAL_ALIAS_NAMES[flat_name]
    alias["x_dispatch_method"] = flat_name
    return alias


# Non-journaled daemon mutation arms (engine-daemon/src/dispatch.rs): these flat
# methods write the legacy in-memory api::Engine without the substrate commit()/
# journal path. Per decision 004 (Private Mutation Ban) no *public* MCP write tool
# may dispatch to one of these; each has a substrate-backed (journaled) public
# equivalent in the datum.pcb.* family, so the canonical alias bridging to a
# non-journaled arm stays dispatchable for compatibility but hidden from tools/list.
NON_JOURNALED_DAEMON_WRITE_METHODS: frozenset[str] = frozenset({})
def _alias_dispatches_to_non_journaled_write(alias_spec: dict[str, Any]) -> bool:
    return alias_spec.get("x_dispatch_method") in NON_JOURNALED_DAEMON_WRITE_METHODS


_LEGACY_CANONICAL_ALIAS_SPECS: list[dict[str, object]] = [
    _legacy_canonical_alias(spec) for spec in _LEGACY_FLAT_TOOL_SPECS
]

# Board-write aliases bridging to non-journaled arms stay dispatchable but are not
# advertised publicly; the journaled datum.pcb.* family is the only public board
# write surface.
_PUBLIC_CANONICAL_ALIAS_SPECS = [s for s in _LEGACY_CANONICAL_ALIAS_SPECS if not _alias_dispatches_to_non_journaled_write(s)]
_HIDDEN_CANONICAL_ALIAS_SPECS = [s for s in _LEGACY_CANONICAL_ALIAS_SPECS if _alias_dispatches_to_non_journaled_write(s)]

# Public catalog: canonical datum.* aliases that do not bypass the journal.
TOOL_SPECS: list[dict[str, object]] = DATUM_TOOL_SPECS + _PUBLIC_CANONICAL_ALIAS_SPECS
# Legacy flat tools + bypassing board-write aliases: dispatchable, hidden from list.
COMPATIBILITY_TOOL_SPECS += _LEGACY_FLAT_TOOL_SPECS + _HIDDEN_CANONICAL_ALIAS_SPECS


def _with_retirement_metadata(spec: dict[str, object]) -> dict[str, object]:
    annotated = dict(spec)
    public_replacements_by_dispatch = {
        str(public_spec.get("x_dispatch_method")): str(public_spec["name"])
        for public_spec in TOOL_SPECS
        if public_spec.get("x_dispatch_method")
    }
    replacements = canonical_replacements_for_hidden_alias(
        annotated,
        public_replacements_by_dispatch,
    )
    retirement_override = retirement_override_for_hidden_alias(annotated)
    annotated.setdefault("x_compatibility_visibility", "hidden")
    annotated.setdefault("x_canonical_replacements", replacements)
    annotated.setdefault("x_retirement_status", retirement_override.get("status", "retained_until_migration_plan"))
    annotated.setdefault(
        "x_retirement_criteria",
        retirement_override.get("criteria", DEFAULT_HIDDEN_ALIAS_RETIREMENT_CRITERIA),
    )
    return annotated


COMPATIBILITY_TOOL_SPECS = [
    _with_retirement_metadata(spec) for spec in COMPATIBILITY_TOOL_SPECS
]


def _catalog_tool(spec: dict[str, Any]) -> dict[str, Any]:
    return {key: value for key, value in spec.items() if not key.startswith("x_")}
TOOLS: list[dict[str, Any]] = [_catalog_tool(spec) for spec in TOOL_SPECS]
TOOL_BY_NAME: dict[str, dict[str, Any]] = {spec["name"]: spec for spec in [*TOOL_SPECS, *COMPATIBILITY_TOOL_SPECS]}
