use super::*;

pub(super) fn is_inventory_command(command: &ProjectCommands) -> bool {
    matches!(
        command,
        ProjectCommands::ExportBom(_)
            | ProjectCommands::CompareBom(_)
            | ProjectCommands::ValidateBom(_)
            | ProjectCommands::InspectBom(_)
            | ProjectCommands::ExportPnp(_)
            | ProjectCommands::ComparePnp(_)
            | ProjectCommands::ValidatePnp(_)
            | ProjectCommands::InspectPnp(_)
    )
}

pub(super) fn execute_inventory_command(
    format: &OutputFormat,
    command: ProjectCommands,
) -> Result<(String, i32)> {
    match command {
        ProjectCommands::ExportBom(ExportBomArgs { path, out }) => {
            let report = export_native_project_bom(&path, &out)?;
            let output = match format {
                OutputFormat::Text => render_native_project_bom_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::CompareBom(CompareBomArgs { path, bom }) => {
            let report = compare_native_project_bom(&path, &bom)?;
            let output = match format {
                OutputFormat::Text => render_native_project_bom_comparison_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ValidateBom(ValidateBomArgs { path, bom }) => {
            let report = validate_native_project_bom(&path, &bom)?;
            let output = match format {
                OutputFormat::Text => render_native_project_bom_validation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            let exit_code = if report.matches_expected { 0 } else { 1 };
            Ok((output, exit_code))
        }
        ProjectCommands::InspectBom(InspectBomArgs { path }) => {
            let report = inspect_native_project_bom(&path)?;
            let output = match format {
                OutputFormat::Text => render_native_project_bom_inspection_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ExportPnp(ExportPnpArgs { path, out }) => {
            let report = export_native_project_pnp(&path, &out)?;
            let output = match format {
                OutputFormat::Text => render_native_project_pnp_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ComparePnp(ComparePnpArgs { path, pnp }) => {
            let report = compare_native_project_pnp(&path, &pnp)?;
            let output = match format {
                OutputFormat::Text => render_native_project_pnp_comparison_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ValidatePnp(ValidatePnpArgs { path, pnp }) => {
            let report = validate_native_project_pnp(&path, &pnp)?;
            let output = match format {
                OutputFormat::Text => render_native_project_pnp_validation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            let exit_code = if report.matches_expected { 0 } else { 1 };
            Ok((output, exit_code))
        }
        ProjectCommands::InspectPnp(InspectPnpArgs { path }) => {
            let report = inspect_native_project_pnp(&path)?;
            let output = match format {
                OutputFormat::Text => render_native_project_pnp_inspection_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        _ => unreachable!("non-inventory command passed to inventory dispatcher"),
    }
}
