use super::*;

pub(super) fn execute_manufacturing_command(
    format: &OutputFormat,
    command: ProjectCommands,
) -> Result<(String, i32)> {
    match command {
        ProjectCommands::ReportManufacturing(ReportManufacturingArgs { path, prefix }) => {
            let report = report_native_project_manufacturing(&path, prefix.as_deref())?;
            let output = match format {
                OutputFormat::Text => render_native_project_manufacturing_report_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        _ => unreachable!("non-manufacturing command passed to manufacturing dispatcher"),
    }
}
