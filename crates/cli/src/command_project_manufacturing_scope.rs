use std::path::Path;

use anyhow::{Context, Result, bail};
use eda_engine::substrate::{
    ArtifactFile, ArtifactKind, ArtifactMetadata, ArtifactProductionProjection,
    ArtifactValidationState, DesignModel, ManufacturingPlan, OutputJob, PanelProjection,
    ProjectResolver, persist_artifact_metadata,
};
use uuid::Uuid;

use crate::{
    NativeProjectGerberPlanView, NativeProjectManufacturingArtifactView,
    NativeProjectManufacturingManifestEntryView,
};

use super::super::command_project_gerber_plan::sanitize_export_prefix;
use super::super::{
    compute_source_hash_bytes, load_native_project_with_resolved_board,
    plan_native_project_gerber_export,
};

#[derive(Debug, Clone)]
pub(crate) struct NativeProjectManufacturingScope {
    pub(crate) prefix: String,
    pub(crate) variant: Option<Uuid>,
    pub(crate) output_job_id: Option<Uuid>,
    pub(crate) manufacturing_plan_id: Option<Uuid>,
    pub(crate) board_or_panel: Uuid,
    pub(crate) panel_projection: Option<PanelProjection>,
    pub(crate) include: Vec<ArtifactKind>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct NativeProjectManufacturingProjection {
    pub(crate) entries: Vec<NativeProjectManufacturingManifestEntryView>,
    pub(crate) gerber_plan: Option<NativeProjectGerberPlanView>,
}

pub(crate) fn resolve_native_project_manufacturing_scope(
    root: &Path,
    prefix_override: Option<&str>,
    variant_override: Option<Uuid>,
    include_override: Option<&str>,
    output_job_id: Option<Uuid>,
    output_job_name: Option<&str>,
) -> Result<NativeProjectManufacturingScope> {
    let model = ProjectResolver::new(root).resolve()?;
    let job: Option<OutputJob> = if output_job_id.is_some() || output_job_name.is_some() {
        if let Some(output_job_id) = output_job_id {
            Some(
                model
                    .output_jobs
                    .get(&output_job_id)
                    .cloned()
                    .with_context(|| format!("output job {output_job_id} not found"))?,
            )
        } else {
            let name = output_job_name.expect("output job name was checked");
            let matches = model
                .output_jobs
                .values()
                .filter(|job| job.name == name)
                .cloned()
                .collect::<Vec<_>>();
            match matches.len() {
                0 => bail!("output job named {name:?} not found"),
                1 => matches.into_iter().next(),
                count => bail!("output job name {name:?} is ambiguous; {count} jobs match"),
            }
        }
    } else {
        None
    };
    let project = load_native_project_with_resolved_board(root)?;
    let manufacturing_plan: Option<ManufacturingPlan> = job
        .as_ref()
        .and_then(|job| job.manufacturing_plan)
        .map(|plan_id| {
            model
                .manufacturing_plans
                .get(&plan_id)
                .cloned()
                .with_context(|| format!("manufacturing plan {plan_id} not found"))
        })
        .transpose()?;
    let board_or_panel = manufacturing_plan
        .as_ref()
        .map(|plan| plan.board_or_panel)
        .or_else(|| job.as_ref().map(|job| job.board_or_panel))
        .unwrap_or(project.board.uuid);
    let panel_projection = model.panel_projections.get(&board_or_panel).cloned();
    let prefix = prefix_override
        .map(str::to_string)
        .or_else(|| job.as_ref().map(|job| job.prefix.clone()))
        .unwrap_or_else(|| project.board.name.clone());
    let include = if let Some(include) = include_override {
        parse_manufacturing_include(include)?
    } else if let Some(job) = &job {
        job.include.clone()
    } else {
        vec![ArtifactKind::ManufacturingSet]
    };
    Ok(NativeProjectManufacturingScope {
        prefix: sanitize_export_prefix(&prefix),
        variant: variant_override
            .or_else(|| job.as_ref().and_then(|job| job.variant))
            .or_else(|| manufacturing_plan.as_ref().and_then(|plan| plan.variant)),
        output_job_id: job.as_ref().map(|job| job.id),
        manufacturing_plan_id: manufacturing_plan.as_ref().map(|plan| plan.id),
        board_or_panel,
        panel_projection,
        include,
    })
}

pub(crate) fn native_project_manufacturing_projection(
    root: &Path,
    scope: &NativeProjectManufacturingScope,
) -> Result<NativeProjectManufacturingProjection> {
    let mut entries = Vec::new();
    if scope.include.contains(&ArtifactKind::ManufacturingSet)
        || scope.include.contains(&ArtifactKind::Bom)
    {
        entries.push(NativeProjectManufacturingManifestEntryView {
            kind: "bom".to_string(),
            filename: format!("{}-bom.csv", scope.prefix),
            contract: "semantic".to_string(),
        });
    }
    if scope.include.contains(&ArtifactKind::ManufacturingSet)
        || scope.include.contains(&ArtifactKind::Pnp)
    {
        entries.push(NativeProjectManufacturingManifestEntryView {
            kind: "pnp".to_string(),
            filename: format!("{}-pnp.csv", scope.prefix),
            contract: "semantic".to_string(),
        });
    }
    if scope.include.contains(&ArtifactKind::ManufacturingSet)
        || scope.include.contains(&ArtifactKind::Drill)
    {
        entries.push(NativeProjectManufacturingManifestEntryView {
            kind: "drill_csv".to_string(),
            filename: format!("{}-drill.csv", scope.prefix),
            contract: "strict".to_string(),
        });
        entries.push(NativeProjectManufacturingManifestEntryView {
            kind: "excellon_drill".to_string(),
            filename: format!("{}-drill.drl", scope.prefix),
            contract: "semantic".to_string(),
        });
    }
    if scope.include.contains(&ArtifactKind::ManufacturingSet)
        || scope.include.contains(&ArtifactKind::GerberSet)
    {
        let gerber_plan = plan_native_project_gerber_export(root, Some(&scope.prefix))?;
        entries.extend(gerber_plan.artifacts.iter().map(|artifact| {
            NativeProjectManufacturingManifestEntryView {
                kind: format!("gerber_{}", artifact.kind),
                filename: artifact.filename.clone(),
                contract: "semantic".to_string(),
            }
        }));
        return Ok(NativeProjectManufacturingProjection {
            entries,
            gerber_plan: Some(gerber_plan),
        });
    }
    Ok(NativeProjectManufacturingProjection {
        entries,
        gerber_plan: None,
    })
}

pub(crate) fn push_manufacturing_artifact_view(
    output_dir: &Path,
    artifacts: &mut Vec<NativeProjectManufacturingArtifactView>,
    artifact_files: &mut Vec<ArtifactFile>,
    kind: &str,
    filename: &str,
) -> Result<()> {
    let output_path = output_dir.join(filename);
    let bytes = std::fs::read(&output_path)
        .with_context(|| format!("failed to read {}", output_path.display()))?;
    let sha256 = compute_source_hash_bytes(&bytes);
    artifact_files.push(ArtifactFile {
        path: filename.into(),
        sha256: sha256.clone(),
    });
    artifacts.push(NativeProjectManufacturingArtifactView {
        kind: kind.to_string(),
        output_path: output_path.display().to_string(),
        sha256,
    });
    Ok(())
}

pub(crate) fn manufacturing_set_artifact_metadata(
    model: &DesignModel,
    prefix: &str,
    output_job: Option<Uuid>,
    variant: Option<Uuid>,
    output_dir: &Path,
    files: Vec<ArtifactFile>,
    production_projections: Vec<ArtifactProductionProjection>,
) -> ArtifactMetadata {
    let mut material = format!(
        "datum-eda:artifact:manufacturing-set:{}:{}:{}:{}:{:?}",
        model.project.project_id,
        model.model_revision.0,
        env!("CARGO_PKG_VERSION"),
        prefix,
        variant
    );
    for file in &files {
        material.push('|');
        material.push_str(&file.path.to_string_lossy());
        material.push('=');
        material.push_str(&file.sha256);
    }
    ArtifactMetadata {
        artifact_id: Uuid::new_v5(&model.project.project_id, material.as_bytes()),
        kind: ArtifactKind::ManufacturingSet,
        project_id: model.project.project_id,
        model_revision: model.model_revision.clone(),
        output_job,
        variant,
        generator_version: env!("CARGO_PKG_VERSION").to_string(),
        output_dir: Some(output_dir.to_path_buf()),
        files,
        production_projections,
        validation_state: ArtifactValidationState::NotValidated,
    }
}

pub(crate) struct ManufacturingSetArtifactValidation {
    pub(crate) artifact_id: Uuid,
    pub(crate) manifest_path: std::path::PathBuf,
    pub(crate) validation_state: &'static str,
    pub(crate) file_hash_mismatch_count: usize,
}

pub(crate) fn update_manufacturing_set_artifact_validation(
    root: &Path,
    output_dir: &Path,
    expected: &[String],
    content_matches: bool,
) -> Result<Option<ManufacturingSetArtifactValidation>> {
    let model = ProjectResolver::new(root).resolve()?;
    let expected_files = expected
        .iter()
        .map(Into::into)
        .collect::<std::collections::BTreeSet<_>>();
    let Some(mut metadata) = model
        .artifact_metadata
        .values()
        .find(|metadata| {
            metadata.kind == ArtifactKind::ManufacturingSet
                && metadata.model_revision == model.model_revision
                && metadata
                    .files
                    .iter()
                    .map(|file| file.path.clone())
                    .collect::<std::collections::BTreeSet<_>>()
                    == expected_files
        })
        .cloned()
    else {
        return Ok(None);
    };
    let mut file_hash_mismatch_count = 0usize;
    for file in &metadata.files {
        let path = output_dir.join(&file.path);
        let Ok(bytes) = std::fs::read(&path) else {
            file_hash_mismatch_count += 1;
            continue;
        };
        if compute_source_hash_bytes(&bytes) != file.sha256 {
            file_hash_mismatch_count += 1;
        }
    }
    metadata.validation_state = if content_matches && file_hash_mismatch_count == 0 {
        ArtifactValidationState::Valid
    } else {
        ArtifactValidationState::Invalid
    };
    let manifest_path = persist_artifact_metadata(root, &metadata)
        .context("failed to persist manufacturing artifact metadata")?;
    let validation_state = match metadata.validation_state {
        ArtifactValidationState::NotValidated => "not_validated",
        ArtifactValidationState::Valid => "valid",
        ArtifactValidationState::Invalid => "invalid",
    };
    Ok(Some(ManufacturingSetArtifactValidation {
        artifact_id: metadata.artifact_id,
        manifest_path,
        validation_state,
        file_hash_mismatch_count,
    }))
}

fn parse_manufacturing_include(include: &str) -> Result<Vec<ArtifactKind>> {
    let mut parsed = Vec::new();
    for raw_scope in include.split(',') {
        let scope = raw_scope.trim();
        if scope.is_empty() {
            continue;
        }
        let kind = match scope {
            "gerber-set" => ArtifactKind::GerberSet,
            "manufacturing-set" | "all" => ArtifactKind::ManufacturingSet,
            "bom" => ArtifactKind::Bom,
            "pnp" => ArtifactKind::Pnp,
            "drill" => ArtifactKind::Drill,
            _ => bail!(
                "unsupported manufacturing include scope: {scope}; supported scopes: gerber-set, manufacturing-set, bom, pnp, drill, all"
            ),
        };
        if !parsed.contains(&kind) {
            parsed.push(kind);
        }
    }
    if parsed.is_empty() {
        bail!("manufacturing set requires at least one include scope");
    }
    Ok(parsed)
}
