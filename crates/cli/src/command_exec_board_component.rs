use super::*;

pub(super) fn execute_move_board_component(
    format: &OutputFormat,
    path: std::path::PathBuf,
    component_uuid: uuid::Uuid,
    x_nm: i64,
    y_nm: i64,
) -> Result<(String, i32)> {
    let report = move_native_project_board_component(
        &path,
        component_uuid,
        eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
    )?;
    let output = match format {
        OutputFormat::Text => render_native_project_board_component_mutation_text(&report),
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn execute_set_board_component_part(
    format: &OutputFormat,
    path: std::path::PathBuf,
    component_uuid: uuid::Uuid,
    part_uuid: uuid::Uuid,
) -> Result<(String, i32)> {
    let report = set_native_project_board_component_part(&path, component_uuid, part_uuid)?;
    let output = match format {
        OutputFormat::Text => render_native_project_board_component_mutation_text(&report),
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn execute_set_board_component_package(
    format: &OutputFormat,
    path: std::path::PathBuf,
    component_uuid: uuid::Uuid,
    package_uuid: uuid::Uuid,
) -> Result<(String, i32)> {
    let report = set_native_project_board_component_package(&path, component_uuid, package_uuid)?;
    let output = match format {
        OutputFormat::Text => render_native_project_board_component_mutation_text(&report),
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn execute_set_board_component_layer(
    format: &OutputFormat,
    path: std::path::PathBuf,
    component_uuid: uuid::Uuid,
    layer: i32,
) -> Result<(String, i32)> {
    let report = set_native_project_board_component_layer(&path, component_uuid, layer)?;
    let output = match format {
        OutputFormat::Text => render_native_project_board_component_mutation_text(&report),
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn execute_set_board_component_reference(
    format: &OutputFormat,
    path: std::path::PathBuf,
    component_uuid: uuid::Uuid,
    reference: String,
) -> Result<(String, i32)> {
    let report = set_native_project_board_component_reference(&path, component_uuid, reference)?;
    let output = match format {
        OutputFormat::Text => render_native_project_board_component_mutation_text(&report),
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn execute_set_board_component_value(
    format: &OutputFormat,
    path: std::path::PathBuf,
    component_uuid: uuid::Uuid,
    value: String,
) -> Result<(String, i32)> {
    let report = set_native_project_board_component_value(&path, component_uuid, value)?;
    let output = match format {
        OutputFormat::Text => render_native_project_board_component_mutation_text(&report),
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn execute_rotate_board_component(
    format: &OutputFormat,
    path: std::path::PathBuf,
    component_uuid: uuid::Uuid,
    rotation_deg: i32,
) -> Result<(String, i32)> {
    let report = rotate_native_project_board_component(&path, component_uuid, rotation_deg)?;
    let output = match format {
        OutputFormat::Text => render_native_project_board_component_mutation_text(&report),
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn execute_set_board_component_locked(
    format: &OutputFormat,
    path: std::path::PathBuf,
    component_uuid: uuid::Uuid,
    locked: bool,
) -> Result<(String, i32)> {
    let report = set_native_project_board_component_locked(&path, component_uuid, locked)?;
    let output = match format {
        OutputFormat::Text => render_native_project_board_component_mutation_text(&report),
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}
