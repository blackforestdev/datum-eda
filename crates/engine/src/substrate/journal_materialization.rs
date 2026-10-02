//! Source materialization belongs to the journal owner; query capture certifies its basis.
use super::*;

pub(in crate::substrate) fn materialized_shard_value(
    model: &DesignModel,
    shard: &SourceShardRef,
) -> Result<serde_json::Value, EngineError> {
    let mut value = match read_json_value(&shard.path) {
        Ok(value) => value,
        Err(_)
            if matches!(
                shard.kind,
                SourceShardKind::SchematicSheet
                    | SourceShardKind::SchematicDefinition
                    | SourceShardKind::Pool
                    | SourceShardKind::ForwardAnnotationReview
                    | SourceShardKind::ProposalMetadata
            ) =>
        {
            match shard.kind {
                SourceShardKind::Pool => {
                    reconstruct_pool_shard_value(&shard.relative_path, &model.journal)?
                }
                SourceShardKind::ForwardAnnotationReview => {
                    reconstruct_forward_annotation_review_value(
                        &shard.relative_path,
                        &model.journal,
                    )?
                }
                SourceShardKind::ProposalMetadata => {
                    reconstruct_proposal_metadata_value(&shard.relative_path, &model.journal)?
                }
                _ => reconstruct_schematic_object_value(&shard.relative_path, &model.journal)?,
            }
        }
        Err(error) => return Err(error),
    };
    if canonical_json_hash(&value)? == shard.content_hash {
        return Ok(value);
    }
    replay_journal_shard_value(&shard.kind, &mut value, &model.journal)?;
    Ok(value)
}

/// Capture current source once. Accept its original byte hash (including lawful
/// noncanonical formatting), otherwise require canonical replay to the expected
/// model hash. Unrelated/tampered disk state cannot masquerade as that revision.
pub(in crate::substrate) fn capture_shard_value(
    model: &DesignModel,
    shard: &SourceShardRef,
) -> Result<serde_json::Value, EngineError> {
    let bytes = std::fs::read(&shard.path)?;
    let mut value: serde_json::Value = serde_json::from_slice(&bytes)?;
    if sha256_hex(&bytes) == shard.content_hash {
        return Ok(value);
    }
    replay_journal_shard_value(&shard.kind, &mut value, &model.journal)?;
    if canonical_json_hash(&value)? != shard.content_hash {
        return Err(EngineError::Validation(format!(
            "stale captured source basis: {}",
            shard.relative_path
        )));
    }
    Ok(value)
}
