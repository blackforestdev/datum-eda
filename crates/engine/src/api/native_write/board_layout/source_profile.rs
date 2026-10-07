//! Explicit BoardRoot profile adoption; build-only, with byte preimage history.
use super::super::context::{BatchComposer, PreparedWrite, WriteProvenance};
use crate::error::EngineError;
use crate::substrate::{DesignModel, ObjectId, Operation, SourceShardKind};

pub fn build_adopt_board_source_profile(
    model: &DesignModel,
    provenance: WriteProvenance,
    board_id: ObjectId,
) -> Result<PreparedWrite, EngineError> {
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.kind == SourceShardKind::BoardRoot)
        .ok_or_else(|| EngineError::Validation("missing authoritative BoardRoot".into()))?;
    let preimage = std::fs::read_to_string(&shard.path)?;
    BatchComposer::compose(model, provenance)
        .push_op(Operation::AdoptBoardSourceProfile { board_id, preimage })
        .primary_object(board_id)
        .finish()
}

#[cfg(test)]
mod tests;
