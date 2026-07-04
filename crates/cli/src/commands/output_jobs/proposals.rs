use std::path::Path;

use anyhow::{Context, Result};
use eda_engine::api::native_write::output_jobs::{
    build_create_output_job, build_delete_output_job, build_set_output_job,
};
use eda_engine::api::native_write::{PreparedWrite, WriteProvenance};
use eda_engine::substrate::{
    DesignModel, ObjectRevision, OutputJob, PRODUCTION_RECORD_SCHEMA_VERSION, ProjectResolver,
    Proposal, ProposalCreateRequest, ProposalSource, create_draft_proposal_from_batch,
};
use serde::Serialize;
use uuid::Uuid;

use super::include::{
    output_job_id_for_includes, output_job_include_label, parse_output_job_include,
};
use super::load_native_project_with_resolved_board;
use crate::commands::gerber::plan::sanitize_export_prefix;

use crate::cli_commit_source;

fn proposal_provenance(reason: &str) -> Result<WriteProvenance> {
    Ok(WriteProvenance::new(
        "datum-eda-cli",
        cli_commit_source()?,
        reason,
    ))
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectOutputJobProposalView {
    pub(crate) contract: &'static str,
    pub(crate) action: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) output_job: OutputJob,
    pub(crate) proposal_id: Uuid,
    pub(crate) proposal: Proposal,
}

pub(crate) fn propose_create_native_project_output_job(
    root: &Path,
    prefix: &str,
    output_dir: Option<&Path>,
    include: &str,
    name: Option<&str>,
    manufacturing_plan: Option<Uuid>,
    variant: Option<Uuid>,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectOutputJobProposalView> {
    let include = parse_output_job_include(include)?;
    let project = load_native_project_with_resolved_board(root)?;
    let prefix = sanitize_export_prefix(prefix);
    let output_job_id = output_job_id_for_includes(project.manifest.uuid, &prefix, &include);
    let mut model = ProjectResolver::new(root).resolve()?;
    if model.output_jobs.contains_key(&output_job_id) {
        anyhow::bail!("output job {output_job_id} already exists");
    }
    let output_job = OutputJob {
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
        id: output_job_id,
        name: name
            .map(str::to_string)
            .unwrap_or_else(|| format!("{} {prefix}", output_job_include_label(&include))),
        include,
        prefix,
        output_dir: output_dir.map(Path::to_path_buf),
        board_or_panel: project.board.uuid,
        variant,
        manufacturing_plan,
        object_revision: ObjectRevision(0),
    };
    validate_output_job_targets(&model, project.board.uuid, &output_job)?;
    let prepared = build_create_output_job(
        &model,
        proposal_provenance("propose create output job")?,
        &output_job,
    )?;
    write_output_job_proposal(
        root,
        &mut model,
        output_job,
        proposal_id,
        prepared,
        "propose_create_output_job",
        rationale
            .map(str::to_string)
            .unwrap_or_else(|| format!("Review OutputJob {output_job_id} creation")),
    )
}

pub(crate) fn propose_update_native_project_output_job(
    root: &Path,
    output_job_id: Uuid,
    name: Option<&str>,
    output_dir: Option<&Path>,
    manufacturing_plan: Option<Uuid>,
    variant: Option<Uuid>,
    clear_manufacturing_plan: bool,
    clear_variant: bool,
    clear_output_dir: bool,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectOutputJobProposalView> {
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

    let project = load_native_project_with_resolved_board(root)?;
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
    validate_output_job_targets(&model, project.board.uuid, &output_job)?;

    let prepared = build_set_output_job(
        &model,
        proposal_provenance("propose update output job")?,
        &previous_output_job,
        &output_job,
    )?;
    write_output_job_proposal(
        root,
        &mut model,
        output_job,
        proposal_id,
        prepared,
        "propose_update_output_job",
        rationale
            .map(str::to_string)
            .unwrap_or_else(|| format!("Review OutputJob {output_job_id} update")),
    )
}

pub(crate) fn propose_delete_native_project_output_job(
    root: &Path,
    output_job_id: Uuid,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectOutputJobProposalView> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let output_job = model
        .output_jobs
        .get(&output_job_id)
        .cloned()
        .with_context(|| format!("output job {output_job_id} not found"))?;
    let prepared = build_delete_output_job(
        &model,
        proposal_provenance("propose delete output job")?,
        &output_job,
    )?;
    write_output_job_proposal(
        root,
        &mut model,
        output_job,
        proposal_id,
        prepared,
        "propose_delete_output_job",
        rationale
            .map(str::to_string)
            .unwrap_or_else(|| format!("Review OutputJob {output_job_id} deletion")),
    )
}

fn write_output_job_proposal(
    root: &Path,
    model: &mut eda_engine::substrate::DesignModel,
    output_job: OutputJob,
    proposal_id: Option<Uuid>,
    prepared: PreparedWrite,
    action: &'static str,
    rationale: String,
) -> Result<NativeProjectOutputJobProposalView> {
    let proposal = create_draft_proposal_from_batch(
        model,
        root,
        ProposalCreateRequest {
            proposal_id,
            batch: prepared.batch,
            rationale,
            source: ProposalSource::Cli,
            checks_run: Vec::new(),
            finding_fingerprints: Vec::new(),
        },
    )?;
    Ok(NativeProjectOutputJobProposalView {
        contract: "proposal_create_v1",
        action,
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0.clone(),
        output_job,
        proposal_id: proposal.proposal_id,
        proposal,
    })
}

fn validate_output_job_targets(
    model: &DesignModel,
    project_board_id: Uuid,
    output_job: &OutputJob,
) -> Result<()> {
    if output_job.board_or_panel != project_board_id
        && !model
            .panel_projections
            .contains_key(&output_job.board_or_panel)
    {
        anyhow::bail!(
            "output job references missing board or panel {}",
            output_job.board_or_panel
        );
    }
    if let Some(variant_id) = output_job.variant {
        if !model.variants.contains_key(&variant_id) {
            anyhow::bail!("output job references missing variant {variant_id}");
        }
    }
    if let Some(manufacturing_plan_id) = output_job.manufacturing_plan {
        if !model
            .manufacturing_plans
            .contains_key(&manufacturing_plan_id)
        {
            anyhow::bail!(
                "output job references missing manufacturing plan {manufacturing_plan_id}"
            );
        }
    }
    Ok(())
}
