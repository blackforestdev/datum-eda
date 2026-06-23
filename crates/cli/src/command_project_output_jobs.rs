use std::path::Path;

use anyhow::{Context, Result};
use eda_engine::substrate::{
    ArtifactKind, ArtifactMetadata, CommitProvenance, CommitSource, ObjectRevision, Operation,
    OperationBatch, OutputJob, OutputJobLogEntry, OutputJobLogLevel, OutputJobRun,
    OutputJobRunStatus, ProjectResolver, persist_output_job_run,
};
use serde::Serialize;
use uuid::Uuid;

#[path = "command_project_output_job_runs.rs"]
mod command_project_output_job_runs;

use self::command_project_output_job_runs::{
    existing_generated_output_job_run, failed_output_job_run, persist_successful_output_job_run,
};
use super::command_project_check_gate::{
    ReleaseCheckGateView, release_check_gate, release_check_gate_error,
};
use super::command_project_gerber_plan::sanitize_export_prefix;
use super::command_project_output_job_include::{
    output_job_id, output_job_id_for_includes, output_job_include_label, output_job_kind_scope,
    parse_output_job_include,
};
use super::load_native_project_with_resolved_board;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectOutputJobsView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) output_job_count: usize,
    pub(crate) output_jobs: Vec<NativeProjectOutputJobStatusView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectOutputJobStatusView {
    #[serde(flatten)]
    pub(crate) output_job: OutputJob,
    pub(crate) status: &'static str,
    pub(crate) execution_count: usize,
    pub(crate) latest_run: Option<OutputJobRun>,
    pub(crate) artifacts: Vec<ArtifactMetadata>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectOutputJobMutationView {
    pub(crate) contract: &'static str,
    pub(crate) action: &'static str,
    pub(crate) project_id: String,
    pub(crate) output_job: OutputJob,
    pub(crate) output_job_path: String,
    pub(crate) created: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectOutputJobRunView {
    pub(crate) contract: &'static str,
    pub(crate) action: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) output_job: OutputJob,
    pub(crate) output_dir: String,
    pub(crate) status: &'static str,
    pub(crate) exit_code: i32,
    pub(crate) output_job_run: OutputJobRun,
    pub(crate) output_job_run_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error: Option<String>,
    pub(crate) check_run: ReleaseCheckGateView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) artifact_report:
        Option<super::command_project_artifacts::NativeProjectArtifactGenerateView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectOutputJobRunLifecycleView {
    pub(crate) contract: &'static str,
    pub(crate) action: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) output_job: OutputJob,
    pub(crate) output_job_run: OutputJobRun,
    pub(crate) output_job_run_path: String,
}

pub(crate) fn query_native_project_output_jobs(root: &Path) -> Result<NativeProjectOutputJobsView> {
    let model = ProjectResolver::new(root).resolve()?;
    let output_jobs = model
        .output_jobs
        .values()
        .map(|job| {
            let mut runs = model
                .output_job_runs
                .values()
                .filter(|run| run.output_job == job.id)
                .cloned()
                .collect::<Vec<_>>();
            runs.sort_by(compare_output_job_runs);
            let latest_run = runs.last().cloned();
            let artifacts = model
                .artifact_metadata
                .values()
                .filter(|artifact| artifact.output_job == Some(job.id))
                .cloned()
                .collect::<Vec<_>>();
            NativeProjectOutputJobStatusView {
                output_job: job.clone(),
                status: latest_run
                    .as_ref()
                    .map(output_job_run_status_name)
                    .unwrap_or("never_run"),
                execution_count: runs.len(),
                latest_run,
                artifacts,
            }
        })
        .collect::<Vec<_>>();
    Ok(NativeProjectOutputJobsView {
        contract: "output_job_list_v1",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        output_job_count: output_jobs.len(),
        output_jobs,
    })
}

fn output_job_run_status_name(run: &OutputJobRun) -> &'static str {
    match run.status {
        OutputJobRunStatus::Running => "running",
        OutputJobRunStatus::Succeeded => "succeeded",
        OutputJobRunStatus::Failed => "failed",
        OutputJobRunStatus::Canceled => "canceled",
    }
}

pub(crate) fn create_native_project_gerber_set_output_job(
    root: &Path,
    prefix: &str,
    output_dir: Option<&Path>,
    name: Option<&str>,
    manufacturing_plan: Option<Uuid>,
    variant: Option<Uuid>,
) -> Result<NativeProjectOutputJobMutationView> {
    let (output_job, output_job_path, created, project_id) =
        ensure_native_project_gerber_set_output_job(
            root,
            prefix,
            output_dir,
            name,
            manufacturing_plan,
            variant,
        )?;
    Ok(NativeProjectOutputJobMutationView {
        contract: "output_job_mutation_v1",
        action: "create_gerber_set_output_job",
        project_id: project_id.to_string(),
        output_job,
        output_job_path: output_job_path.display().to_string(),
        created,
    })
}

pub(crate) fn create_native_project_output_job(
    root: &Path,
    prefix: &str,
    output_dir: Option<&Path>,
    include: &str,
    name: Option<&str>,
    manufacturing_plan: Option<Uuid>,
    variant: Option<Uuid>,
) -> Result<NativeProjectOutputJobMutationView> {
    let include = parse_output_job_include(include)?;
    let (output_job, output_job_path, created, project_id) = ensure_native_project_output_job(
        root,
        prefix,
        output_dir,
        include,
        name,
        manufacturing_plan,
        variant,
    )?;
    Ok(NativeProjectOutputJobMutationView {
        contract: "output_job_mutation_v1",
        action: "create_output_job",
        project_id: project_id.to_string(),
        output_job,
        output_job_path: output_job_path.display().to_string(),
        created,
    })
}

pub(crate) fn update_native_project_output_job(
    root: &Path,
    output_job_id: Uuid,
    name: Option<&str>,
    output_dir: Option<&Path>,
    manufacturing_plan: Option<Uuid>,
    variant: Option<Uuid>,
    clear_manufacturing_plan: bool,
    clear_variant: bool,
    clear_output_dir: bool,
) -> Result<NativeProjectOutputJobMutationView> {
    if name.is_none()
        && output_dir.is_none()
        && manufacturing_plan.is_none()
        && variant.is_none()
        && !clear_manufacturing_plan
        && !clear_variant
        && !clear_output_dir
    {
        anyhow::bail!("update-output-job requires at least one replacement field");
    }

    let mut model = ProjectResolver::new(root).resolve()?;
    let mut output_job = model
        .output_jobs
        .get(&output_job_id)
        .cloned()
        .with_context(|| format!("output job {output_job_id} not found"))?;
    let previous_output_job = output_job.clone();
    if let Some(name) = name {
        output_job.name = name.to_string();
    }
    if clear_output_dir {
        output_job.output_dir = None;
    } else if let Some(output_dir) = output_dir {
        output_job.output_dir = Some(output_dir.to_path_buf());
    }
    if clear_manufacturing_plan {
        output_job.manufacturing_plan = None;
    } else if let Some(manufacturing_plan) = manufacturing_plan {
        output_job.manufacturing_plan = Some(manufacturing_plan);
    }
    if clear_variant {
        output_job.variant = None;
    } else if let Some(variant) = variant {
        output_job.variant = Some(variant);
    }
    output_job.object_revision = ObjectRevision(output_job.object_revision.0 + 1);

    let expected_model_revision = model.model_revision.clone();
    model.commit_journaled(
        root,
        OperationBatch {
            batch_id: Uuid::new_v4(),
            expected_model_revision: Some(expected_model_revision),
            provenance: CommitProvenance {
                actor: "datum-eda-cli".to_string(),
                source: CommitSource::Cli,
                reason: "update output job".to_string(),
            },
            operations: vec![Operation::SetOutputJob {
                output_job_id,
                previous_output_job: serde_json::to_value(&previous_output_job)
                    .context("failed to serialize previous output job operation")?,
                output_job: serde_json::to_value(&output_job)
                    .context("failed to serialize output job operation")?,
            }],
        },
    )?;

    Ok(NativeProjectOutputJobMutationView {
        contract: "output_job_mutation_v1",
        action: "update_output_job",
        project_id: model.project.project_id.to_string(),
        output_job,
        output_job_path: root
            .join(".datum/output_jobs")
            .join(format!("{output_job_id}.json"))
            .display()
            .to_string(),
        created: false,
    })
}

pub(crate) fn run_native_project_output_job(
    root: &Path,
    output_job_id: Uuid,
    output_dir_override: Option<&Path>,
) -> Result<NativeProjectOutputJobRunView> {
    let model = ProjectResolver::new(root).resolve()?;
    let output_job = model
        .output_jobs
        .get(&output_job_id)
        .cloned()
        .with_context(|| format!("output job {output_job_id} not found"))?;
    if output_job.include.is_empty() {
        anyhow::bail!("output job {output_job_id} has no artifact include scopes");
    }
    let output_dir = output_dir_override
        .map(Path::to_path_buf)
        .or_else(|| output_job.output_dir.clone())
        .unwrap_or_else(|| root.join("generated-artifacts"));
    let include = output_job
        .include
        .iter()
        .map(|kind| output_job_kind_scope(*kind))
        .collect::<Vec<_>>()
        .join(",");
    let check_gate = release_check_gate(root)?;
    if check_gate.active_error_count > 0 {
        let error = release_check_gate_error(&check_gate);
        let output_job_run = failed_output_job_run(&model, output_job.id, &include, &error);
        let output_job_run_path = persist_output_job_run(root, &output_job_run)
            .context("failed to persist failed output job run")?;
        return Ok(NativeProjectOutputJobRunView {
            contract: "output_job_run_v1",
            action: "run_output_job",
            project_id: model.project.project_id.to_string(),
            model_revision: model.model_revision.0,
            output_job,
            output_dir: output_dir.display().to_string(),
            status: "failed",
            exit_code: 1,
            output_job_run,
            output_job_run_path: output_job_run_path.display().to_string(),
            error: Some(error),
            check_run: check_gate,
            artifact_report: None,
        });
    }
    let artifact_report = match super::command_project_artifacts::generate_native_project_artifacts(
        root,
        &output_dir,
        &include,
        Some(&output_job.prefix),
        output_job.variant,
        Some(output_job.id),
        false,
    ) {
        Ok(report) => report,
        Err(error) => {
            let error = format!("{error:#}");
            let output_job_run = failed_output_job_run(&model, output_job.id, &include, &error);
            let output_job_run_path = persist_output_job_run(root, &output_job_run)
                .context("failed to persist failed output job run")?;
            return Ok(NativeProjectOutputJobRunView {
                contract: "output_job_run_v1",
                action: "run_output_job",
                project_id: model.project.project_id.to_string(),
                model_revision: model.model_revision.0,
                output_job,
                output_dir: output_dir.display().to_string(),
                status: "failed",
                exit_code: 1,
                output_job_run,
                output_job_run_path: output_job_run_path.display().to_string(),
                error: Some(error),
                check_run: check_gate,
                artifact_report: None,
            });
        }
    };
    let (output_job_run, output_job_run_path) = if artifact_report.generated_count == 1 {
        if let Some((run, path)) = existing_generated_output_job_run(root, &artifact_report) {
            (run, path)
        } else {
            persist_successful_output_job_run(root, output_job.id, &include, &artifact_report)?
        }
    } else {
        persist_successful_output_job_run(root, output_job.id, &include, &artifact_report)?
    };
    Ok(NativeProjectOutputJobRunView {
        contract: "output_job_run_v1",
        action: "run_output_job",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        output_job,
        output_dir: output_dir.display().to_string(),
        status: "succeeded",
        exit_code: 0,
        output_job_run,
        output_job_run_path: output_job_run_path.display().to_string(),
        error: None,
        check_run: check_gate,
        artifact_report: Some(artifact_report),
    })
}

pub(crate) fn next_output_job_run_sequence(
    model: &eda_engine::substrate::DesignModel,
    output_job: Uuid,
) -> u64 {
    model
        .output_job_runs
        .values()
        .filter(|run| run.output_job == output_job)
        .map(|run| run.run_sequence)
        .max()
        .unwrap_or(0)
        + 1
}

fn compare_output_job_runs(a: &OutputJobRun, b: &OutputJobRun) -> std::cmp::Ordering {
    a.run_sequence
        .cmp(&b.run_sequence)
        .then_with(|| a.run_id.cmp(&b.run_id))
}

pub(crate) fn start_native_project_output_job_run(
    root: &Path,
    output_job_id: Uuid,
) -> Result<NativeProjectOutputJobRunLifecycleView> {
    let model = ProjectResolver::new(root).resolve()?;
    let output_job = model
        .output_jobs
        .get(&output_job_id)
        .cloned()
        .with_context(|| format!("output job {output_job_id} not found"))?;
    let output_job_run = lifecycle_output_job_run(
        &model,
        output_job.id,
        OutputJobRunStatus::Running,
        next_output_job_run_sequence(&model, output_job.id),
        0,
        "output job run started",
    );
    let output_job_run_path = persist_output_job_run(root, &output_job_run)
        .context("failed to persist running output job run")?;
    Ok(NativeProjectOutputJobRunLifecycleView {
        contract: "output_job_run_lifecycle_v1",
        action: "start_output_job_run",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        output_job,
        output_job_run,
        output_job_run_path: output_job_run_path.display().to_string(),
    })
}

pub(crate) fn cancel_native_project_output_job_run(
    root: &Path,
    run_id: Uuid,
) -> Result<NativeProjectOutputJobRunLifecycleView> {
    let model = ProjectResolver::new(root).resolve()?;
    let mut output_job_run = model
        .output_job_runs
        .get(&run_id)
        .cloned()
        .with_context(|| format!("output job run {run_id} not found"))?;
    let output_job = model
        .output_jobs
        .get(&output_job_run.output_job)
        .cloned()
        .with_context(|| format!("output job {} not found", output_job_run.output_job))?;
    output_job_run.status = OutputJobRunStatus::Canceled;
    output_job_run.exit_code = Some(130);
    output_job_run.artifact_id = None;
    output_job_run.log.push(OutputJobLogEntry {
        sequence: output_job_run.log.len() as u64 + 1,
        level: OutputJobLogLevel::Warning,
        message: "output job run canceled".to_string(),
    });
    let output_job_run_path = persist_output_job_run(root, &output_job_run)
        .context("failed to persist canceled output job run")?;
    Ok(NativeProjectOutputJobRunLifecycleView {
        contract: "output_job_run_lifecycle_v1",
        action: "cancel_output_job_run",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        output_job,
        output_job_run,
        output_job_run_path: output_job_run_path.display().to_string(),
    })
}

fn lifecycle_output_job_run(
    model: &eda_engine::substrate::DesignModel,
    output_job: Uuid,
    status: OutputJobRunStatus,
    run_sequence: u64,
    sequence_seed: u8,
    message: &str,
) -> OutputJobRun {
    let material = format!(
        "datum-eda:output-job-run:lifecycle:{}:{}:{run_sequence}:{sequence_seed}",
        output_job, model.model_revision.0
    );
    OutputJobRun {
        run_id: Uuid::new_v5(&model.project.project_id, material.as_bytes()),
        output_job,
        run_sequence,
        project_id: model.project.project_id,
        model_revision: model.model_revision.clone(),
        status,
        artifact_id: None,
        exit_code: None,
        provenance: None,
        log: vec![OutputJobLogEntry {
            sequence: 1,
            level: OutputJobLogLevel::Info,
            message: message.to_string(),
        }],
    }
}

pub(crate) fn delete_native_project_output_job(
    root: &Path,
    output_job_id: Uuid,
) -> Result<NativeProjectOutputJobMutationView> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let output_job = model
        .output_jobs
        .get(&output_job_id)
        .cloned()
        .with_context(|| format!("output job {output_job_id} not found"))?;

    let expected_model_revision = model.model_revision.clone();
    model.commit_journaled(
        root,
        OperationBatch {
            batch_id: Uuid::new_v4(),
            expected_model_revision: Some(expected_model_revision),
            provenance: CommitProvenance {
                actor: "datum-eda-cli".to_string(),
                source: CommitSource::Cli,
                reason: "delete output job".to_string(),
            },
            operations: vec![Operation::DeleteOutputJob {
                output_job_id,
                output_job: serde_json::to_value(&output_job)
                    .context("failed to serialize output job operation")?,
            }],
        },
    )?;

    Ok(NativeProjectOutputJobMutationView {
        contract: "output_job_mutation_v1",
        action: "delete_output_job",
        project_id: model.project.project_id.to_string(),
        output_job,
        output_job_path: root
            .join(".datum/output_jobs")
            .join(format!("{output_job_id}.json"))
            .display()
            .to_string(),
        created: false,
    })
}

pub(crate) fn ensure_native_project_gerber_set_output_job(
    root: &Path,
    prefix: &str,
    output_dir: Option<&Path>,
    name: Option<&str>,
    manufacturing_plan: Option<Uuid>,
    variant: Option<Uuid>,
) -> Result<(OutputJob, std::path::PathBuf, bool, Uuid)> {
    ensure_native_project_output_job(
        root,
        prefix,
        output_dir,
        vec![ArtifactKind::GerberSet],
        name,
        manufacturing_plan,
        variant,
    )
}

pub(crate) fn ensure_native_project_manufacturing_set_output_job(
    root: &Path,
    prefix: &str,
    output_dir: Option<&Path>,
    name: Option<&str>,
    manufacturing_plan: Option<Uuid>,
    variant: Option<Uuid>,
) -> Result<(OutputJob, std::path::PathBuf, bool, Uuid)> {
    ensure_native_project_output_job(
        root,
        prefix,
        output_dir,
        vec![ArtifactKind::ManufacturingSet],
        name,
        manufacturing_plan,
        variant,
    )
}

pub(crate) fn find_native_project_output_job_for_scope(
    model: &eda_engine::substrate::DesignModel,
    prefix: &str,
    kind: ArtifactKind,
) -> Option<Uuid> {
    let prefix = sanitize_export_prefix(prefix);
    let job_id = output_job_id(model.project.project_id, &prefix, kind);
    if model.output_jobs.contains_key(&job_id) {
        return Some(job_id);
    }
    model
        .output_jobs
        .values()
        .find(|job| sanitize_export_prefix(&job.prefix) == prefix && job.include.contains(&kind))
        .map(|job| job.id)
}

fn ensure_native_project_output_job(
    root: &Path,
    prefix: &str,
    output_dir: Option<&Path>,
    include: Vec<ArtifactKind>,
    name: Option<&str>,
    manufacturing_plan: Option<Uuid>,
    variant: Option<Uuid>,
) -> Result<(OutputJob, std::path::PathBuf, bool, Uuid)> {
    let project = load_native_project_with_resolved_board(root)?;
    let prefix = sanitize_export_prefix(prefix);
    let job_id = output_job_id_for_includes(project.manifest.uuid, &prefix, &include);
    let output_job_dir = root.join(".datum/output_jobs");
    let output_path = output_job_dir.join(format!("{job_id}.json"));
    if output_path.exists() {
        let job = serde_json::from_str(
            &std::fs::read_to_string(&output_path)
                .with_context(|| format!("failed to read {}", output_path.display()))?,
        )
        .with_context(|| format!("failed to parse {}", output_path.display()))?;
        return Ok((job, output_path, false, project.manifest.uuid));
    }
    let job = OutputJob {
        id: job_id,
        name: name
            .map(str::to_string)
            .unwrap_or_else(|| format!("{} {prefix}", output_job_include_label(&include))),
        include,
        prefix: prefix.clone(),
        output_dir: output_dir.map(Path::to_path_buf),
        board_or_panel: project.board.uuid,
        variant,
        manufacturing_plan,
        object_revision: ObjectRevision(0),
    };
    let mut model = ProjectResolver::new(root).resolve()?;
    let expected_model_revision = model.model_revision.clone();
    model.commit_journaled(
        root,
        OperationBatch {
            batch_id: Uuid::new_v4(),
            expected_model_revision: Some(expected_model_revision),
            provenance: CommitProvenance {
                actor: "datum-eda-cli".to_string(),
                source: CommitSource::Cli,
                reason: "create output job".to_string(),
            },
            operations: vec![Operation::CreateOutputJob {
                output_job_id: job_id,
                output_job: serde_json::to_value(&job)
                    .context("failed to serialize output job operation")?,
            }],
        },
    )?;
    Ok((job, output_path, true, project.manifest.uuid))
}
