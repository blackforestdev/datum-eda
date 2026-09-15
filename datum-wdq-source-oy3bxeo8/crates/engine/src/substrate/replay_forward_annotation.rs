use std::path::Path;

use super::forward_annotation_review_journal_ops::{
    FORWARD_ANNOTATION_REVIEW_RELATIVE_PATH, forward_annotation_review_operation_write,
};
use super::{
    EngineError, SourceShardKind, SourceShardRef, TransactionRecord, read_json_value,
    source_shard_ref_builders::source_shard_ref_for_value,
};

pub(super) fn replay_forward_annotation_review_shard(
    project_root: &Path,
    shards: &mut Vec<SourceShardRef>,
    journal: &[TransactionRecord],
) -> Result<(), EngineError> {
    let mut value = shards
        .iter()
        .find(|shard| shard.kind == SourceShardKind::ForwardAnnotationReview)
        .filter(|shard| shard.path.exists())
        .and_then(|shard| read_json_value(&shard.path).ok());
    let mut touched = false;
    for transaction in journal {
        for operation in &transaction.operations {
            let Some((relative_path, next_value)) =
                forward_annotation_review_operation_write(operation)
            else {
                continue;
            };
            if relative_path != FORWARD_ANNOTATION_REVIEW_RELATIVE_PATH {
                continue;
            }
            touched = true;
            value = next_value.cloned();
        }
    }
    if !touched {
        return Ok(());
    }
    shards.retain(|shard| shard.kind != SourceShardKind::ForwardAnnotationReview);
    if let Some(value) = value {
        shards.push(source_shard_ref_for_value(
            project_root,
            SourceShardKind::ForwardAnnotationReview,
            FORWARD_ANNOTATION_REVIEW_RELATIVE_PATH.to_string(),
            &value,
        )?);
    }
    Ok(())
}
