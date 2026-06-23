use std::path::Path;

use super::component_instance_journal_ops::{
    apply_component_instance_shard_operation, inverse_component_instance_operation,
    maybe_stage_component_instance_operation,
};
use super::import_map_journal_ops::{
    apply_import_map_shard_operation, inverse_import_map_operation,
    maybe_stage_import_map_operation,
};
use super::pool_journal_ops::{
    apply_pool_shard_operation, inverse_pool_operation, maybe_stage_pool_operation,
};
use super::production_journal_ops::{
    apply_production_shard_operation, inverse_production_operation, stage_production_operation,
};
use super::proposal_journal_ops::{
    apply_proposal_shard_operation, inverse_proposal_operation, maybe_stage_proposal_operation,
};
use super::relationship_journal_ops::{
    apply_relationship_shard_operation, inverse_relationship_operation,
    stage_relationship_operation,
};
use super::zone_fill_journal_ops::{
    apply_zone_fill_shard_operation, inverse_zone_fill_operation, maybe_stage_zone_fill_operation,
};
use super::{EngineError, Operation, OperationBatch, SourceShardKind, journal::StagedShardWrite};

pub(super) fn stage_non_core_operation(
    project_root: &Path,
    batch: &OperationBatch,
    operation: &Operation,
    staged: &mut Vec<StagedShardWrite>,
) -> Result<(), EngineError> {
    if let Some(write) = stage_production_operation(project_root, batch, operation)? {
        staged.push(write);
    }
    if let Some(write) = stage_relationship_operation(project_root, batch, operation)? {
        staged.push(write);
    }
    maybe_stage_pool_operation(project_root, batch, operation, staged)?;
    maybe_stage_import_map_operation(project_root, batch, operation, staged)?;
    maybe_stage_proposal_operation(project_root, batch, operation, staged)?;
    maybe_stage_zone_fill_operation(project_root, batch, operation, staged)?;
    maybe_stage_component_instance_operation(project_root, batch, operation, staged)
}

pub(super) fn inverse_non_core_operation(
    operation: &Operation,
    inverse_operations: &mut Vec<Operation>,
) {
    inverse_production_operation(operation, inverse_operations);
    inverse_relationship_operation(operation, inverse_operations);
    inverse_pool_operation(operation, inverse_operations);
    inverse_import_map_operation(operation, inverse_operations);
    inverse_proposal_operation(operation, inverse_operations);
    inverse_zone_fill_operation(operation, inverse_operations);
    inverse_component_instance_operation(operation, inverse_operations);
}

pub(super) fn apply_non_core_shard_operation(
    shard_kind: &SourceShardKind,
    value: &mut serde_json::Value,
    operation: &Operation,
) -> Result<bool, EngineError> {
    let mut changed = false;
    if matches!(
        shard_kind,
        SourceShardKind::ManufacturingPlan
            | SourceShardKind::PanelProjection
            | SourceShardKind::OutputJob
    ) && apply_production_shard_operation(shard_kind, value, operation)?
    {
        changed = true;
    }
    if matches!(
        shard_kind,
        SourceShardKind::Relationship | SourceShardKind::VariantOverlay
    ) && apply_relationship_shard_operation(shard_kind, value, operation)?
    {
        changed = true;
    }
    if apply_pool_shard_operation(shard_kind, value, operation)?
        || apply_import_map_shard_operation(shard_kind, value, operation)?
        || apply_proposal_shard_operation(shard_kind, value, operation)?
        || apply_zone_fill_shard_operation(shard_kind, value, operation)?
        || apply_component_instance_shard_operation(shard_kind, value, operation)?
    {
        changed = true;
    }
    Ok(changed)
}
