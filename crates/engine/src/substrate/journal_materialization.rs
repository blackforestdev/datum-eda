//! Source materialization belongs to the journal owner; query capture certifies its basis.
use super::*;

impl DesignModel {
    /// Capture one model-owned shard using the existing certified read/replay owner.
    pub fn capture_source_shard_value(
        &self,
        shard_id: uuid::Uuid,
    ) -> Result<serde_json::Value, EngineError> {
        let shard = self
            .source_shards
            .iter()
            .find(|s| s.shard_id == shard_id)
            .ok_or_else(|| {
                EngineError::Validation(format!("model missing source shard {shard_id}"))
            })?;
        capture_shard_value(self, shard)
    }
}

pub(in crate::substrate) fn materialized_shard_value(
    model: &DesignModel,
    shard: &SourceShardRef,
) -> Result<serde_json::Value, EngineError> {
    // Keep the same UTF-8/JSON error conversion as read_json_value, but retain
    // the original hash from that single read instead of reopening the source.
    let observed = (|| -> Result<_, EngineError> {
        let text = std::fs::read_to_string(&shard.path)?;
        let value: serde_json::Value = serde_json::from_str(&text)?;
        Ok((value, sha256_hex(text.as_bytes()) == shard.content_hash))
    })();
    let (mut value, raw_matches) = match observed {
        Ok(observed) => observed,
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
            let reconstructed = match shard.kind {
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
            };
            (reconstructed, false)
        }
        Err(error) => return Err(error),
    };
    if raw_matches || canonical_json_hash(&value)? == shard.content_hash {
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
