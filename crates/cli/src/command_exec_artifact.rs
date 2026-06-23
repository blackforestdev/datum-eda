use super::*;

pub(crate) fn execute_artifact_command(
    format: &OutputFormat,
    action: ArtifactCommands,
) -> Result<(String, i32)> {
    match action {
        ArtifactCommands::Generate(ArtifactGenerateArgs {
            path,
            output_dir,
            include,
            prefix,
            output_job,
        }) => {
            if let Some(output_job) = output_job {
                let report =
                    run_native_project_output_job(&path, output_job, output_dir.as_deref())?;
                let exit_code = report.exit_code;
                return Ok((render_output(format, &report), exit_code));
            }
            let output_dir = output_dir.as_deref().ok_or_else(|| {
                anyhow::anyhow!(
                    "artifact generate requires --output-dir unless --output-job is provided"
                )
            })?;
            let include = include.as_deref().ok_or_else(|| {
                anyhow::anyhow!(
                    "artifact generate requires --include unless --output-job is provided"
                )
            })?;
            Ok((
                render_output(
                    format,
                    &generate_native_project_artifacts(
                        &path,
                        output_dir,
                        include,
                        prefix.as_deref(),
                        None,
                        None,
                        true,
                    )?,
                ),
                0,
            ))
        }
        ArtifactCommands::StartOutputJobRun(ArtifactStartOutputJobRunArgs { path, output_job }) => {
            Ok((
                render_output(
                    format,
                    &start_native_project_output_job_run(&path, output_job)?,
                ),
                0,
            ))
        }
        ArtifactCommands::CancelOutputJobRun(ArtifactCancelOutputJobRunArgs { path, run }) => Ok((
            render_output(format, &cancel_native_project_output_job_run(&path, run)?),
            0,
        )),
        ArtifactCommands::List(ArtifactListArgs { path }) => Ok((
            render_output(format, &query_native_project_artifacts(&path)?),
            0,
        )),
        ArtifactCommands::Show(ArtifactShowArgs { path, artifact }) => Ok((
            render_output(format, &query_native_project_artifact(&path, artifact)?),
            0,
        )),
        ArtifactCommands::Files(ArtifactFilesArgs { path, artifact }) => Ok((
            render_output(
                format,
                &query_native_project_artifact_files(&path, artifact)?,
            ),
            0,
        )),
        ArtifactCommands::Preview(ArtifactPreviewArgs {
            path,
            artifact,
            artifact_dir,
            file,
        }) => Ok((
            render_output(
                format,
                &preview_native_project_artifact_file(
                    &path,
                    artifact,
                    artifact_dir.as_deref(),
                    &file,
                )?,
            ),
            0,
        )),
        ArtifactCommands::Compare(ArtifactCompareArgs {
            path,
            before,
            after,
        }) => Ok((
            render_output(
                format,
                &compare_native_project_artifacts(&path, before, after)?,
            ),
            0,
        )),
        ArtifactCommands::Validate(ArtifactValidateArgs { path, artifact }) => {
            let report = validate_native_project_artifact(&path, artifact)?;
            let exit_code = if report.valid { 0 } else { 1 };
            Ok((render_output(format, &report), exit_code))
        }
        ArtifactCommands::ExportManufacturingSet(ExportManufacturingSetArgs {
            path,
            output_dir,
            prefix,
            output_job,
            job,
            include,
            variant,
        }) => {
            let report = export_native_project_manufacturing_set(
                &path,
                &output_dir,
                prefix.as_deref(),
                variant,
                include.as_deref(),
                output_job,
                job.as_deref(),
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_manufacturing_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ArtifactCommands::ValidateManufacturingSet(ValidateManufacturingSetArgs {
            path,
            output_dir,
            prefix,
            output_job,
            job,
            include,
            variant,
        }) => {
            let report = validate_native_project_manufacturing_set(
                &path,
                &output_dir,
                prefix.as_deref(),
                variant,
                include.as_deref(),
                output_job,
                job.as_deref(),
            )?;
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
    }
}
