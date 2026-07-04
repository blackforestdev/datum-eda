use std::path::Path;

use anyhow::{Context, Result};
use eda_engine::api::native_write::manufacturing::{
    build_create_manufacturing_plan, build_create_panel_projection,
    build_delete_manufacturing_plan, build_delete_panel_projection, build_set_manufacturing_plan,
    build_set_panel_projection, derive_manufacturing_plan_id, derive_panel_projection_id,
};
use eda_engine::api::native_write::{WriteProvenance, commit_prepared};
use eda_engine::substrate::{
    ManufacturingPlan, ObjectRevision, PRODUCTION_RECORD_SCHEMA_VERSION,
    PanelBoardInstance, PanelProjection, ProjectResolver,
};
use serde::Serialize;
use uuid::Uuid;

use super::load_native_project_with_resolved_board;

use crate::command_project::cli_commit_source;

fn cli_provenance(reason: &str) -> Result<WriteProvenance> {
    Ok(WriteProvenance::new("datum-eda-cli", cli_commit_source()?, reason))
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectManufacturingPlansView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) manufacturing_plan_count: usize,
    pub(crate) manufacturing_plans: Vec<ManufacturingPlan>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectManufacturingPlanMutationView {
    pub(crate) contract: &'static str,
    pub(crate) action: &'static str,
    pub(crate) project_id: String,
    pub(crate) manufacturing_plan: ManufacturingPlan,
    pub(crate) manufacturing_plan_path: String,
    pub(crate) created: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectPanelProjectionsView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) panel_projection_count: usize,
    pub(crate) panel_projections: Vec<PanelProjection>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectPanelProjectionMutationView {
    pub(crate) contract: &'static str,
    pub(crate) action: &'static str,
    pub(crate) project_id: String,
    pub(crate) panel_projection: PanelProjection,
    pub(crate) panel_projection_path: String,
    pub(crate) created: bool,
}

pub(crate) fn query_native_project_manufacturing_plans(
    root: &Path,
) -> Result<NativeProjectManufacturingPlansView> {
    let model = ProjectResolver::new(root).resolve()?;
    let manufacturing_plans = model
        .manufacturing_plans
        .values()
        .cloned()
        .collect::<Vec<_>>();
    Ok(NativeProjectManufacturingPlansView {
        contract: "manufacturing_plan_list_v1",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        manufacturing_plan_count: manufacturing_plans.len(),
        manufacturing_plans,
    })
}

pub(crate) fn create_native_project_manufacturing_plan(
    root: &Path,
    prefix: &str,
    name: Option<&str>,
    variant: Option<Uuid>,
    panel_projection: Option<Uuid>,
) -> Result<NativeProjectManufacturingPlanMutationView> {
    let project = load_native_project_with_resolved_board(root)?;
    let plan_id = derive_manufacturing_plan_id(&project.manifest.uuid, prefix);
    let plan_path = root
        .join(".datum/manufacturing_plans")
        .join(format!("{plan_id}.json"));
    let mut model = ProjectResolver::new(root).resolve()?;
    if let Some(plan) = model.manufacturing_plans.get(&plan_id).cloned() {
        return Ok(manufacturing_plan_mutation(
            project.manifest.uuid,
            "create_manufacturing_plan",
            plan,
            plan_path,
            false,
        ));
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
    let prepared = build_create_manufacturing_plan(
        &model,
        cli_provenance("create manufacturing plan")?,
        &plan,
    )?;
    commit_prepared(&mut model, root, prepared)?;
    Ok(manufacturing_plan_mutation(
        project.manifest.uuid,
        "create_manufacturing_plan",
        plan,
        plan_path,
        true,
    ))
}

pub(crate) fn delete_native_project_manufacturing_plan(
    root: &Path,
    manufacturing_plan_id: Uuid,
) -> Result<NativeProjectManufacturingPlanMutationView> {
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
    let plan_path = root
        .join(".datum/manufacturing_plans")
        .join(format!("{manufacturing_plan_id}.json"));
    let prepared = build_delete_manufacturing_plan(
        &model,
        cli_provenance("delete manufacturing plan")?,
        &plan,
    )?;
    commit_prepared(&mut model, root, prepared)?;
    Ok(manufacturing_plan_mutation(
        model.project.project_id,
        "delete_manufacturing_plan",
        plan,
        plan_path,
        false,
    ))
}

pub(crate) fn update_native_project_manufacturing_plan(
    root: &Path,
    manufacturing_plan_id: Uuid,
    name: Option<&str>,
    prefix: Option<&str>,
    variant: Option<Uuid>,
    clear_variant: bool,
    panel_projection: Option<Uuid>,
    clear_panel_projection: bool,
) -> Result<NativeProjectManufacturingPlanMutationView> {
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

    let plan_path = root
        .join(".datum/manufacturing_plans")
        .join(format!("{manufacturing_plan_id}.json"));
    let prepared = build_set_manufacturing_plan(
        &model,
        cli_provenance("update manufacturing plan")?,
        &previous_plan,
        &plan,
    )?;
    commit_prepared(&mut model, root, prepared)?;
    Ok(manufacturing_plan_mutation(
        project.manifest.uuid,
        "update_manufacturing_plan",
        plan,
        plan_path,
        false,
    ))
}

pub(crate) fn query_native_project_panel_projections(
    root: &Path,
) -> Result<NativeProjectPanelProjectionsView> {
    let model = ProjectResolver::new(root).resolve()?;
    let panel_projections = model
        .panel_projections
        .values()
        .cloned()
        .collect::<Vec<_>>();
    Ok(NativeProjectPanelProjectionsView {
        contract: "panel_projection_list_v1",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        panel_projection_count: panel_projections.len(),
        panel_projections,
    })
}

pub(crate) fn create_native_project_panel_projection(
    root: &Path,
    key: &str,
    name: Option<&str>,
    board: Option<Uuid>,
    x_nm: i64,
    y_nm: i64,
    rotation_deg: i32,
) -> Result<NativeProjectPanelProjectionMutationView> {
    let project = load_native_project_with_resolved_board(root)?;
    let panel_id = derive_panel_projection_id(&project.manifest.uuid, key);
    let panel_path = root
        .join(".datum/panel_projections")
        .join(format!("{panel_id}.json"));
    let mut model = ProjectResolver::new(root).resolve()?;
    if let Some(panel) = model.panel_projections.get(&panel_id).cloned() {
        return Ok(panel_projection_mutation(
            project.manifest.uuid,
            "create_panel_projection",
            panel,
            panel_path,
            false,
        ));
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
    let prepared =
        build_create_panel_projection(&model, cli_provenance("create panel projection")?, &panel)?;
    commit_prepared(&mut model, root, prepared)?;
    Ok(panel_projection_mutation(
        project.manifest.uuid,
        "create_panel_projection",
        panel,
        panel_path,
        true,
    ))
}

pub(crate) fn delete_native_project_panel_projection(
    root: &Path,
    panel_projection_id: Uuid,
) -> Result<NativeProjectPanelProjectionMutationView> {
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
    let panel_path = root
        .join(".datum/panel_projections")
        .join(format!("{panel_projection_id}.json"));
    let prepared =
        build_delete_panel_projection(&model, cli_provenance("delete panel projection")?, &panel)?;
    commit_prepared(&mut model, root, prepared)?;
    Ok(panel_projection_mutation(
        model.project.project_id,
        "delete_panel_projection",
        panel,
        panel_path,
        false,
    ))
}

pub(crate) fn update_native_project_panel_projection(
    root: &Path,
    panel_projection_id: Uuid,
    name: Option<&str>,
    board: Option<Uuid>,
    x_nm: Option<i64>,
    y_nm: Option<i64>,
    rotation_deg: Option<i32>,
) -> Result<NativeProjectPanelProjectionMutationView> {
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

    let panel_path = root
        .join(".datum/panel_projections")
        .join(format!("{panel_projection_id}.json"));
    let prepared = build_set_panel_projection(
        &model,
        cli_provenance("update panel projection")?,
        &previous_panel,
        &panel,
    )?;
    commit_prepared(&mut model, root, prepared)?;
    Ok(panel_projection_mutation(
        project.manifest.uuid,
        "update_panel_projection",
        panel,
        panel_path,
        false,
    ))
}

fn manufacturing_plan_mutation(
    project_id: Uuid,
    action: &'static str,
    manufacturing_plan: ManufacturingPlan,
    manufacturing_plan_path: std::path::PathBuf,
    created: bool,
) -> NativeProjectManufacturingPlanMutationView {
    NativeProjectManufacturingPlanMutationView {
        contract: "manufacturing_plan_mutation_v1",
        action,
        project_id: project_id.to_string(),
        manufacturing_plan,
        manufacturing_plan_path: manufacturing_plan_path.display().to_string(),
        created,
    }
}

fn panel_projection_mutation(
    project_id: Uuid,
    action: &'static str,
    panel_projection: PanelProjection,
    panel_projection_path: std::path::PathBuf,
    created: bool,
) -> NativeProjectPanelProjectionMutationView {
    NativeProjectPanelProjectionMutationView {
        contract: "panel_projection_mutation_v1",
        action,
        project_id: project_id.to_string(),
        panel_projection,
        panel_projection_path: panel_projection_path.display().to_string(),
        created,
    }
}
