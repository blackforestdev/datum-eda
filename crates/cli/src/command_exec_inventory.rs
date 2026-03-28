use super::*;

pub(super) fn execute_inventory_command(
    format: &OutputFormat,
    command: ProjectCommands,
) -> Result<(String, i32)> {
    match command {
        ProjectCommands::ExportBom { path, out } => {
            let report = export_native_project_bom(&path, &out)?;
            let output = match format {
                OutputFormat::Text => render_native_project_bom_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::CompareBom { path, bom } => {
            let report = compare_native_project_bom(&path, &bom)?;
            let output = match format {
                OutputFormat::Text => render_native_project_bom_comparison_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::InspectBom { path } => {
            let report = inspect_native_project_bom(&path)?;
            let output = match format {
                OutputFormat::Text => render_native_project_bom_inspection_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ExportPnp { path, out } => {
            let report = export_native_project_pnp(&path, &out)?;
            let output = match format {
                OutputFormat::Text => render_native_project_pnp_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ComparePnp { path, pnp } => {
            let report = compare_native_project_pnp(&path, &pnp)?;
            let output = match format {
                OutputFormat::Text => render_native_project_pnp_comparison_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        _ => unreachable!("non-inventory command passed to inventory dispatcher"),
    }
}
