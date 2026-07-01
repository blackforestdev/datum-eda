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

#[path = "main_tests_context.rs"]
mod main_tests_context;
#[path = "main_tests_context_active.rs"]
mod main_tests_context_active;
#[path = "main_tests_context_check_context.rs"]
mod main_tests_context_check_context;
#[path = "main_tests_context_source_shards.rs"]
mod main_tests_context_source_shards;
#[path = "main_tests_context_terminal_lifecycle.rs"]
mod main_tests_context_terminal_lifecycle;
#[path = "main_tests_project_artifact_runs.rs"]
mod main_tests_project_artifact_runs;
#[path = "main_tests_project_bom_compare.rs"]
mod main_tests_project_bom_compare;
#[path = "main_tests_project_bom_export.rs"]
mod main_tests_project_bom_export;
#[path = "main_tests_project_bom_inspect.rs"]
mod main_tests_project_bom_inspect;
#[path = "main_tests_project_bom_validate.rs"]
mod main_tests_project_bom_validate;
#[path = "main_tests_project_drc.rs"]
mod main_tests_project_drc;
#[path = "main_tests_project_drill_compare.rs"]
mod main_tests_project_drill_compare;
#[path = "main_tests_project_drill_export.rs"]
mod main_tests_project_drill_export;
#[path = "main_tests_project_drill_hole_classes.rs"]
mod main_tests_project_drill_hole_classes;
#[path = "main_tests_project_drill_inspect.rs"]
mod main_tests_project_drill_inspect;
#[path = "main_tests_project_drill_validate.rs"]
mod main_tests_project_drill_validate;
#[path = "main_tests_project_excellon_drill_compare.rs"]
mod main_tests_project_excellon_drill_compare;
#[path = "main_tests_project_excellon_drill_component_pads.rs"]
mod main_tests_project_excellon_drill_component_pads;
#[path = "main_tests_project_excellon_drill_export.rs"]
mod main_tests_project_excellon_drill_export;
#[path = "main_tests_project_excellon_drill_inspect.rs"]
mod main_tests_project_excellon_drill_inspect;
#[path = "main_tests_project_excellon_drill_validate.rs"]
mod main_tests_project_excellon_drill_validate;
#[path = "main_tests_project_gerber_copper_compare.rs"]
mod main_tests_project_gerber_copper_compare;
#[path = "main_tests_project_gerber_copper_export.rs"]
mod main_tests_project_gerber_copper_export;
#[path = "main_tests_project_gerber_copper_validate.rs"]
mod main_tests_project_gerber_copper_validate;
#[path = "main_tests_project_gerber_inspect.rs"]
mod main_tests_project_gerber_inspect;
#[path = "main_tests_project_gerber_mask_compare.rs"]
mod main_tests_project_gerber_mask_compare;
#[path = "main_tests_project_gerber_mask_export.rs"]
mod main_tests_project_gerber_mask_export;
#[path = "main_tests_project_gerber_mask_validate.rs"]
mod main_tests_project_gerber_mask_validate;
#[path = "main_tests_project_gerber_mech_compare.rs"]
mod main_tests_project_gerber_mech_compare;
#[path = "main_tests_project_gerber_mech_component.rs"]
mod main_tests_project_gerber_mech_component;
#[path = "main_tests_project_gerber_mech_component_arc.rs"]
mod main_tests_project_gerber_mech_component_arc;
#[path = "main_tests_project_gerber_mech_component_text.rs"]
mod main_tests_project_gerber_mech_component_text;
#[path = "main_tests_project_gerber_mech_export.rs"]
mod main_tests_project_gerber_mech_export;
#[path = "main_tests_project_gerber_mech_text.rs"]
mod main_tests_project_gerber_mech_text;
#[path = "main_tests_project_gerber_mech_validate.rs"]
mod main_tests_project_gerber_mech_validate;
#[path = "main_tests_project_gerber_outline_compare.rs"]
mod main_tests_project_gerber_outline_compare;
#[path = "main_tests_project_gerber_outline_export.rs"]
mod main_tests_project_gerber_outline_export;
#[path = "main_tests_project_gerber_outline_validate.rs"]
mod main_tests_project_gerber_outline_validate;
#[path = "main_tests_project_gerber_paste_compare.rs"]
mod main_tests_project_gerber_paste_compare;
#[path = "main_tests_project_gerber_paste_export.rs"]
mod main_tests_project_gerber_paste_export;
#[path = "main_tests_project_gerber_paste_validate.rs"]
mod main_tests_project_gerber_paste_validate;
#[path = "main_tests_project_gerber_plan.rs"]
mod main_tests_project_gerber_plan;
#[path = "main_tests_project_gerber_plan_compare.rs"]
mod main_tests_project_gerber_plan_compare;
#[path = "main_tests_project_gerber_set.rs"]
mod main_tests_project_gerber_set;
#[path = "main_tests_project_gerber_set_compare.rs"]
mod main_tests_project_gerber_set_compare;
#[path = "main_tests_project_gerber_silk_compare.rs"]
mod main_tests_project_gerber_silk_compare;
#[path = "main_tests_project_gerber_silk_component_arc.rs"]
mod main_tests_project_gerber_silk_component_arc;
#[path = "main_tests_project_gerber_silk_component_circle.rs"]
mod main_tests_project_gerber_silk_component_circle;
#[path = "main_tests_project_gerber_silk_component_polygon.rs"]
mod main_tests_project_gerber_silk_component_polygon;
#[path = "main_tests_project_gerber_silk_component_polyline.rs"]
mod main_tests_project_gerber_silk_component_polyline;
#[path = "main_tests_project_gerber_silk_export.rs"]
mod main_tests_project_gerber_silk_export;
#[path = "main_tests_project_gerber_silk_validate.rs"]
mod main_tests_project_gerber_silk_validate;
#[path = "main_tests_project_manufacturing.rs"]
mod main_tests_project_manufacturing;
#[path = "main_tests_project_manufacturing_compare.rs"]
mod main_tests_project_manufacturing_compare;
#[path = "main_tests_project_manufacturing_export.rs"]
mod main_tests_project_manufacturing_export;
#[path = "main_tests_project_manufacturing_inspect.rs"]
mod main_tests_project_manufacturing_inspect;
#[path = "main_tests_project_manufacturing_manifest.rs"]
mod main_tests_project_manufacturing_manifest;
#[path = "main_tests_project_manufacturing_plan_proposal_validation.rs"]
mod main_tests_project_manufacturing_plan_proposal_validation;
#[path = "main_tests_project_manufacturing_plan_proposals.rs"]
mod main_tests_project_manufacturing_plan_proposals;
#[path = "main_tests_project_manufacturing_projection_parity.rs"]
mod main_tests_project_manufacturing_projection_parity;
#[path = "main_tests_project_manufacturing_validate.rs"]
mod main_tests_project_manufacturing_validate;
#[path = "main_tests_project_output_job_include.rs"]
mod main_tests_project_output_job_include;
#[path = "main_tests_project_output_job_proposal_validation.rs"]
mod main_tests_project_output_job_proposal_validation;
#[path = "main_tests_project_output_job_run.rs"]
mod main_tests_project_output_job_run;
#[path = "main_tests_project_output_job_run_replay.rs"]
mod main_tests_project_output_job_run_replay;
#[path = "main_tests_project_output_job_update.rs"]
mod main_tests_project_output_job_update;
#[path = "main_tests_project_output_job_variant.rs"]
mod main_tests_project_output_job_variant;
#[path = "main_tests_project_panel_projection.rs"]
mod main_tests_project_panel_projection;
#[path = "main_tests_project_panel_projection_output_job.rs"]
mod main_tests_project_panel_projection_output_job;
#[path = "main_tests_project_pnp_compare.rs"]
mod main_tests_project_pnp_compare;
#[path = "main_tests_project_pnp_export.rs"]
mod main_tests_project_pnp_export;
#[path = "main_tests_project_pnp_inspect.rs"]
mod main_tests_project_pnp_inspect;
#[path = "main_tests_project_pnp_validate.rs"]
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

#[path = "main_tests_check.rs"]
mod main_tests_check;
#[path = "main_tests_check_goldens.rs"]
mod main_tests_check_goldens;
#[path = "main_tests_import_plan.rs"]
mod main_tests_import_plan;
#[path = "main_tests_modify_advanced.rs"]
mod main_tests_modify_advanced;
#[path = "main_tests_modify_advanced_plan.rs"]
mod main_tests_modify_advanced_plan;
#[path = "main_tests_modify_basic.rs"]
mod main_tests_modify_basic;
#[path = "main_tests_plan_apply.rs"]
mod main_tests_plan_apply;
#[path = "main_tests_plan_apply_compare_artifact.rs"]
mod main_tests_plan_apply_compare_artifact;
#[path = "main_tests_plan_apply_inspect_artifact.rs"]
mod main_tests_plan_apply_inspect_artifact;
#[path = "main_tests_plan_apply_legacy_manifest.rs"]
mod main_tests_plan_apply_legacy_manifest;
#[path = "main_tests_plan_apply_legacy_manifest_text.rs"]
mod main_tests_plan_apply_legacy_manifest_text;
#[path = "main_tests_plan_apply_text.rs"]
mod main_tests_plan_apply_text;
#[path = "main_tests_plan_apply_validate.rs"]
mod main_tests_plan_apply_validate;
#[path = "main_tests_plan_apply_validate_artifact.rs"]
mod main_tests_plan_apply_validate_artifact;
#[path = "main_tests_project_board_check.rs"]
mod main_tests_project_board_check;
#[path = "main_tests_project_board_component.rs"]
mod main_tests_project_board_component;
#[path = "main_tests_project_board_component_mechanical.rs"]
mod main_tests_project_board_component_mechanical;
#[path = "main_tests_project_board_component_models_3d.rs"]
mod main_tests_project_board_component_models_3d;
#[path = "main_tests_project_board_component_pads.rs"]
mod main_tests_project_board_component_pads;
#[path = "main_tests_project_board_component_pool_materialization.rs"]
mod main_tests_project_board_component_pool_materialization;
#[path = "main_tests_project_board_component_pool_materialization_b.rs"]
mod main_tests_project_board_component_pool_materialization_b;
#[path = "main_tests_project_board_component_query.rs"]
mod main_tests_project_board_component_query;
#[path = "main_tests_project_board_component_query_single.rs"]
mod main_tests_project_board_component_query_single;
#[path = "main_tests_project_board_component_replacement_proposal.rs"]
mod main_tests_project_board_component_replacement_proposal;
#[path = "main_tests_project_board_component_silkscreen.rs"]
mod main_tests_project_board_component_silkscreen;
#[path = "main_tests_project_board_connectivity.rs"]
mod main_tests_project_board_connectivity;
#[path = "main_tests_project_board_dimension.rs"]
mod main_tests_project_board_dimension;
#[path = "main_tests_project_board_handoff.rs"]
mod main_tests_project_board_handoff;
#[path = "main_tests_project_board_keepout.rs"]
mod main_tests_project_board_keepout;
#[path = "main_tests_project_board_net.rs"]
mod main_tests_project_board_net;
#[path = "main_tests_project_board_net_class.rs"]
mod main_tests_project_board_net_class;
#[path = "main_tests_project_board_outline.rs"]
mod main_tests_project_board_outline;
#[path = "main_tests_project_board_pad.rs"]
mod main_tests_project_board_pad;
#[path = "main_tests_project_board_stackup.rs"]
mod main_tests_project_board_stackup;
#[path = "main_tests_project_board_text.rs"]
mod main_tests_project_board_text;
#[path = "main_tests_project_board_track.rs"]
mod main_tests_project_board_track;
#[path = "main_tests_project_board_via.rs"]
mod main_tests_project_board_via;
#[path = "main_tests_project_board_zone.rs"]
mod main_tests_project_board_zone;
#[path = "main_tests_project_board_zone_drc.rs"]
mod main_tests_project_board_zone_drc;
#[path = "main_tests_project_board_zone_fill_solver.rs"]
mod main_tests_project_board_zone_fill_solver;
#[path = "main_tests_project_bus.rs"]
mod main_tests_project_bus;
#[path = "main_tests_project_check.rs"]
mod main_tests_project_check;
#[path = "main_tests_project_check_profiles.rs"]
mod main_tests_project_check_profiles;
#[path = "main_tests_project_check_run.rs"]
mod main_tests_project_check_run;
#[path = "main_tests_project_check_run_drc_lifecycle.rs"]
mod main_tests_project_check_run_drc_lifecycle;
#[path = "main_tests_project_check_run_history.rs"]
mod main_tests_project_check_run_history;
#[path = "main_tests_project_connectivity.rs"]
mod main_tests_project_connectivity;
#[path = "main_tests_project_drawing.rs"]
mod main_tests_project_drawing;
#[path = "main_tests_project_erc.rs"]
mod main_tests_project_erc;
#[path = "main_tests_project_forward_annotation.rs"]
mod main_tests_project_forward_annotation;
#[path = "main_tests_project_forward_annotation_apply_explicit_input.rs"]
mod main_tests_project_forward_annotation_apply_explicit_input;
#[path = "main_tests_project_forward_annotation_apply_part_mismatch.rs"]
mod main_tests_project_forward_annotation_apply_part_mismatch;
#[path = "main_tests_project_forward_annotation_apply_single_action.rs"]
mod main_tests_project_forward_annotation_apply_single_action;
#[path = "main_tests_project_forward_annotation_artifact_apply.rs"]
mod main_tests_project_forward_annotation_artifact_apply;
#[path = "main_tests_project_forward_annotation_artifact_compare.rs"]
mod main_tests_project_forward_annotation_artifact_compare;
#[path = "main_tests_project_forward_annotation_artifact_filter.rs"]
mod main_tests_project_forward_annotation_artifact_filter;
#[path = "main_tests_project_forward_annotation_artifact_plan.rs"]
mod main_tests_project_forward_annotation_artifact_plan;
#[path = "main_tests_project_forward_annotation_artifact_review_import.rs"]
mod main_tests_project_forward_annotation_artifact_review_import;
#[path = "main_tests_project_forward_annotation_artifact_review_replace.rs"]
mod main_tests_project_forward_annotation_artifact_review_replace;
#[path = "main_tests_project_forward_annotation_artifact_validate.rs"]
mod main_tests_project_forward_annotation_artifact_validate;
#[path = "main_tests_project_forward_annotation_batch_apply.rs"]
mod main_tests_project_forward_annotation_batch_apply;
#[path = "main_tests_project_forward_annotation_export_inspect.rs"]
mod main_tests_project_forward_annotation_export_inspect;
#[path = "main_tests_project_forward_annotation_export_selection.rs"]
mod main_tests_project_forward_annotation_export_selection;
#[path = "main_tests_project_forward_annotation_proposal.rs"]
mod main_tests_project_forward_annotation_proposal;
#[path = "main_tests_project_forward_annotation_review.rs"]
mod main_tests_project_forward_annotation_review;
#[path = "main_tests_project_forward_annotation_support.rs"]
mod main_tests_project_forward_annotation_support;
#[path = "main_tests_project_hierarchy.rs"]
mod main_tests_project_hierarchy;
#[path = "main_tests_project_import_map.rs"]
mod main_tests_project_import_map;
#[path = "main_tests_project_imports.rs"]
mod main_tests_project_imports;
#[path = "main_tests_project_imports_eagle.rs"]
mod main_tests_project_imports_eagle;
#[path = "main_tests_project_imports_lifecycle.rs"]
mod main_tests_project_imports_lifecycle;
#[path = "main_tests_project_imports_schematic.rs"]
mod main_tests_project_imports_schematic;
#[path = "main_tests_project_inspect.rs"]
mod main_tests_project_inspect;
#[path = "main_tests_project_journal.rs"]
mod main_tests_project_journal;
#[path = "main_tests_project_journal_guards.rs"]
mod main_tests_project_journal_guards;
#[path = "main_tests_project_journal_support.rs"]
mod main_tests_project_journal_support;
#[path = "main_tests_project_junction.rs"]
mod main_tests_project_junction;
#[path = "main_tests_project_label.rs"]
mod main_tests_project_label;
#[path = "main_tests_project_new.rs"]
mod main_tests_project_new;
#[path = "main_tests_project_noconnect.rs"]
mod main_tests_project_noconnect;
#[path = "main_tests_project_pool_footprint.rs"]
mod main_tests_project_pool_footprint;
#[path = "main_tests_project_pool_footprint_proposals.rs"]
mod main_tests_project_pool_footprint_proposals;
#[path = "main_tests_project_pool_footprint_silkscreen.rs"]
mod main_tests_project_pool_footprint_silkscreen;
#[path = "main_tests_project_pool_footprint_silkscreen_proposals.rs"]
mod main_tests_project_pool_footprint_silkscreen_proposals;
#[path = "main_tests_project_pool_library.rs"]
mod main_tests_project_pool_library;
#[path = "main_tests_project_pool_library_b.rs"]
mod main_tests_project_pool_library_b;
#[path = "main_tests_project_pool_library_proposals.rs"]
mod main_tests_project_pool_library_proposals;
#[path = "main_tests_project_pool_package_courtyard.rs"]
mod main_tests_project_pool_package_courtyard;
#[path = "main_tests_project_pool_package_geometry.rs"]
mod main_tests_project_pool_package_geometry;
#[path = "main_tests_project_pool_package_proposals.rs"]
mod main_tests_project_pool_package_proposals;
#[path = "main_tests_project_pool_part.rs"]
mod main_tests_project_pool_part;
#[path = "main_tests_project_pool_part_pin_semantics.rs"]
mod main_tests_project_pool_part_pin_semantics;
#[path = "main_tests_project_pool_pin_pad_map.rs"]
mod main_tests_project_pool_pin_pad_map;
#[path = "main_tests_project_pool_pin_pad_map_b.rs"]
mod main_tests_project_pool_pin_pad_map_b;
#[path = "main_tests_project_pool_resolver.rs"]
mod main_tests_project_pool_resolver;
#[path = "main_tests_project_pool_symbol_library.rs"]
mod main_tests_project_pool_symbol_library;
#[path = "main_tests_project_pool_symbol_pin_anchor.rs"]
mod main_tests_project_pool_symbol_pin_anchor;
#[path = "main_tests_project_port.rs"]
mod main_tests_project_port;
#[path = "main_tests_project_production_idempotency.rs"]
mod main_tests_project_production_idempotency;
#[path = "main_tests_project_proposal_policy.rs"]
mod main_tests_project_proposal_policy;
#[path = "main_tests_project_proposals.rs"]
mod main_tests_project_proposals;
#[path = "main_tests_project_query.rs"]
mod main_tests_project_query;
#[path = "main_tests_project_query_canonical.rs"]
mod main_tests_project_query_canonical;
#[path = "main_tests_project_relationship_diagnostics.rs"]
mod main_tests_project_relationship_diagnostics;
#[path = "main_tests_project_relationships.rs"]
mod main_tests_project_relationships;
#[path = "main_tests_project_repo_validation.rs"]
mod main_tests_project_repo_validation;
#[path = "main_tests_project_resolve_debug_guards.rs"]
mod main_tests_project_resolve_debug_guards;
#[path = "main_tests_project_resolver.rs"]
mod main_tests_project_resolver;
#[path = "main_tests_project_route_apply.rs"]
mod main_tests_project_route_apply;
#[path = "main_tests_project_route_apply_taxonomy.rs"]
mod main_tests_project_route_apply_taxonomy;
#[path = "main_tests_project_route_corridor.rs"]
mod main_tests_project_route_corridor;
#[path = "main_tests_project_route_path_candidate.rs"]
mod main_tests_project_route_path_candidate;
#[path = "main_tests_project_route_path_candidate_authored_copper_graph.rs"]
mod main_tests_project_route_path_candidate_authored_copper_graph;
#[path = "main_tests_project_route_path_candidate_authored_copper_graph_explain.rs"]
mod main_tests_project_route_path_candidate_authored_copper_graph_explain;
#[path = "main_tests_project_route_path_candidate_authored_copper_graph_obstacle_aware.rs"]
mod main_tests_project_route_path_candidate_authored_copper_graph_obstacle_aware;
#[path = "main_tests_project_route_path_candidate_authored_copper_graph_obstacle_aware_explain.rs"]
mod main_tests_project_route_path_candidate_authored_copper_graph_obstacle_aware_explain;
#[path = "main_tests_project_route_path_candidate_authored_copper_graph_zone_aware.rs"]
mod main_tests_project_route_path_candidate_authored_copper_graph_zone_aware;
#[path = "main_tests_project_route_path_candidate_authored_copper_graph_zone_aware_explain.rs"]
mod main_tests_project_route_path_candidate_authored_copper_graph_zone_aware_explain;
#[path = "main_tests_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware.rs"]
mod main_tests_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware;
#[path = "main_tests_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_explain.rs"]
mod main_tests_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_explain;
#[path = "main_tests_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware.rs"]
mod main_tests_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware;
#[path = "main_tests_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_explain.rs"]
mod main_tests_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_explain;
#[path = "main_tests_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware.rs"]
mod main_tests_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware;
#[path = "main_tests_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware_explain.rs"]
mod main_tests_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware_explain;
#[path = "main_tests_project_route_path_candidate_authored_copper_plus_one_gap.rs"]
mod main_tests_project_route_path_candidate_authored_copper_plus_one_gap;
#[path = "main_tests_project_route_path_candidate_authored_copper_plus_one_gap_explain.rs"]
mod main_tests_project_route_path_candidate_authored_copper_plus_one_gap_explain;
#[path = "main_tests_project_route_path_candidate_authored_via_chain.rs"]
mod main_tests_project_route_path_candidate_authored_via_chain;
#[path = "main_tests_project_route_path_candidate_authored_via_chain_explain.rs"]
mod main_tests_project_route_path_candidate_authored_via_chain_explain;
#[path = "main_tests_project_route_path_candidate_explain.rs"]
mod main_tests_project_route_path_candidate_explain;
#[path = "main_tests_project_route_path_candidate_explain_generic.rs"]
mod main_tests_project_route_path_candidate_explain_generic;
#[path = "main_tests_project_route_path_candidate_five_via.rs"]
mod main_tests_project_route_path_candidate_five_via;
#[path = "main_tests_project_route_path_candidate_five_via_explain.rs"]
mod main_tests_project_route_path_candidate_five_via_explain;
#[path = "main_tests_project_route_path_candidate_four_via.rs"]
mod main_tests_project_route_path_candidate_four_via;
#[path = "main_tests_project_route_path_candidate_four_via_explain.rs"]
mod main_tests_project_route_path_candidate_four_via_explain;
#[path = "main_tests_project_route_path_candidate_generic.rs"]
mod main_tests_project_route_path_candidate_generic;
#[path = "main_tests_project_route_path_candidate_six_via.rs"]
mod main_tests_project_route_path_candidate_six_via;
#[path = "main_tests_project_route_path_candidate_six_via_explain.rs"]
mod main_tests_project_route_path_candidate_six_via_explain;
#[path = "main_tests_project_route_path_candidate_three_via.rs"]
mod main_tests_project_route_path_candidate_three_via;
#[path = "main_tests_project_route_path_candidate_three_via_explain.rs"]
mod main_tests_project_route_path_candidate_three_via_explain;
#[path = "main_tests_project_route_path_candidate_two_via.rs"]
mod main_tests_project_route_path_candidate_two_via;
#[path = "main_tests_project_route_path_candidate_two_via_explain.rs"]
mod main_tests_project_route_path_candidate_two_via_explain;
#[path = "main_tests_project_route_path_candidate_via.rs"]
mod main_tests_project_route_path_candidate_via;
#[path = "main_tests_project_route_path_candidate_via_explain.rs"]
mod main_tests_project_route_path_candidate_via_explain;
#[path = "main_tests_project_route_preflight.rs"]
mod main_tests_project_route_preflight;
#[path = "main_tests_project_route_proposal_artifact.rs"]
mod main_tests_project_route_proposal_artifact;
#[path = "main_tests_project_route_proposal_explain.rs"]
mod main_tests_project_route_proposal_explain;
#[path = "main_tests_project_route_proposal_policy.rs"]
mod main_tests_project_route_proposal_policy;
#[path = "main_tests_project_route_proposal_review.rs"]
mod main_tests_project_route_proposal_review;
#[path = "main_tests_project_route_proposal_selection.rs"]
mod main_tests_project_route_proposal_selection;
#[path = "main_tests_project_route_strategy_batch_evaluate.rs"]
mod main_tests_project_route_strategy_batch_evaluate;
#[path = "main_tests_project_route_strategy_batch_result.rs"]
mod main_tests_project_route_strategy_batch_result;
#[path = "main_tests_project_route_strategy_compare.rs"]
mod main_tests_project_route_strategy_compare;
#[path = "main_tests_project_route_strategy_delta.rs"]
mod main_tests_project_route_strategy_delta;
#[path = "main_tests_project_route_strategy_fixture_suite.rs"]
mod main_tests_project_route_strategy_fixture_suite;
#[path = "main_tests_project_route_strategy_repo_baseline.rs"]
mod main_tests_project_route_strategy_repo_baseline;
#[path = "main_tests_project_route_strategy_report.rs"]
mod main_tests_project_route_strategy_report;
#[path = "main_tests_project_routing_substrate.rs"]
mod main_tests_project_routing_substrate;
#[path = "main_tests_project_rules.rs"]
mod main_tests_project_rules;
#[path = "main_tests_project_schematic_proposals.rs"]
mod main_tests_project_schematic_proposals;
#[path = "main_tests_project_schematic_proposals_b.rs"]
mod main_tests_project_schematic_proposals_b;
#[path = "main_tests_project_selected_route_proposal.rs"]
mod main_tests_project_selected_route_proposal;
#[path = "main_tests_project_sheet.rs"]
mod main_tests_project_sheet;
#[path = "main_tests_project_source_shard_reporting.rs"]
mod main_tests_project_source_shard_reporting;
#[path = "main_tests_project_source_shard_root_replay.rs"]
mod main_tests_project_source_shard_root_replay;
#[path = "main_tests_project_source_shard_unknown.rs"]
mod main_tests_project_source_shard_unknown;
#[path = "main_tests_project_source_shard_unknown_generated.rs"]
mod main_tests_project_source_shard_unknown_generated;
#[path = "main_tests_project_source_shard_unknown_schematic.rs"]
mod main_tests_project_source_shard_unknown_schematic;
#[path = "main_tests_project_source_shard_unknown_sidecar.rs"]
mod main_tests_project_source_shard_unknown_sidecar;
#[path = "main_tests_project_standards_clearance_repairs.rs"]
mod main_tests_project_standards_clearance_repairs;
#[path = "main_tests_project_standards_peer_aperture_repairs.rs"]
mod main_tests_project_standards_peer_aperture_repairs;
#[path = "main_tests_project_standards_repair_apply.rs"]
mod main_tests_project_standards_repair_apply;
#[path = "main_tests_project_standards_repairs.rs"]
mod main_tests_project_standards_repairs;
#[path = "main_tests_project_standards_silk_repairs.rs"]
mod main_tests_project_standards_silk_repairs;
#[path = "main_tests_project_standards_zone_fill_repairs.rs"]
mod main_tests_project_standards_zone_fill_repairs;
#[path = "main_tests_project_symbol.rs"]
mod main_tests_project_symbol;
#[path = "main_tests_project_symbol_binding.rs"]
mod main_tests_project_symbol_binding;
#[path = "main_tests_project_symbol_library_materialization.rs"]
mod main_tests_project_symbol_library_materialization;
#[path = "main_tests_project_symbol_resolution.rs"]
mod main_tests_project_symbol_resolution;
#[path = "main_tests_project_symbol_semantics.rs"]
mod main_tests_project_symbol_semantics;
#[path = "main_tests_project_text.rs"]
mod main_tests_project_text;
#[path = "main_tests_project_validate.rs"]
mod main_tests_project_validate;
#[path = "main_tests_project_validate_pool_defaults.rs"]
mod main_tests_project_validate_pool_defaults;
#[path = "main_tests_project_validate_pool_refs.rs"]
mod main_tests_project_validate_pool_refs;
#[path = "main_tests_project_validate_resolver.rs"]
mod main_tests_project_validate_resolver;
#[path = "main_tests_project_validate_rules.rs"]
mod main_tests_project_validate_rules;
#[path = "main_tests_project_wire.rs"]
mod main_tests_project_wire;
#[path = "main_tests_query_goldens.rs"]
mod main_tests_query_goldens;
#[path = "main_tests_query_surface.rs"]
mod main_tests_query_surface;
