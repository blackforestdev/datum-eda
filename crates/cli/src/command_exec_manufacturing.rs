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
        ProjectCommands::ExportManufacturingSet(ExportManufacturingSetArgs {
            path,
            output_dir,
            prefix,
        }) => {
            let report =
                export_native_project_manufacturing_set(&path, &output_dir, prefix.as_deref())?;
            let output = match format {
                OutputFormat::Text => render_native_project_manufacturing_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ValidateManufacturingSet(ValidateManufacturingSetArgs {
            path,
            output_dir,
            prefix,
        }) => {
            let report =
                validate_native_project_manufacturing_set(&path, &output_dir, prefix.as_deref())?;
            let output = match format {
                OutputFormat::Text => render_native_project_manufacturing_validation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            let exit_code = if report.missing_count == 0
                && report.mismatched_count == 0
                && report.extra_count == 0
            {
                0
            } else {
                1
            };
            Ok((output, exit_code))
        }
        ProjectCommands::CompareManufacturingSet(CompareManufacturingSetArgs {
            path,
            output_dir,
            prefix,
        }) => {
            let report =
                compare_native_project_manufacturing_set(&path, &output_dir, prefix.as_deref())?;
            let output = match format {
                OutputFormat::Text => render_native_project_manufacturing_comparison_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            let exit_code = if report.missing_count == 0
                && report.mismatched_count == 0
                && report.extra_count == 0
            {
                0
            } else {
                1
            };
            Ok((output, exit_code))
        }
        ProjectCommands::ManifestManufacturingSet(ManifestManufacturingSetArgs {
            path,
            output_dir,
            prefix,
        }) => {
            let report =
                manifest_native_project_manufacturing_set(&path, &output_dir, prefix.as_deref())?;
            let output = match format {
                OutputFormat::Text => render_native_project_manufacturing_manifest_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::InspectManufacturingSet(InspectManufacturingSetArgs {
            path,
            output_dir,
            prefix,
        }) => {
            let report =
                inspect_native_project_manufacturing_set(&path, &output_dir, prefix.as_deref())?;
            let output = match format {
                OutputFormat::Text => render_native_project_manufacturing_inspection_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        _ => unreachable!("non-manufacturing command passed to manufacturing dispatcher"),
    }
}
