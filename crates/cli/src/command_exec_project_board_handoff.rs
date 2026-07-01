use super::*;

pub(super) fn execute_place_board_component(
    format: &OutputFormat,
    path: PathBuf,
    part_uuid: Uuid,
    package_uuid: Uuid,
    reference: String,
    value: String,
    x_nm: i64,
    y_nm: i64,
    layer: i32,
) -> Result<(String, i32)> {
    let report = place_native_project_board_component(
        &path,
        part_uuid,
        package_uuid,
        reference,
        value,
        eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
        layer,
    )?;
    let output = match format {
        OutputFormat::Text => render_native_project_board_component_mutation_text(&report),
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn execute_set_board_outline(
    format: &OutputFormat,
    path: PathBuf,
    vertices: Vec<String>,
) -> Result<(String, i32)> {
    let polygon = parse_native_polygon_vertices(&vertices)?;
    let report = set_native_project_board_outline(&path, polygon)?;
    let output = match format {
        OutputFormat::Text => render_native_project_board_outline_mutation_text(&report),
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn execute_set_board_name(
    format: &OutputFormat,
    path: PathBuf,
    name: String,
) -> Result<(String, i32)> {
    let report = set_native_project_board_name(&path, name)?;
    let output = match format {
        OutputFormat::Text => render_native_project_board_name_mutation_text(&report),
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn execute_generate_board_components(
    format: &OutputFormat,
    args: ProjectGenerateBoardComponentsArgs,
) -> Result<(String, i32)> {
    let ProjectGenerateBoardComponentsArgs {
        path,
        apply,
        as_proposal,
        proposal,
        rationale,
        origin_x_nm,
        origin_y_nm,
        pitch_nm,
        layer,
    } = args;
    let report = generate_native_project_board_components(
        &path,
        apply,
        as_proposal,
        proposal,
        rationale,
        eda_engine::ir::geometry::Point {
            x: origin_x_nm,
            y: origin_y_nm,
        },
        pitch_nm,
        layer,
    )?;
    let output = match format {
        OutputFormat::Text => render_native_project_board_handoff_text(&report),
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}
