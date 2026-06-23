use std::collections::BTreeMap;
use std::path::Path;

use uuid::Uuid;

use super::{
    EngineError, ManufacturingPlan, ObjectId, Operation, OperationBatch, OutputJob,
    PanelProjection, SourceShardKind, TransactionRecord, journal::StagedShardWrite,
};
use crate::ir::serialization::to_json_deterministic;

pub(super) fn stage_production_operation(
    project_root: &Path,
    batch: &OperationBatch,
    operation: &Operation,
) -> Result<Option<StagedShardWrite>, EngineError> {
    let Some((kind, object_id, value, delete)) = production_operation_write(operation) else {
        return Ok(None);
    };
    let relative_path = production_relative_path(kind.clone(), object_id)?;
    let destination = project_root.join(&relative_path);
    if delete {
        return Ok(Some(StagedShardWrite {
            destination,
            staged: None,
            kind,
            relative_path,
            content_hash: String::new(),
            delete: true,
        }));
    }

    let stage_path = project_root
        .join(".datum/stage")
        .join(batch.batch_id.to_string())
        .join(&relative_path);
    if let Some(parent) = stage_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = to_json_deterministic(value)?;
    let bytes = format!("{json}\n").into_bytes();
    std::fs::write(&stage_path, &bytes)?;
    std::fs::File::open(&stage_path)?.sync_all()?;
    if let Some(parent) = stage_path.parent() {
        sync_directory(parent)?;
    }

    Ok(Some(StagedShardWrite {
        destination,
        staged: Some(stage_path),
        kind,
        relative_path,
        content_hash: super::sha256_hex(&bytes),
        delete: false,
    }))
}

pub(super) fn inverse_production_operation(
    operation: &Operation,
    inverse_operations: &mut Vec<Operation>,
) {
    match operation {
        Operation::CreateManufacturingPlan {
            manufacturing_plan_id,
            manufacturing_plan,
        } => inverse_operations.push(Operation::DeleteManufacturingPlan {
            manufacturing_plan_id: *manufacturing_plan_id,
            manufacturing_plan: manufacturing_plan.clone(),
        }),
        Operation::SetManufacturingPlan {
            manufacturing_plan_id,
            previous_manufacturing_plan,
            manufacturing_plan,
        } => inverse_operations.push(Operation::SetManufacturingPlan {
            manufacturing_plan_id: *manufacturing_plan_id,
            previous_manufacturing_plan: manufacturing_plan.clone(),
            manufacturing_plan: previous_manufacturing_plan.clone(),
        }),
        Operation::DeleteManufacturingPlan {
            manufacturing_plan_id,
            manufacturing_plan,
        } => inverse_operations.push(Operation::CreateManufacturingPlan {
            manufacturing_plan_id: *manufacturing_plan_id,
            manufacturing_plan: manufacturing_plan.clone(),
        }),
        Operation::CreatePanelProjection {
            panel_projection_id,
            panel_projection,
        } => inverse_operations.push(Operation::DeletePanelProjection {
            panel_projection_id: *panel_projection_id,
            panel_projection: panel_projection.clone(),
        }),
        Operation::SetPanelProjection {
            panel_projection_id,
            previous_panel_projection,
            panel_projection,
        } => inverse_operations.push(Operation::SetPanelProjection {
            panel_projection_id: *panel_projection_id,
            previous_panel_projection: panel_projection.clone(),
            panel_projection: previous_panel_projection.clone(),
        }),
        Operation::DeletePanelProjection {
            panel_projection_id,
            panel_projection,
        } => inverse_operations.push(Operation::CreatePanelProjection {
            panel_projection_id: *panel_projection_id,
            panel_projection: panel_projection.clone(),
        }),
        Operation::CreateOutputJob {
            output_job_id,
            output_job,
        } => inverse_operations.push(Operation::DeleteOutputJob {
            output_job_id: *output_job_id,
            output_job: output_job.clone(),
        }),
        Operation::SetOutputJob {
            output_job_id,
            previous_output_job,
            output_job,
        } => inverse_operations.push(Operation::SetOutputJob {
            output_job_id: *output_job_id,
            previous_output_job: output_job.clone(),
            output_job: previous_output_job.clone(),
        }),
        Operation::DeleteOutputJob {
            output_job_id,
            output_job,
        } => inverse_operations.push(Operation::CreateOutputJob {
            output_job_id: *output_job_id,
            output_job: output_job.clone(),
        }),
        _ => {}
    }
}

pub(super) fn apply_production_shard_operation(
    shard_kind: &SourceShardKind,
    value: &mut serde_json::Value,
    operation: &Operation,
) -> Result<bool, EngineError> {
    let Some((operation_kind, object_id, payload, delete)) = production_operation_write(operation)
    else {
        return Ok(false);
    };
    if &operation_kind != shard_kind {
        return Ok(false);
    }
    let Some(current_id) = value.get("id").and_then(serde_json::Value::as_str) else {
        return Ok(false);
    };
    if current_id != object_id.to_string() {
        return Ok(false);
    }
    if delete {
        *value = serde_json::Value::Null;
    } else {
        *value = payload.clone();
    }
    Ok(true)
}

pub(super) fn production_operation_write(
    operation: &Operation,
) -> Option<(SourceShardKind, Uuid, &serde_json::Value, bool)> {
    match operation {
        Operation::CreateManufacturingPlan {
            manufacturing_plan_id,
            manufacturing_plan,
        } => Some((
            SourceShardKind::ManufacturingPlan,
            *manufacturing_plan_id,
            manufacturing_plan,
            false,
        )),
        Operation::SetManufacturingPlan {
            manufacturing_plan_id,
            manufacturing_plan,
            ..
        } => Some((
            SourceShardKind::ManufacturingPlan,
            *manufacturing_plan_id,
            manufacturing_plan,
            false,
        )),
        Operation::DeleteManufacturingPlan {
            manufacturing_plan_id,
            manufacturing_plan,
        } => Some((
            SourceShardKind::ManufacturingPlan,
            *manufacturing_plan_id,
            manufacturing_plan,
            true,
        )),
        Operation::CreatePanelProjection {
            panel_projection_id,
            panel_projection,
        } => Some((
            SourceShardKind::PanelProjection,
            *panel_projection_id,
            panel_projection,
            false,
        )),
        Operation::SetPanelProjection {
            panel_projection_id,
            panel_projection,
            ..
        } => Some((
            SourceShardKind::PanelProjection,
            *panel_projection_id,
            panel_projection,
            false,
        )),
        Operation::DeletePanelProjection {
            panel_projection_id,
            panel_projection,
        } => Some((
            SourceShardKind::PanelProjection,
            *panel_projection_id,
            panel_projection,
            true,
        )),
        Operation::CreateOutputJob {
            output_job_id,
            output_job,
        } => Some((
            SourceShardKind::OutputJob,
            *output_job_id,
            output_job,
            false,
        )),
        Operation::SetOutputJob {
            output_job_id,
            output_job,
            ..
        } => Some((
            SourceShardKind::OutputJob,
            *output_job_id,
            output_job,
            false,
        )),
        Operation::DeleteOutputJob {
            output_job_id,
            output_job,
        } => Some((SourceShardKind::OutputJob, *output_job_id, output_job, true)),
        _ => None,
    }
}

pub(super) fn production_relative_path(
    shard_kind: SourceShardKind,
    object_id: Uuid,
) -> Result<String, EngineError> {
    let directory = match shard_kind {
        SourceShardKind::ManufacturingPlan => ".datum/manufacturing_plans",
        SourceShardKind::PanelProjection => ".datum/panel_projections",
        SourceShardKind::OutputJob => ".datum/output_jobs",
        _ => {
            return Err(EngineError::Operation(format!(
                "unsupported production shard kind for operation: {shard_kind:?}"
            )));
        }
    };
    Ok(format!("{directory}/{object_id}.json"))
}

pub(super) fn apply_production_journal_to_maps(
    journal: &[TransactionRecord],
    manufacturing_plans: &mut BTreeMap<ObjectId, ManufacturingPlan>,
    panel_projections: &mut BTreeMap<ObjectId, PanelProjection>,
    output_jobs: &mut BTreeMap<ObjectId, OutputJob>,
) -> Result<(), EngineError> {
    for transaction in journal {
        for operation in &transaction.operations {
            match operation {
                Operation::CreateManufacturingPlan {
                    manufacturing_plan_id,
                    manufacturing_plan,
                } => {
                    manufacturing_plans.insert(
                        *manufacturing_plan_id,
                        serde_json::from_value(manufacturing_plan.clone())?,
                    );
                }
                Operation::SetManufacturingPlan {
                    manufacturing_plan_id,
                    manufacturing_plan,
                    ..
                } => {
                    manufacturing_plans.insert(
                        *manufacturing_plan_id,
                        serde_json::from_value(manufacturing_plan.clone())?,
                    );
                }
                Operation::DeleteManufacturingPlan {
                    manufacturing_plan_id,
                    ..
                } => {
                    manufacturing_plans.remove(manufacturing_plan_id);
                }
                Operation::CreatePanelProjection {
                    panel_projection_id,
                    panel_projection,
                } => {
                    panel_projections.insert(
                        *panel_projection_id,
                        serde_json::from_value(panel_projection.clone())?,
                    );
                }
                Operation::SetPanelProjection {
                    panel_projection_id,
                    panel_projection,
                    ..
                } => {
                    panel_projections.insert(
                        *panel_projection_id,
                        serde_json::from_value(panel_projection.clone())?,
                    );
                }
                Operation::DeletePanelProjection {
                    panel_projection_id,
                    ..
                } => {
                    panel_projections.remove(panel_projection_id);
                }
                Operation::CreateOutputJob {
                    output_job_id,
                    output_job,
                } => {
                    output_jobs.insert(*output_job_id, serde_json::from_value(output_job.clone())?);
                }
                Operation::SetOutputJob {
                    output_job_id,
                    output_job,
                    ..
                } => {
                    output_jobs.insert(*output_job_id, serde_json::from_value(output_job.clone())?);
                }
                Operation::DeleteOutputJob { output_job_id, .. } => {
                    output_jobs.remove(output_job_id);
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn sync_directory(path: &Path) -> Result<(), EngineError> {
    std::fs::File::open(path)?.sync_all()?;
    Ok(())
}
