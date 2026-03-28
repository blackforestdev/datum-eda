use super::*;

pub(super) fn execute_drill_command(
    format: &OutputFormat,
    command: ProjectCommands,
) -> Result<(String, i32)> {
    match command {
        ProjectCommands::ExportDrill(ExportDrillArgs { path, out }) => {
            let report = export_native_project_drill(&path, &out)?;
            let output = match format {
                OutputFormat::Text => render_native_project_drill_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ValidateDrill(ValidateDrillArgs { path, drill }) => {
            let report = validate_native_project_drill(&path, &drill)?;
            let output = match format {
                OutputFormat::Text => render_native_project_drill_validation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            let exit_code = if report.matches_expected { 0 } else { 1 };
            Ok((output, exit_code))
        }
        ProjectCommands::CompareDrill(CompareDrillArgs { path, drill }) => {
            let report = compare_native_project_drill(&path, &drill)?;
            let output = match format {
                OutputFormat::Text => render_native_project_drill_comparison_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ExportExcellonDrill(ExportExcellonDrillArgs { path, out }) => {
            let report = export_native_project_excellon_drill(&path, &out)?;
            let output = match format {
                OutputFormat::Text => render_native_project_excellon_drill_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::InspectDrill(InspectDrillArgs { path }) => {
            let report = inspect_native_project_drill(&path)?;
            let output = match format {
                OutputFormat::Text => render_native_project_drill_inspection_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::CompareExcellonDrill(CompareExcellonDrillArgs { path, drill }) => {
            let report = compare_native_project_excellon_drill(&path, &drill)?;
            let output = match format {
                OutputFormat::Text => render_native_project_excellon_drill_comparison_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ValidateExcellonDrill(ValidateExcellonDrillArgs { path, drill }) => {
            let report = validate_native_project_excellon_drill(&path, &drill)?;
            let output = match format {
                OutputFormat::Text => render_native_project_excellon_drill_validation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            let exit_code = if report.matches_expected { 0 } else { 1 };
            Ok((output, exit_code))
        }
        ProjectCommands::ReportDrillHoleClasses(ReportDrillHoleClassesArgs { path }) => {
            let report = report_native_project_drill_hole_classes(&path)?;
            let output = match format {
                OutputFormat::Text => render_native_project_drill_hole_class_report_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        _ => unreachable!("non-drill command passed to drill dispatcher"),
    }
}
