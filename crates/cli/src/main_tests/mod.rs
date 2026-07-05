use super::*;

fn eagle_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/eagle")
        .join(name)
}

fn kicad_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/kicad")
        .join(name)
}

fn execute(cli: Cli) -> Result<String> {
    let (output, _) = execute_with_exit_code(cli)?;
    Ok(output)
}

mod main_tests_context;
mod main_tests_context_active;
mod main_tests_context_check_context;
mod main_tests_context_source_shards;
mod main_tests_context_terminal_lifecycle;
mod main_tests_project_artifact_runs;
mod main_tests_project_bom_compare;
mod main_tests_project_bom_export;
mod main_tests_project_bom_inspect;
mod main_tests_project_bom_validate;
mod main_tests_project_drc;
mod main_tests_project_drill_compare;
mod main_tests_project_drill_export;
mod main_tests_project_drill_hole_classes;
mod main_tests_project_drill_inspect;
mod main_tests_project_drill_validate;
mod main_tests_project_excellon_drill_compare;
mod main_tests_project_excellon_drill_component_pads;
mod main_tests_project_excellon_drill_export;
mod main_tests_project_excellon_drill_inspect;
mod main_tests_project_excellon_drill_validate;
mod main_tests_project_gerber_copper_compare;
mod main_tests_project_gerber_copper_export;
mod main_tests_project_gerber_copper_validate;
mod main_tests_project_gerber_inspect;
mod main_tests_project_gerber_mask_compare;
mod main_tests_project_gerber_mask_export;
mod main_tests_project_gerber_mask_validate;
mod main_tests_project_gerber_mech_compare;
mod main_tests_project_gerber_mech_component;
mod main_tests_project_gerber_mech_component_arc;
mod main_tests_project_gerber_mech_component_text;
mod main_tests_project_gerber_mech_export;
mod main_tests_project_gerber_mech_text;
mod main_tests_project_gerber_mech_validate;
mod main_tests_project_gerber_outline_compare;
mod main_tests_project_gerber_outline_export;
mod main_tests_project_gerber_outline_validate;
mod main_tests_project_gerber_paste_compare;
mod main_tests_project_gerber_paste_export;
mod main_tests_project_gerber_paste_validate;
mod main_tests_project_gerber_plan;
mod main_tests_project_gerber_plan_compare;
mod main_tests_project_gerber_set;
mod main_tests_project_gerber_set_compare;
mod main_tests_project_gerber_silk_compare;
mod main_tests_project_gerber_silk_component_arc;
mod main_tests_project_gerber_silk_component_circle;
mod main_tests_project_gerber_silk_component_polygon;
mod main_tests_project_gerber_silk_component_polyline;
mod main_tests_project_gerber_silk_export;
mod main_tests_project_gerber_silk_validate;
mod main_tests_project_manufacturing;
mod main_tests_project_manufacturing_compare;
mod main_tests_project_manufacturing_export;
mod main_tests_project_manufacturing_inspect;
mod main_tests_project_manufacturing_manifest;
mod main_tests_project_manufacturing_plan_proposal_validation;
mod main_tests_project_manufacturing_plan_proposals;
mod main_tests_project_manufacturing_projection_parity;
mod main_tests_project_manufacturing_validate;
mod main_tests_project_output_job_include;
mod main_tests_project_output_job_proposal_validation;
mod main_tests_project_output_job_run;
mod main_tests_project_output_job_run_replay;
mod main_tests_project_output_job_update;
mod main_tests_project_output_job_variant;
mod main_tests_project_panel_projection;
mod main_tests_project_panel_projection_output_job;
mod main_tests_project_pnp_compare;
mod main_tests_project_pnp_export;
mod main_tests_project_pnp_inspect;
mod main_tests_project_pnp_validate;

#[allow(clippy::too_many_arguments)]
fn modify_board(
    path: &Path,
    delete_track: &[Uuid],
    delete_via: &[Uuid],
    delete_component: &[Uuid],
    libraries: &[PathBuf],
    move_component: &[MoveComponentInput],
    rotate_component: &[RotateComponentInput],
    set_value: &[SetValueInput],
    assign_part: &[AssignPartInput],
    set_package: &[SetPackageInput],
    set_package_with_part: &[SetPackageWithPartInput],
    replace_component: &[ReplaceComponentInput],
    set_net_class: &[SetNetClassInput],
    set_reference: &[SetReferenceInput],
    set_clearance_min_nm: Option<i64>,
    undo: usize,
    redo: usize,
    save: Option<&Path>,
    save_original: bool,
) -> Result<ModifyReportView> {
    modify_board_with_plan(
        path,
        delete_track,
        delete_via,
        delete_component,
        libraries,
        move_component,
        rotate_component,
        set_value,
        assign_part,
        set_package,
        set_package_with_part,
        replace_component,
        set_net_class,
        set_reference,
        set_clearance_min_nm,
        undo,
        redo,
        save,
        save_original,
        &[],
        &[],
        &[],
        &[],
    )
}

mod main_tests_check;
mod main_tests_check_goldens;
mod main_tests_import_plan;
mod main_tests_modify_advanced;
mod main_tests_modify_advanced_plan;
mod main_tests_modify_basic;
mod main_tests_plan_apply;
mod main_tests_plan_apply_compare_artifact;
mod main_tests_plan_apply_inspect_artifact;
mod main_tests_plan_apply_legacy_manifest;
mod main_tests_plan_apply_legacy_manifest_text;
mod main_tests_plan_apply_text;
mod main_tests_plan_apply_validate;
mod main_tests_plan_apply_validate_artifact;
mod main_tests_project_board_check;
mod main_tests_project_board_component;
mod main_tests_project_board_component_mechanical;
mod main_tests_project_board_component_models_3d;
mod main_tests_project_board_component_pads;
mod main_tests_project_board_component_pool_materialization;
mod main_tests_project_board_component_query;
mod main_tests_project_board_component_query_single;
mod main_tests_project_board_component_replacement_proposal;
mod main_tests_project_board_component_silkscreen;
mod main_tests_project_board_connectivity;
mod main_tests_project_board_dimension;
mod main_tests_project_board_handoff;
mod main_tests_project_board_keepout;
mod main_tests_project_board_net;
mod main_tests_project_board_net_class;
mod main_tests_project_board_outline;
mod main_tests_project_board_pad;
mod main_tests_project_board_stackup;
mod main_tests_project_board_text;
mod main_tests_project_board_track;
mod main_tests_project_board_via;
mod main_tests_project_board_zone;
mod main_tests_project_board_zone_drc;
mod main_tests_project_board_zone_fill_solver;
mod main_tests_project_bus;
mod main_tests_project_check;
mod main_tests_project_check_profiles;
mod main_tests_project_check_run;
mod main_tests_project_check_run_drc_lifecycle;
mod main_tests_project_check_run_history;
mod main_tests_project_connectivity;
mod main_tests_project_drawing;
mod main_tests_project_erc;
mod main_tests_project_forward_annotation;
mod main_tests_project_forward_annotation_apply_explicit_input;
mod main_tests_project_forward_annotation_apply_part_mismatch;
mod main_tests_project_forward_annotation_apply_single_action;
mod main_tests_project_forward_annotation_artifact_apply;
mod main_tests_project_forward_annotation_artifact_compare;
mod main_tests_project_forward_annotation_artifact_filter;
mod main_tests_project_forward_annotation_artifact_plan;
mod main_tests_project_forward_annotation_artifact_review_import;
mod main_tests_project_forward_annotation_artifact_review_replace;
mod main_tests_project_forward_annotation_artifact_validate;
mod main_tests_project_forward_annotation_batch_apply;
mod main_tests_project_forward_annotation_export_inspect;
mod main_tests_project_forward_annotation_export_selection;
mod main_tests_project_forward_annotation_proposal;
mod main_tests_project_forward_annotation_review;
mod main_tests_project_forward_annotation_support;
mod main_tests_project_hierarchy;
mod main_tests_project_import_map;
mod main_tests_project_imports;
mod main_tests_project_imports_eagle;
mod main_tests_project_imports_lifecycle;
mod main_tests_project_imports_schematic;
mod main_tests_project_inspect;
mod main_tests_project_journal;
mod main_tests_project_journal_guards;
mod main_tests_project_journal_support;
mod main_tests_project_junction;
mod main_tests_project_label;
mod main_tests_project_new;
mod main_tests_project_noconnect;
mod main_tests_project_pool_footprint;
mod main_tests_project_pool_footprint_proposals;
mod main_tests_project_pool_footprint_silkscreen;
mod main_tests_project_pool_footprint_silkscreen_proposals;
mod main_tests_project_pool_library;
mod main_tests_project_pool_library_proposals;
mod main_tests_project_pool_package_courtyard;
mod main_tests_project_pool_package_geometry;
mod main_tests_project_pool_package_proposals;
mod main_tests_project_pool_part;
mod main_tests_project_pool_part_pin_semantics;
mod main_tests_project_pool_pin_pad_map;
mod main_tests_project_pool_resolver;
mod main_tests_project_pool_symbol_library;
mod main_tests_project_pool_symbol_pin_anchor;
mod main_tests_project_port;
mod main_tests_project_production_idempotency;
mod main_tests_project_proposal_policy;
mod main_tests_project_proposals;
mod main_tests_project_query;
mod main_tests_project_query_canonical;
mod main_tests_project_relationship_diagnostics;
mod main_tests_project_relationships;
mod main_tests_project_repo_validation;
mod main_tests_project_resolve_debug_guards;
mod main_tests_project_resolver;
mod main_tests_project_route_apply;
mod main_tests_project_route_apply_taxonomy;
mod main_tests_project_route_corridor;
mod main_tests_project_route_path_candidate;
mod main_tests_project_route_path_candidate_authored_copper_graph;
mod main_tests_project_route_path_candidate_authored_copper_graph_explain;
mod main_tests_project_route_path_candidate_authored_copper_graph_obstacle_aware;
mod main_tests_project_route_path_candidate_authored_copper_graph_obstacle_aware_explain;
mod main_tests_project_route_path_candidate_authored_copper_graph_zone_aware;
mod main_tests_project_route_path_candidate_authored_copper_graph_zone_aware_explain;
mod main_tests_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware;
mod main_tests_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_explain;
mod main_tests_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware;
mod main_tests_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_explain;
mod main_tests_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware;
mod main_tests_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware_explain;
mod main_tests_project_route_path_candidate_authored_copper_plus_one_gap;
mod main_tests_project_route_path_candidate_authored_copper_plus_one_gap_explain;
mod main_tests_project_route_path_candidate_authored_via_chain;
mod main_tests_project_route_path_candidate_authored_via_chain_explain;
mod main_tests_project_route_path_candidate_explain;
mod main_tests_project_route_path_candidate_explain_generic;
mod main_tests_project_route_path_candidate_five_via;
mod main_tests_project_route_path_candidate_five_via_explain;
mod main_tests_project_route_path_candidate_four_via;
mod main_tests_project_route_path_candidate_four_via_explain;
mod main_tests_project_route_path_candidate_generic;
mod main_tests_project_route_path_candidate_six_via;
mod main_tests_project_route_path_candidate_six_via_explain;
mod main_tests_project_route_path_candidate_three_via;
mod main_tests_project_route_path_candidate_three_via_explain;
mod main_tests_project_route_path_candidate_two_via;
mod main_tests_project_route_path_candidate_two_via_explain;
mod main_tests_project_route_path_candidate_via;
mod main_tests_project_route_path_candidate_via_explain;
mod main_tests_project_route_preflight;
mod main_tests_project_route_proposal_artifact;
mod main_tests_project_route_proposal_explain;
mod main_tests_project_route_proposal_policy;
mod main_tests_project_route_proposal_review;
mod main_tests_project_route_proposal_selection;
mod main_tests_project_route_strategy_batch_evaluate;
mod main_tests_project_route_strategy_batch_result;
mod main_tests_project_route_strategy_compare;
mod main_tests_project_route_strategy_delta;
mod main_tests_project_route_strategy_fixture_suite;
mod main_tests_project_route_strategy_repo_baseline;
mod main_tests_project_route_strategy_report;
mod main_tests_project_routing_substrate;
mod main_tests_project_rules;
mod main_tests_project_schematic_proposals;
mod main_tests_project_selected_route_proposal;
mod main_tests_project_sheet;
mod main_tests_project_source_shard_reporting;
mod main_tests_project_source_shard_root_replay;
mod main_tests_project_source_shard_unknown;
mod main_tests_project_source_shard_unknown_generated;
mod main_tests_project_source_shard_unknown_schematic;
mod main_tests_project_source_shard_unknown_sidecar;
mod main_tests_project_standards_clearance_repairs;
mod main_tests_project_standards_peer_aperture_repairs;
mod main_tests_project_standards_repair_apply;
mod main_tests_project_standards_repairs;
mod main_tests_project_standards_silk_repairs;
mod main_tests_project_standards_zone_fill_repairs;
mod main_tests_project_symbol;
mod main_tests_project_symbol_binding;
mod main_tests_project_symbol_library_materialization;
mod main_tests_project_symbol_resolution;
mod main_tests_project_symbol_semantics;
mod main_tests_project_text;
mod main_tests_project_validate;
mod main_tests_project_validate_pool_defaults;
mod main_tests_project_validate_pool_refs;
mod main_tests_project_validate_resolver;
mod main_tests_project_validate_rules;
mod main_tests_project_wire;
mod main_tests_query_goldens;
mod main_tests_query_surface;
