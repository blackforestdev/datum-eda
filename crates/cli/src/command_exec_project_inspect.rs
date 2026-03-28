use std::path::Path;

use anyhow::Result;

use super::*;

pub(super) fn execute_project_excellon_drill_inspection(
    format: &OutputFormat,
    path: &Path,
) -> Result<(String, i32)> {
    let report = inspect_excellon_drill(path)?;
    let output = match format {
        OutputFormat::Text => render_native_project_excellon_drill_inspection_text(&report),
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn execute_project_gerber_inspection(
    format: &OutputFormat,
    path: &Path,
) -> Result<(String, i32)> {
    let report = inspect_gerber(path)?;
    let output = match format {
        OutputFormat::Text => render_native_project_gerber_inspection_text(&report),
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}
