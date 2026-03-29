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

#[path = "main_tests_project_bom_compare.rs"]
mod main_tests_project_bom_compare;
#[path = "main_tests_project_bom_export.rs"]
mod main_tests_project_bom_export;
#[path = "main_tests_project_bom_inspect.rs"]
mod main_tests_project_bom_inspect;
#[path = "main_tests_project_bom_validate.rs"]
mod main_tests_project_bom_validate;
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
#[path = "main_tests_project_manufacturing_validate.rs"]
mod main_tests_project_manufacturing_validate;
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
#[path = "main_tests_plan_apply_compare_artifact.rs"]
mod main_tests_plan_apply_compare_artifact;
#[path = "main_tests_plan_apply_inspect_artifact.rs"]
mod main_tests_plan_apply_inspect_artifact;
#[path = "main_tests_project_board_check.rs"]
mod main_tests_project_board_check;
#[path = "main_tests_project_board_component.rs"]
mod main_tests_project_board_component;
#[path = "main_tests_project_board_component_query_single.rs"]
mod main_tests_project_board_component_query_single;
#[path = "main_tests_project_board_component_mechanical.rs"]
mod main_tests_project_board_component_mechanical;
#[path = "main_tests_project_board_component_models_3d.rs"]
mod main_tests_project_board_component_models_3d;
#[path = "main_tests_project_board_component_pads.rs"]
mod main_tests_project_board_component_pads;
#[path = "main_tests_project_board_component_pool_materialization.rs"]
mod main_tests_project_board_component_pool_materialization;
#[path = "main_tests_project_board_component_query.rs"]
mod main_tests_project_board_component_query;
#[path = "main_tests_project_board_component_silkscreen.rs"]
mod main_tests_project_board_component_silkscreen;
#[path = "main_tests_project_board_connectivity.rs"]
mod main_tests_project_board_connectivity;
#[path = "main_tests_project_board_dimension.rs"]
mod main_tests_project_board_dimension;
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
#[path = "main_tests_project_bus.rs"]
mod main_tests_project_bus;
#[path = "main_tests_project_check.rs"]
mod main_tests_project_check;
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
#[path = "main_tests_project_forward_annotation_batch_apply.rs"]
mod main_tests_project_forward_annotation_batch_apply;
#[path = "main_tests_project_forward_annotation_export_inspect.rs"]
mod main_tests_project_forward_annotation_export_inspect;
#[path = "main_tests_project_forward_annotation_artifact_validate.rs"]
mod main_tests_project_forward_annotation_artifact_validate;
#[path = "main_tests_project_forward_annotation_export_selection.rs"]
mod main_tests_project_forward_annotation_export_selection;
#[path = "main_tests_project_forward_annotation_proposal.rs"]
mod main_tests_project_forward_annotation_proposal;
#[path = "main_tests_project_forward_annotation_review.rs"]
mod main_tests_project_forward_annotation_review;
#[path = "main_tests_project_forward_annotation_support.rs"]
mod main_tests_project_forward_annotation_support;
#[path = "main_tests_project_inspect.rs"]
mod main_tests_project_inspect;
#[path = "main_tests_project_junction.rs"]
mod main_tests_project_junction;
#[path = "main_tests_project_label.rs"]
mod main_tests_project_label;
#[path = "main_tests_project_new.rs"]
mod main_tests_project_new;
#[path = "main_tests_project_noconnect.rs"]
mod main_tests_project_noconnect;
#[path = "main_tests_project_port.rs"]
mod main_tests_project_port;
#[path = "main_tests_project_query.rs"]
mod main_tests_project_query;
#[path = "main_tests_project_symbol.rs"]
mod main_tests_project_symbol;
#[path = "main_tests_project_symbol_resolution.rs"]
mod main_tests_project_symbol_resolution;
#[path = "main_tests_project_symbol_semantics.rs"]
mod main_tests_project_symbol_semantics;
#[path = "main_tests_project_text.rs"]
mod main_tests_project_text;
#[path = "main_tests_project_wire.rs"]
mod main_tests_project_wire;
#[path = "main_tests_query_surface.rs"]
mod main_tests_query_surface;
