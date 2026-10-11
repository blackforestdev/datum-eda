//! Explicit forward-only BoardRoot capability adoption (PM057).
use std::path::Path;

use super::{DesignModel, EngineError, Operation, OperationBatch, SourceShardKind};

pub(super) const BOARD_SOURCE_PROFILE: u64 = 4;

fn refusal(message: &str) -> EngineError {
    EngineError::Validation(format!("BoardRoot profile adoption: {message}"))
}

pub(super) fn refuse_unjournaled(
    model: &DesignModel,
    batch: &OperationBatch,
) -> Result<(), EngineError> {
    if batch
        .operations
        .iter()
        .any(|op| matches!(op, Operation::Process { .. }))
    {
        return Err(refusal(
            "process operations require the canonical journaled transaction",
        ));
    }
    if super::component_pad_source::touches_batch(model, batch)?
        || batch.operations.iter().any(|op| {
            matches!(op, Operation::AdoptBoardSourceProfile { .. })
                || super::board_graphic_source::identity(op).is_some()
                || super::component_pad_source::touches(op)
        })
    {
        return Err(refusal("requires the canonical journaled transaction"));
    }
    Ok(())
}

pub(super) fn validate_batch(
    model: &DesignModel,
    root: &Path,
    batch: &OperationBatch,
) -> Result<(), EngineError> {
    if batch
        .operations
        .iter()
        .any(|op| super::board_graphic_source::identity(op).is_some())
    {
        let shard = model
            .source_shards
            .iter()
            .find(|s| s.kind == SourceShardKind::BoardRoot)
            .ok_or_else(|| refusal("missing authoritative BoardRoot"))?;
        let bytes = std::fs::read(root.join(&shard.relative_path))?;
        let value: serde_json::Value = serde_json::from_slice(&bytes)?;
        let canonical = format!(
            "{}\n",
            crate::ir::serialization::to_json_deterministic(&value)?
        );
        if super::sha256_hex(&bytes) != shard.content_hash || bytes != canonical.as_bytes() {
            return Err(refusal(
                "graphic writes require a current canonical byte preimage",
            ));
        }
        // Check the same canonical model operations before writing stage artifacts.
        // This also validates ImportMap bindings and source namespace collisions.
        let mut candidate = model.clone();
        super::operation_application_batch::apply_operations(
            &mut candidate,
            &batch.operations,
            &mut super::CommitDiff::default(),
        )?;
    }
    let adoptions = batch
        .operations
        .iter()
        .filter_map(|op| match op {
            Operation::AdoptBoardSourceProfile { board_id, preimage } => Some((board_id, preimage)),
            _ => None,
        })
        .collect::<Vec<_>>();
    if adoptions.is_empty() {
        return Ok(());
    }
    if adoptions.len() != 1
        || batch.operations.iter().any(|op| {
            !matches!(
                op,
                Operation::AdoptBoardSourceProfile { .. } | Operation::GuardObjectRevision { .. }
            )
        })
    {
        return Err(refusal("must be a separate single-adoption transaction"));
    }
    let (board_id, preimage) = adoptions[0];
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.kind == SourceShardKind::BoardRoot)
        .ok_or_else(|| refusal("missing authoritative BoardRoot"))?;
    let bytes = std::fs::read(root.join(&shard.relative_path))?;
    if bytes != preimage.as_bytes() || super::sha256_hex(&bytes) != shard.content_hash {
        return Err(refusal("stale or mismatched original byte preimage"));
    }
    let mut value: serde_json::Value = serde_json::from_slice(&bytes)?;
    apply(
        &mut value,
        &Operation::AdoptBoardSourceProfile {
            board_id: *board_id,
            preimage: preimage.clone(),
        },
    )?;
    Ok(())
}

pub(super) fn apply(value: &mut serde_json::Value, op: &Operation) -> Result<bool, EngineError> {
    let Operation::AdoptBoardSourceProfile { board_id, preimage } = op else {
        return Ok(false);
    };
    let original: serde_json::Value = serde_json::from_str(preimage)?;
    if &original != value
        || value.get("uuid").and_then(serde_json::Value::as_str)
            != Some(board_id.to_string().as_str())
    {
        return Err(refusal("source identity or original preimage mismatch"));
    }
    let profile = value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(1);
    if !(1..BOARD_SOURCE_PROFILE).contains(&profile) {
        return Err(refusal(
            "only supported historical profiles may adopt profile 4",
        ));
    }
    value
        .as_object_mut()
        .ok_or_else(|| refusal("source must be an object"))?
        .insert("schema_version".into(), BOARD_SOURCE_PROFILE.into());
    Ok(true)
}

// Adoption captures actual bytes; replaying old operations onto that preimage
// would confuse migration history with a pending source materialization.
pub(super) fn materialize(
    model: &DesignModel,
    shard: &super::SourceShardRef,
    batch: &OperationBatch,
) -> Result<serde_json::Value, EngineError> {
    if batch.operations.iter().any(|op| {
        matches!(op, Operation::AdoptBoardSourceProfile { .. })
            || matches!(
                op,
                Operation::Process {
                    operation: super::process_operations::ProcessOperation::AdoptCapability { .. }
                }
            )
            || super::board_graphic_source::identity(op).is_some()
    }) {
        super::journal::capture_shard_value(model, shard)
    } else {
        super::journal::materialized_shard_value(model, shard)
    }
}

pub(super) fn validate_native(value: &serde_json::Value) -> Result<(), EngineError> {
    super::process_operations::validate_native(value)?;
    super::board_graphic_source::validate_native(value)?;
    super::component_pad_source::validate_native(value)
}
