use super::*;

pub(super) fn execute_gerber_workflow_command(
    format: &OutputFormat,
    command: ProjectCommands,
) -> Result<(String, i32)> {
    match command {
        ProjectCommands::PlanGerberExport(PlanGerberExportArgs { path, prefix }) => {
            execute_plan_gerber_export(format, path, prefix)
        }
        ProjectCommands::ExportGerberSet(ExportGerberSetArgs {
            path,
            output_dir,
            prefix,
        }) => execute_export_gerber_set(format, path, output_dir, prefix),
        ProjectCommands::CompareGerberExportPlan(CompareGerberExportPlanArgs {
            path,
            output_dir,
            prefix,
        }) => execute_compare_gerber_export_plan(format, path, output_dir, prefix),
        ProjectCommands::ValidateGerberSet(ValidateGerberSetArgs {
            path,
            output_dir,
            prefix,
        }) => execute_validate_gerber_set(format, path, output_dir, prefix),
        ProjectCommands::CompareGerberSet(CompareGerberSetArgs {
            path,
            output_dir,
            prefix,
        }) => execute_compare_gerber_set(format, path, output_dir, prefix),
        _ => unreachable!("non-Gerber workflow command passed to Gerber workflow dispatcher"),
    }
}

pub(super) fn execute_plan_gerber_export(
    format: &OutputFormat,
    path: std::path::PathBuf,
    prefix: Option<String>,
) -> Result<(String, i32)> {
    let report = plan_native_project_gerber_export(&path, prefix.as_deref())?;
    let output = match format {
        OutputFormat::Text => render_native_project_gerber_plan_text(&report),
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn execute_export_gerber_set(
    format: &OutputFormat,
    path: std::path::PathBuf,
    output_dir: std::path::PathBuf,
    prefix: Option<String>,
) -> Result<(String, i32)> {
    let report = export_native_project_gerber_set(&path, &output_dir, prefix.as_deref())?;
    let output = match format {
        OutputFormat::Text => render_native_project_gerber_set_export_text(&report),
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn execute_compare_gerber_export_plan(
    format: &OutputFormat,
    path: std::path::PathBuf,
    output_dir: std::path::PathBuf,
    prefix: Option<String>,
) -> Result<(String, i32)> {
    let report = compare_native_project_gerber_export_plan(&path, &output_dir, prefix.as_deref())?;
    let output = match format {
        OutputFormat::Text => render_native_project_gerber_plan_comparison_text(&report),
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn execute_validate_gerber_set(
    format: &OutputFormat,
    path: std::path::PathBuf,
    output_dir: std::path::PathBuf,
    prefix: Option<String>,
) -> Result<(String, i32)> {
    let report = validate_native_project_gerber_set(&path, &output_dir, prefix.as_deref())?;
    let output = match format {
        OutputFormat::Text => render_native_project_gerber_set_validation_text(&report),
        OutputFormat::Json => render_output(format, &report),
    };
    let exit_code =
        if report.missing_count == 0 && report.mismatched_count == 0 && report.extra_count == 0 {
            0
        } else {
            1
        };
    Ok((output, exit_code))
}

pub(super) fn execute_compare_gerber_set(
    format: &OutputFormat,
    path: std::path::PathBuf,
    output_dir: std::path::PathBuf,
    prefix: Option<String>,
) -> Result<(String, i32)> {
    let report = compare_native_project_gerber_set(&path, &output_dir, prefix.as_deref())?;
    let output = match format {
        OutputFormat::Text => render_native_project_gerber_set_comparison_text(&report),
        OutputFormat::Json => render_output(format, &report),
    };
    let exit_code =
        if report.missing_count == 0 && report.mismatched_count == 0 && report.extra_count == 0 {
            0
        } else {
            1
        };
    Ok((output, exit_code))
}
