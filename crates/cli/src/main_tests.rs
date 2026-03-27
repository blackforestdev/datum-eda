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

#[path = "main_tests_import_plan.rs"]
mod main_tests_import_plan;
#[path = "main_tests_project_new.rs"]
mod main_tests_project_new;
#[path = "main_tests_project_inspect.rs"]
mod main_tests_project_inspect;
#[path = "main_tests_project_query.rs"]
mod main_tests_project_query;
#[path = "main_tests_project_symbol.rs"]
mod main_tests_project_symbol;
#[path = "main_tests_project_symbol_semantics.rs"]
mod main_tests_project_symbol_semantics;
#[path = "main_tests_project_text.rs"]
mod main_tests_project_text;
#[path = "main_tests_project_drawing.rs"]
mod main_tests_project_drawing;
#[path = "main_tests_project_label.rs"]
mod main_tests_project_label;
#[path = "main_tests_project_wire.rs"]
mod main_tests_project_wire;
#[path = "main_tests_project_junction.rs"]
mod main_tests_project_junction;
#[path = "main_tests_project_port.rs"]
mod main_tests_project_port;
#[path = "main_tests_project_bus.rs"]
mod main_tests_project_bus;
#[path = "main_tests_project_noconnect.rs"]
mod main_tests_project_noconnect;
#[path = "main_tests_plan_apply.rs"]
mod main_tests_plan_apply;
#[path = "main_tests_plan_apply_validate.rs"]
mod main_tests_plan_apply_validate;
#[path = "main_tests_plan_apply_text.rs"]
mod main_tests_plan_apply_text;
#[path = "main_tests_plan_apply_legacy_manifest.rs"]
mod main_tests_plan_apply_legacy_manifest;
#[path = "main_tests_plan_apply_legacy_manifest_text.rs"]
mod main_tests_plan_apply_legacy_manifest_text;
#[path = "main_tests_query_surface.rs"]
mod main_tests_query_surface;
#[path = "main_tests_check.rs"]
mod main_tests_check;
#[path = "main_tests_modify_basic.rs"]
mod main_tests_modify_basic;
#[path = "main_tests_modify_advanced.rs"]
mod main_tests_modify_advanced;
#[path = "main_tests_modify_advanced_plan.rs"]
mod main_tests_modify_advanced_plan;
