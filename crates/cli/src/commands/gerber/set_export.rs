//! Gerber-set publication lifecycle, including process refusal before output writes.
use super::*;

pub(crate) fn export_native_project_gerber_set_from_plan(
    root: &Path,
    output_dir: &Path,
    plan: NativeProjectGerberPlanView,
    persist_output_run: bool,
    panel_projection: Option<&PanelProjection>,
    output_job_override: Option<Uuid>,
) -> Result<NativeProjectGerberSetExportView> {
    if persist_output_run {
        ensure_release_check_gate_clear(root)?;
    }
    crate::commands::gerber::layers::preflight_process_set(root, &plan)?;
    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("failed to create {}", output_dir.display()))?;
    let output_job = if persist_output_run {
        let (output_job, _, _, _) = ensure_native_project_gerber_set_output_job(
            root,
            &plan.prefix,
            None,
            None,
            None,
            None,
        )?;
        Some(output_job)
    } else {
        None
    };
    let output_job_id = output_job
        .as_ref()
        .map(|job| job.id)
        .or(output_job_override);
    let mut model = ProjectResolver::new(root).resolve()?;
    let mut artifacts = Vec::new();
    let mut artifact_files = Vec::new();
    let mut production_projections = Vec::new();

    for artifact in plan.artifacts {
        let output_path = output_dir.join(&artifact.filename);
        match (artifact.kind.as_str(), artifact.layer_id) {
            ("outline", None) => {
                export_native_project_gerber_outline(root, &output_path)?;
            }
            ("copper", Some(layer)) => {
                let report = export_native_project_gerber_copper_layer(root, layer, &output_path)?;
                if panel_projection.is_none() {
                    production_projections.push(artifact_production_projection_from_view(
                        report.production_projection,
                    ));
                }
            }
            ("soldermask", Some(layer)) => {
                export_native_project_gerber_soldermask_layer(root, layer, &output_path)?;
            }
            ("silkscreen", Some(layer)) => {
                export_native_project_gerber_silkscreen_layer(root, layer, &output_path)?;
            }
            ("paste", Some(layer)) => {
                export_native_project_gerber_paste_layer(root, layer, &output_path)?;
            }
            ("mechanical", Some(layer)) => {
                export_native_project_gerber_mechanical_layer(root, layer, &output_path)?;
            }
            _ => anyhow::bail!("unsupported Gerber export plan artifact: {}", artifact.kind),
        }
        if let Some(panel_projection) = panel_projection {
            let panel_gerber =
                panelize_and_rewrite_rs274x_gerber_file(root, &output_path, panel_projection)?;
            if artifact.kind == "copper" {
                production_projections.push(panel_gerber_production_projection(
                    &model,
                    panel_gerber.as_bytes(),
                ));
            }
        }
        let bytes = std::fs::read(&output_path)
            .with_context(|| format!("failed to read {}", output_path.display()))?;
        let sha256 = compute_source_hash_bytes(&bytes);
        artifact_files.push(ArtifactFile {
            path: PathBuf::from(&artifact.filename),
            sha256: sha256.clone(),
        });
        artifacts.push(NativeProjectGerberSetArtifactView {
            kind: artifact.kind,
            layer_id: artifact.layer_id,
            layer_name: artifact.layer_name,
            filename: artifact.filename,
            output_path: output_path.display().to_string(),
            sha256,
        });
    }
    let artifact_metadata = gerber_set_artifact_metadata(
        &model,
        &plan.prefix,
        output_job_id,
        output_dir,
        artifact_files,
        production_projections,
    );
    let (output_job_run, output_job_run_path) = if persist_output_run {
        let output_job = output_job.expect("persisted Gerber export should have an OutputJob");
        let run = gerber_set_output_job_run(
            &model,
            output_job.id,
            artifact_metadata.artifact_id,
            artifacts.len(),
            &artifact_metadata.production_projections,
        );
        let (_, path) =
            commit_gerber_set_evidence(root, &mut model, &artifact_metadata, Some(&run))
                .context("failed to persist Gerber set generated evidence")?;
        (Some(run), Some(path.display().to_string()))
    } else {
        commit_gerber_set_evidence(root, &mut model, &artifact_metadata, None)
            .context("failed to persist Gerber set artifact metadata")?;
        (None, None)
    };
    let artifact_manifest_path = artifact_metadata_path(root, artifact_metadata.artifact_id);

    Ok(NativeProjectGerberSetExportView {
        action: "export_gerber_set".to_string(),
        project_root: plan.project_root,
        output_dir: output_dir.display().to_string(),
        prefix: plan.prefix,
        written_count: artifacts.len(),
        artifact_manifest_path: artifact_manifest_path.display().to_string(),
        artifact_metadata,
        output_job_run,
        output_job_run_path,
        artifacts,
    })
}
