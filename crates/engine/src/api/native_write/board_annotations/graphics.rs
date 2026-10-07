//! Build-only graphic authoring through the existing native annotation family.
use super::super::{BatchComposer, PreparedWrite, WriteProvenance};
use crate::board::graphics::BoardGraphic;
use crate::error::EngineError;
use crate::substrate::{DesignModel, ObjectId, Operation, SourceShardKind};

pub fn build_create_board_graphic(
    model: &DesignModel,
    provenance: WriteProvenance,
    board_id: ObjectId,
    graphic: &BoardGraphic,
) -> Result<PreparedWrite, EngineError> {
    let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    BatchComposer::compose(model, provenance)
        .push_op(Operation::CreateBoardGraphic {
            board_id,
            graphic_id: graphic.uuid,
            graphic: serde_json::to_value(graphic)?,
            map_was_present: board.get("graphics").is_some(),
        })
        .primary_object(graphic.uuid)
        .finish()
}

pub fn build_set_board_graphic(
    model: &DesignModel,
    provenance: WriteProvenance,
    board_id: ObjectId,
    graphic: &BoardGraphic,
) -> Result<PreparedWrite, EngineError> {
    let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    let previous = board
        .get("graphics")
        .and_then(|v| v.get(graphic.uuid.to_string()))
        .ok_or_else(|| EngineError::Validation("missing graphic source preimage".into()))?;
    let mut payload = serde_json::to_value(graphic)?;
    if graphic.import_key.is_none()
        && let Some(key) = previous.get("import_key")
    {
        payload
            .as_object_mut()
            .expect("typed graphic is an object")
            .insert("import_key".into(), key.clone());
    }
    BatchComposer::compose(model, provenance)
        .push_op(Operation::SetBoardGraphic {
            board_id,
            graphic_id: graphic.uuid,
            graphic: payload,
        })
        .primary_object(graphic.uuid)
        .finish()
}

pub fn build_delete_board_graphic(
    model: &DesignModel,
    provenance: WriteProvenance,
    board_id: ObjectId,
    graphic_id: ObjectId,
) -> Result<PreparedWrite, EngineError> {
    let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    let graphic = board
        .get("graphics")
        .and_then(|v| v.get(graphic_id.to_string()))
        .cloned()
        .ok_or_else(|| EngineError::Validation("missing graphic deletion preimage".into()))?;
    BatchComposer::compose(model, provenance)
        .push_op(Operation::DeleteBoardGraphic {
            board_id,
            graphic_id,
            graphic,
            remove_map: false,
        })
        .primary_object(graphic_id)
        .finish()
}

#[cfg(test)]
mod tests;
