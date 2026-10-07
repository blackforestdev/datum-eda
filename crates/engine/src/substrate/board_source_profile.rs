//! Explicit forward-only BoardRoot capability adoption (PM057).
use std::path::Path;

use super::{DesignModel, EngineError, Operation, OperationBatch, SourceShardKind};

pub(super) const BOARD_SOURCE_PROFILE: u64 = 4;

fn refusal(message: &str) -> EngineError {
    EngineError::Validation(format!("BoardRoot profile adoption: {message}"))
}

pub(super) fn refuse_unjournaled(batch: &OperationBatch) -> Result<(), EngineError> {
    if batch
        .operations
        .iter()
        .any(|op| matches!(op, Operation::AdoptBoardSourceProfile { .. }))
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
    if batch
        .operations
        .iter()
        .any(|op| matches!(op, Operation::AdoptBoardSourceProfile { .. }))
    {
        super::journal::capture_shard_value(model, shard)
    } else {
        super::journal::materialized_shard_value(model, shard)
    }
}

// Nonempty graphic source must not load as silently unindexed authority before
// the typed graphic family is delivered. An empty map still declares a feature.
pub(super) fn validate_native(value: &serde_json::Value) -> Result<(), EngineError> {
    if let Some(graphics) = value.get("graphics") {
        if value
            .get("schema_version")
            .and_then(serde_json::Value::as_u64)
            != Some(BOARD_SOURCE_PROFILE)
        {
            return Err(refusal("graphics require declared profile 4"));
        }
        let map = graphics
            .as_object()
            .ok_or_else(|| refusal("graphics must be a UUID-keyed map"))?;
        if !map.is_empty() {
            return Err(refusal(
                "nonempty graphics require the typed graphic source consumer",
            ));
        }
    }
    Ok(())
}
