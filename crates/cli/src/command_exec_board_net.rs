use super::*;

pub(super) fn execute_place_board_net(
    format: &OutputFormat,
    path: PathBuf,
    name: String,
    class_uuid: Uuid,
) -> Result<(String, i32)> {
    let report = place_native_project_board_net(&path, name, class_uuid)?;
    let output = match format {
        OutputFormat::Text => render_native_project_board_net_mutation_text(&report),
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn execute_edit_board_net(
    format: &OutputFormat,
    path: PathBuf,
    net_uuid: Uuid,
    name: Option<String>,
    class_uuid: Option<Uuid>,
) -> Result<(String, i32)> {
    let report = edit_native_project_board_net(&path, net_uuid, name, class_uuid)?;
    let output = match format {
        OutputFormat::Text => render_native_project_board_net_mutation_text(&report),
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn execute_delete_board_net(
    format: &OutputFormat,
    path: PathBuf,
    net_uuid: Uuid,
) -> Result<(String, i32)> {
    let report = delete_native_project_board_net(&path, net_uuid)?;
    let output = match format {
        OutputFormat::Text => render_native_project_board_net_mutation_text(&report),
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn execute_place_board_net_class(
    format: &OutputFormat,
    path: PathBuf,
    name: String,
    clearance_nm: i64,
    track_width_nm: i64,
    via_drill_nm: i64,
    via_diameter_nm: i64,
    diffpair_width_nm: i64,
    diffpair_gap_nm: i64,
) -> Result<(String, i32)> {
    let report = place_native_project_board_net_class(
        &path,
        name,
        clearance_nm,
        track_width_nm,
        via_drill_nm,
        via_diameter_nm,
        diffpair_width_nm,
        diffpair_gap_nm,
    )?;
    let output = match format {
        OutputFormat::Text => render_native_project_board_net_class_mutation_text(&report),
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn execute_edit_board_net_class(
    format: &OutputFormat,
    path: PathBuf,
    net_class_uuid: Uuid,
    name: Option<String>,
    clearance_nm: Option<i64>,
    track_width_nm: Option<i64>,
    via_drill_nm: Option<i64>,
    via_diameter_nm: Option<i64>,
    diffpair_width_nm: Option<i64>,
    diffpair_gap_nm: Option<i64>,
) -> Result<(String, i32)> {
    let report = edit_native_project_board_net_class(
        &path,
        net_class_uuid,
        name,
        clearance_nm,
        track_width_nm,
        via_drill_nm,
        via_diameter_nm,
        diffpair_width_nm,
        diffpair_gap_nm,
    )?;
    let output = match format {
        OutputFormat::Text => render_native_project_board_net_class_mutation_text(&report),
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn execute_delete_board_net_class(
    format: &OutputFormat,
    path: PathBuf,
    net_class_uuid: Uuid,
) -> Result<(String, i32)> {
    let report = delete_native_project_board_net_class(&path, net_class_uuid)?;
    let output = match format {
        OutputFormat::Text => render_native_project_board_net_class_mutation_text(&report),
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}
