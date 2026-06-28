use std::path::Path;

use uuid::Uuid;

use super::{
    DesignModel, EngineError, Operation, OperationBatch, SourceShardKind, TransactionRecord,
    ZoneFill,
    generated_evidence_journal_ops::validate_generated_evidence_scope,
    journal::{StagedShardWrite, stage_new_shard_write},
    zone_fill::validated_zone_fill_payload,
};

pub(super) fn maybe_stage_zone_fill_operation(
    project_root: &Path,
    model: &DesignModel,
    batch: &OperationBatch,
    operation: &Operation,
    staged: &mut Vec<StagedShardWrite>,
) -> Result<(), EngineError> {
    match operation {
        Operation::SetZoneFill {
            zone_id, zone_fill, ..
        } => {
            let fill = validated_zone_fill_payload(*zone_id, zone_fill)?;
            validate_generated_evidence_scope("zone fill", None, &fill.model_revision, model)?;
            staged.push(stage_new_shard_write(
                project_root,
                batch,
                SourceShardKind::ZoneFill,
                &zone_fill_relative_path(*zone_id),
                zone_fill,
            )?);
        }
        Operation::DeleteZoneFill { zone_id, .. } => {
            if let Operation::DeleteZoneFill { zone_fill, .. } = operation {
                validated_zone_fill_payload(*zone_id, zone_fill)?;
            }
            let relative_path = zone_fill_relative_path(*zone_id);
            staged.push(StagedShardWrite {
                destination: project_root.join(&relative_path),
                staged: None,
                kind: SourceShardKind::ZoneFill,
                relative_path,
                content_hash: String::new(),
                schema_version: None,
                delete: true,
            });
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn inverse_zone_fill_operation(
    operation: &Operation,
    inverse_operations: &mut Vec<Operation>,
) {
    match operation {
        Operation::SetZoneFill {
            zone_id,
            previous_zone_fill: Some(previous_zone_fill),
            zone_fill,
        } => inverse_operations.push(Operation::SetZoneFill {
            zone_id: *zone_id,
            previous_zone_fill: Some(zone_fill.clone()),
            zone_fill: previous_zone_fill.clone(),
        }),
        Operation::SetZoneFill {
            zone_id,
            previous_zone_fill: None,
            zone_fill,
        } => inverse_operations.push(Operation::DeleteZoneFill {
            zone_id: *zone_id,
            zone_fill: zone_fill.clone(),
        }),
        Operation::DeleteZoneFill { zone_id, zone_fill } => {
            inverse_operations.push(Operation::SetZoneFill {
                zone_id: *zone_id,
                previous_zone_fill: None,
                zone_fill: zone_fill.clone(),
            });
        }
        _ => {}
    }
}

pub(super) fn apply_zone_fill_shard_operation(
    shard_kind: &SourceShardKind,
    value: &mut serde_json::Value,
    operation: &Operation,
) -> Result<bool, EngineError> {
    if shard_kind != &SourceShardKind::ZoneFill {
        return Ok(false);
    }
    let Some(current_id) = value.get("zone_id").and_then(serde_json::Value::as_str) else {
        return Ok(false);
    };
    match operation {
        Operation::SetZoneFill {
            zone_id, zone_fill, ..
        } if current_id == zone_id.to_string() => {
            validated_zone_fill_payload(*zone_id, zone_fill)?;
            *value = zone_fill.clone();
            Ok(true)
        }
        Operation::DeleteZoneFill { zone_id, .. } if current_id == zone_id.to_string() => {
            *value = serde_json::Value::Null;
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub(super) fn zone_fill_relative_path(zone_id: Uuid) -> String {
    format!(".datum/zone_fills/{zone_id}.json")
}

pub(super) fn apply_zone_fill_journal_to_map(
    journal: &[TransactionRecord],
    zone_fills: &mut std::collections::BTreeMap<Uuid, ZoneFill>,
) -> Result<(), EngineError> {
    for transaction in journal {
        for operation in &transaction.operations {
            match operation {
                Operation::SetZoneFill {
                    zone_id, zone_fill, ..
                } => {
                    let fill = validated_zone_fill_payload(*zone_id, zone_fill)?;
                    zone_fills.insert(*zone_id, fill);
                }
                Operation::DeleteZoneFill { zone_id, zone_fill } => {
                    validated_zone_fill_payload(*zone_id, zone_fill)?;
                    zone_fills.remove(zone_id);
                }
                _ => {}
            }
        }
    }
    Ok(())
}
