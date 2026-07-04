use std::path::Path;

use anyhow::{Context, Result};
use eda_engine::api::native_write::manufacturing::{
    build_create_manufacturing_plan, build_create_panel_projection,
    build_delete_manufacturing_plan, build_delete_panel_projection, build_set_manufacturing_plan,
    build_set_panel_projection, derive_manufacturing_plan_id, derive_panel_projection_id,
};
use eda_engine::api::native_write::{PreparedWrite, WriteProvenance};
use eda_engine::substrate::{
    DesignModel, ManufacturingPlan, ObjectRevision, PRODUCTION_RECORD_SCHEMA_VERSION,
    PanelBoardInstance, PanelProjection, ProjectResolver, Proposal, ProposalCreateRequest,
    ProposalSource, create_draft_proposal_from_batch,
};
use serde::Serialize;
use uuid::Uuid;

use super::load_native_project_with_resolved_board;

use crate::cli_commit_source;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectManufacturingPlanProposalView {
    pub(crate) contract: &'static str,
    pub(crate) action: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) manufacturing_plan: ManufacturingPlan,
    pub(crate) proposal_id: Uuid,
    pub(crate) proposal: Proposal,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectPanelProjectionProposalView {
    pub(crate) contract: &'static str,
    pub(crate) action: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) panel_projection: PanelProjection,
    pub(crate) proposal_id: Uuid,
    pub(crate) proposal: Proposal,
}

pub(crate) fn propose_create_native_project_manufacturing_plan(
    root: &Path,
    prefix: &str,
    name: Option<&str>,
    variant: Option<Uuid>,
    panel_projection: Option<Uuid>,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectManufacturingPlanProposalView> {
    let project = load_native_project_with_resolved_board(root)?;
    let plan_id = derive_manufacturing_plan_id(&project.manifest.uuid, prefix);
    let mut model = ProjectResolver::new(root).resolve()?;
    if model.manufacturing_plans.contains_key(&plan_id) {
        anyhow::bail!("manufacturing plan {plan_id} already exists");
    }
    if let Some(panel_projection_id) = panel_projection {
        if !model.panel_projections.contains_key(&panel_projection_id) {
            anyhow::bail!("panel projection {panel_projection_id} was not found");
        }
    }
    let plan = ManufacturingPlan {
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
        id: plan_id,
        name: name
            .map(str::to_string)
            .unwrap_or_else(|| format!("Manufacturing plan {prefix}")),
        board_or_panel: panel_projection.unwrap_or(project.board.uuid),
        variant,
        prefix: prefix.to_string(),
        object_revision: ObjectRevision(0),
    };
    validate_manufacturing_plan_targets(&model, project.board.uuid, &plan)?;
    let prepared = build_create_manufacturing_plan(
        &model,
        proposal_provenance("propose create manufacturing plan")?,
        &plan,
    )?;
    write_manufacturing_plan_proposal(
        root,
        &mut model,
        plan,
        proposal_id,
        prepared,
        "propose_create_manufacturing_plan",
        rationale
            .map(str::to_string)
            .unwrap_or_else(|| format!("Review ManufacturingPlan {plan_id} creation")),
    )
}

pub(crate) fn propose_update_native_project_manufacturing_plan(
    root: &Path,
    manufacturing_plan_id: Uuid,
    name: Option<&str>,
    prefix: Option<&str>,
    variant: Option<Uuid>,
    clear_variant: bool,
    panel_projection: Option<Uuid>,
    clear_panel_projection: bool,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectManufacturingPlanProposalView> {
    if variant.is_none()
        && !clear_variant
        && panel_projection.is_none()
        && !clear_panel_projection
        && name.is_none()
        && prefix.is_none()
    {
        anyhow::bail!("no manufacturing plan update fields were provided");
    }
    if variant.is_some() && clear_variant {
        anyhow::bail!("--variant and --clear-variant are mutually exclusive");
    }
    if panel_projection.is_some() && clear_panel_projection {
        anyhow::bail!("--panel-projection and --clear-panel-projection are mutually exclusive");
    }

    let project = load_native_project_with_resolved_board(root)?;
    let mut model = ProjectResolver::new(root).resolve()?;
    let previous_plan = model
        .manufacturing_plans
        .get(&manufacturing_plan_id)
        .cloned()
        .with_context(|| format!("manufacturing plan {manufacturing_plan_id} was not found"))?;
    if let Some(panel_projection_id) = panel_projection {
        if !model.panel_projections.contains_key(&panel_projection_id) {
            anyhow::bail!("panel projection {panel_projection_id} was not found");
        }
    }

    let mut plan = previous_plan.clone();
    if let Some(name) = name {
        plan.name = name.to_string();
    }
    if let Some(prefix) = prefix {
        plan.prefix = prefix.to_string();
    }
    if clear_variant {
        plan.variant = None;
    } else if let Some(variant) = variant {
        plan.variant = Some(variant);
    }
    if clear_panel_projection {
        plan.board_or_panel = project.board.uuid;
    } else if let Some(panel_projection) = panel_projection {
        plan.board_or_panel = panel_projection;
    }
    plan.object_revision = ObjectRevision(plan.object_revision.0 + 1);
    validate_manufacturing_plan_targets(&model, project.board.uuid, &plan)?;

    let prepared = build_set_manufacturing_plan(
        &model,
        proposal_provenance("propose update manufacturing plan")?,
        &previous_plan,
        &plan,
    )?;
    write_manufacturing_plan_proposal(
        root,
        &mut model,
        plan,
        proposal_id,
        prepared,
        "propose_update_manufacturing_plan",
        rationale
            .map(str::to_string)
            .unwrap_or_else(|| format!("Review ManufacturingPlan {manufacturing_plan_id} update")),
    )
}

pub(crate) fn propose_delete_native_project_manufacturing_plan(
    root: &Path,
    manufacturing_plan_id: Uuid,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectManufacturingPlanProposalView> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let plan = model
        .manufacturing_plans
        .get(&manufacturing_plan_id)
        .cloned()
        .with_context(|| format!("manufacturing plan {manufacturing_plan_id} was not found"))?;
    if let Some(output_job) = model
        .output_jobs
        .values()
        .find(|job| job.manufacturing_plan == Some(manufacturing_plan_id))
    {
        anyhow::bail!(
            "cannot delete manufacturing plan {manufacturing_plan_id}: output job {} still references it",
            output_job.id
        );
    }
    let prepared = build_delete_manufacturing_plan(
        &model,
        proposal_provenance("propose delete manufacturing plan")?,
        &plan,
    )?;
    write_manufacturing_plan_proposal(
        root,
        &mut model,
        plan,
        proposal_id,
        prepared,
        "propose_delete_manufacturing_plan",
        rationale.map(str::to_string).unwrap_or_else(|| {
            format!("Review ManufacturingPlan {manufacturing_plan_id} deletion")
        }),
    )
}

pub(crate) fn propose_create_native_project_panel_projection(
    root: &Path,
    key: &str,
    name: Option<&str>,
    board: Option<Uuid>,
    x_nm: i64,
    y_nm: i64,
    rotation_deg: i32,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectPanelProjectionProposalView> {
    let project = load_native_project_with_resolved_board(root)?;
    let panel_id = derive_panel_projection_id(&project.manifest.uuid, key);
    let mut model = ProjectResolver::new(root).resolve()?;
    if model.panel_projections.contains_key(&panel_id) {
        anyhow::bail!("panel projection {panel_id} already exists");
    }
    if let Some(board_id) = board {
        validate_project_board_target(
            project.board.uuid,
            board_id,
            "panel projection board instance",
        )?;
    }
    let panel = PanelProjection {
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
        id: panel_id,
        name: name
            .map(str::to_string)
            .unwrap_or_else(|| format!("Panel projection {key}")),
        board_instances: vec![PanelBoardInstance {
            board: board.unwrap_or(project.board.uuid),
            x_nm,
            y_nm,
            rotation_deg,
        }],
        object_revision: ObjectRevision(0),
    };
    let prepared = build_create_panel_projection(
        &model,
        proposal_provenance("propose create panel projection")?,
        &panel,
    )?;
    write_panel_projection_proposal(
        root,
        &mut model,
        panel,
        proposal_id,
        prepared,
        "propose_create_panel_projection",
        rationale
            .map(str::to_string)
            .unwrap_or_else(|| format!("Review PanelProjection {panel_id} creation")),
    )
}

pub(crate) fn propose_update_native_project_panel_projection(
    root: &Path,
    panel_projection_id: Uuid,
    name: Option<&str>,
    board: Option<Uuid>,
    x_nm: Option<i64>,
    y_nm: Option<i64>,
    rotation_deg: Option<i32>,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectPanelProjectionProposalView> {
    if name.is_none()
        && board.is_none()
        && x_nm.is_none()
        && y_nm.is_none()
        && rotation_deg.is_none()
    {
        anyhow::bail!("no panel projection update fields were provided");
    }

    let project = load_native_project_with_resolved_board(root)?;
    let mut model = ProjectResolver::new(root).resolve()?;
    let previous_panel = model
        .panel_projections
        .get(&panel_projection_id)
        .cloned()
        .with_context(|| format!("panel projection {panel_projection_id} was not found"))?;
    let mut panel = previous_panel.clone();
    if let Some(name) = name {
        panel.name = name.to_string();
    }
    if panel.board_instances.is_empty() {
        panel.board_instances.push(PanelBoardInstance {
            board: project.board.uuid,
            x_nm: 0,
            y_nm: 0,
            rotation_deg: 0,
        });
    }
    let first = panel
        .board_instances
        .first_mut()
        .expect("panel projection should have a first board instance");
    if let Some(board) = board {
        validate_project_board_target(
            project.board.uuid,
            board,
            "panel projection board instance",
        )?;
        first.board = board;
    }
    if let Some(x_nm) = x_nm {
        first.x_nm = x_nm;
    }
    if let Some(y_nm) = y_nm {
        first.y_nm = y_nm;
    }
    if let Some(rotation_deg) = rotation_deg {
        first.rotation_deg = rotation_deg;
    }
    panel.object_revision = ObjectRevision(panel.object_revision.0 + 1);

    let prepared = build_set_panel_projection(
        &model,
        proposal_provenance("propose update panel projection")?,
        &previous_panel,
        &panel,
    )?;
    write_panel_projection_proposal(
        root,
        &mut model,
        panel,
        proposal_id,
        prepared,
        "propose_update_panel_projection",
        rationale
            .map(str::to_string)
            .unwrap_or_else(|| format!("Review PanelProjection {panel_projection_id} update")),
    )
}

pub(crate) fn propose_delete_native_project_panel_projection(
    root: &Path,
    panel_projection_id: Uuid,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectPanelProjectionProposalView> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let panel = model
        .panel_projections
        .get(&panel_projection_id)
        .cloned()
        .with_context(|| format!("panel projection {panel_projection_id} was not found"))?;
    if let Some(plan) = model
        .manufacturing_plans
        .values()
        .find(|plan| plan.board_or_panel == panel_projection_id)
    {
        anyhow::bail!(
            "cannot delete panel projection {panel_projection_id}: manufacturing plan {} still references it",
            plan.id
        );
    }
    let prepared = build_delete_panel_projection(
        &model,
        proposal_provenance("propose delete panel projection")?,
        &panel,
    )?;
    write_panel_projection_proposal(
        root,
        &mut model,
        panel,
        proposal_id,
        prepared,
        "propose_delete_panel_projection",
        rationale
            .map(str::to_string)
            .unwrap_or_else(|| format!("Review PanelProjection {panel_projection_id} deletion")),
    )
}

fn proposal_provenance(reason: &str) -> Result<WriteProvenance> {
    Ok(WriteProvenance::new(
        "datum-eda-cli",
        cli_commit_source()?,
        reason,
    ))
}

fn validate_manufacturing_plan_targets(
    model: &DesignModel,
    project_board_id: Uuid,
    plan: &ManufacturingPlan,
) -> Result<()> {
    validate_board_or_panel_target(
        model,
        project_board_id,
        plan.board_or_panel,
        "manufacturing plan",
    )?;
    if let Some(variant_id) = plan.variant {
        validate_variant_target(model, variant_id, "manufacturing plan")?;
    }
    Ok(())
}

fn validate_board_or_panel_target(
    model: &DesignModel,
    project_board_id: Uuid,
    target: Uuid,
    subject: &str,
) -> Result<()> {
    if target == project_board_id || model.panel_projections.contains_key(&target) {
        return Ok(());
    }
    anyhow::bail!("{subject} references missing board or panel {target}");
}

fn validate_project_board_target(
    project_board_id: Uuid,
    target: Uuid,
    subject: &str,
) -> Result<()> {
    if target == project_board_id {
        return Ok(());
    }
    anyhow::bail!("{subject} references missing board {target}");
}

fn validate_variant_target(model: &DesignModel, variant_id: Uuid, subject: &str) -> Result<()> {
    if model.variants.contains_key(&variant_id) {
        return Ok(());
    }
    anyhow::bail!("{subject} references missing variant {variant_id}");
}

fn write_manufacturing_plan_proposal(
    root: &Path,
    model: &mut eda_engine::substrate::DesignModel,
    manufacturing_plan: ManufacturingPlan,
    proposal_id: Option<Uuid>,
    prepared: PreparedWrite,
    action: &'static str,
    rationale: String,
) -> Result<NativeProjectManufacturingPlanProposalView> {
    let proposal = write_proposal(root, model, proposal_id, prepared, rationale)?;
    Ok(NativeProjectManufacturingPlanProposalView {
        contract: "proposal_create_v1",
        action,
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0.clone(),
        manufacturing_plan,
        proposal_id: proposal.proposal_id,
        proposal,
    })
}

fn write_panel_projection_proposal(
    root: &Path,
    model: &mut eda_engine::substrate::DesignModel,
    panel_projection: PanelProjection,
    proposal_id: Option<Uuid>,
    prepared: PreparedWrite,
    action: &'static str,
    rationale: String,
) -> Result<NativeProjectPanelProjectionProposalView> {
    let proposal = write_proposal(root, model, proposal_id, prepared, rationale)?;
    Ok(NativeProjectPanelProjectionProposalView {
        contract: "proposal_create_v1",
        action,
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0.clone(),
        panel_projection,
        proposal_id: proposal.proposal_id,
        proposal,
    })
}

fn write_proposal(
    root: &Path,
    model: &mut eda_engine::substrate::DesignModel,
    proposal_id: Option<Uuid>,
    prepared: PreparedWrite,
    rationale: String,
) -> Result<Proposal> {
    Ok(create_draft_proposal_from_batch(
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
    )?)
}
