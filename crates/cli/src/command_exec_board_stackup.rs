use super::*;

pub(super) fn execute_set_board_stackup(
    format: &OutputFormat,
    path: std::path::PathBuf,
    layers: Vec<String>,
) -> Result<(String, i32)> {
    let stackup_layers = parse_native_stackup_layers(&layers)?;
    let report = set_native_project_board_stackup(&path, stackup_layers)?;
    let output = match format {
        OutputFormat::Text => render_native_project_board_stackup_mutation_text(&report),
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn execute_add_default_top_stackup(
    format: &OutputFormat,
    path: std::path::PathBuf,
) -> Result<(String, i32)> {
    let report = add_native_project_default_top_stackup(&path)?;
    let output = match format {
        OutputFormat::Text => render_native_project_board_stackup_mutation_text(&report),
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}
