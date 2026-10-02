//! Explicit placed-pad connection facts on the existing canonical mutation path.
use super::super::board_json_maps::{
    board_map_value, insert_board_map_value, remove_board_map_value, replace_board_map_value,
};
use super::{EngineError, Operation};
use crate::board::{PadLayerConnection, PlacedPad};
pub(in crate::substrate) const BOARD_PAD_SCHEMA_VERSION: u64 = 3;
pub(in crate::substrate) fn validate(operation: &Operation) -> Result<(), EngineError> {
    match operation {
        Operation::CreateBoardPad { pad_id, pad } | Operation::SetBoardPad { pad_id, pad } => {
            validate_pad(pad, Some(*pad_id))
        }
        _ => Ok(()),
    }
}
fn known(value: &serde_json::Value) -> bool {
    value
        .get("layer_connection")
        .is_some_and(|v| v.get("kind").and_then(serde_json::Value::as_str) != Some("unknown"))
}
fn validate_pad(
    value: &serde_json::Value,
    expected: Option<uuid::Uuid>,
) -> Result<(), EngineError> {
    if !known(value) {
        return Ok(());
    }
    let pad: PlacedPad = serde_json::from_value(value.clone())?;
    if expected.is_some_and(|id| id != pad.uuid) || pad.uuid.is_nil() {
        return Err(EngineError::Validation(
            "Pad source UUID differs from operation identity".into(),
        ));
    }
    match pad.layer_connection {
        PadLayerConnection::PlatedThrough | PadLayerConnection::PlatedSpan { .. }
            if pad.drill <= 0 =>
        {
            return Err(EngineError::Validation(
                "plated Pad requires a positive authored drill".into(),
            ));
        }
        PadLayerConnection::PlatedSpan {
            start_layer,
            end_layer,
        } if start_layer == end_layer => {
            return Err(EngineError::Validation(
                "plated Pad span endpoints must differ".into(),
            ));
        }
        _ => {}
    }
    if pad.drill < 0 {
        return Err(EngineError::Validation(
            "Pad drill must not be negative".into(),
        ));
    }
    Ok(())
}
pub(in crate::substrate) fn validate_native(value: &serde_json::Value) -> Result<(), EngineError> {
    if let Some(pads) = value.get("pads").and_then(serde_json::Value::as_object) {
        for (key, pad) in pads {
            if !known(pad) {
                continue;
            }
            if value
                .get("schema_version")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(1)
                < BOARD_PAD_SCHEMA_VERSION
            {
                return Err(EngineError::Validation(
                    "unsupported BoardRoot schema_version declaration: explicit Pad layer connection requires schema_version 3".into(),
                ));
            }
            let id =
                uuid::Uuid::parse_str(key).map_err(|e| EngineError::Validation(e.to_string()))?;
            validate_pad(pad, Some(id))?;
        }
    }
    Ok(())
}
pub(in crate::substrate) fn stamp_schema(value: &mut serde_json::Value) {
    if value
        .get("pads")
        .and_then(serde_json::Value::as_object)
        .is_some_and(|m| m.values().any(known))
    {
        let previous = value
            .get("schema_version")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(1);
        value["schema_version"] = serde_json::json!(previous.max(BOARD_PAD_SCHEMA_VERSION));
    }
}
pub(in crate::substrate) fn apply(
    value: &mut serde_json::Value,
    operation: &Operation,
) -> Result<Option<bool>, EngineError> {
    let board_value = value;
    match operation {
        Operation::CreateBoardPad { pad_id, pad } => {
            insert_board_map_value(board_value, "pads", *pad_id, pad.clone())?;
        }
        Operation::SetBoardPad { pad_id, pad } => {
            replace_board_map_value(board_value, "pads", *pad_id, pad.clone())?;
        }
        Operation::DeleteBoardPad { pad_id, .. } => {
            remove_board_map_value(board_value, "pads", *pad_id)?;
        }
        _ => return Ok(None),
    };
    stamp_schema(board_value);
    Ok(Some(true))
}
pub(in crate::substrate) fn inverse(
    board_value: &mut serde_json::Value,
    operation: &Operation,
    inverse_operations: &mut Vec<Operation>,
) -> Result<bool, EngineError> {
    match operation {
        Operation::CreateBoardPad { pad_id, pad } => {
            inverse_operations.push(Operation::DeleteBoardPad {
                pad_id: *pad_id,
                pad: pad.clone(),
            });
            insert_board_map_value(board_value, "pads", *pad_id, pad.clone())?;
        }
        Operation::SetBoardPad { pad_id, pad } => {
            let previous = board_map_value(board_value, "pads", *pad_id)?.clone();
            inverse_operations.push(Operation::SetBoardPad {
                pad_id: *pad_id,
                pad: previous,
            });
            replace_board_map_value(board_value, "pads", *pad_id, pad.clone())?;
        }
        Operation::DeleteBoardPad { pad_id, .. } => {
            let previous = board_map_value(board_value, "pads", *pad_id)?.clone();
            inverse_operations.push(Operation::CreateBoardPad {
                pad_id: *pad_id,
                pad: previous,
            });
            remove_board_map_value(board_value, "pads", *pad_id)?;
        }
        _ => return Ok(false),
    }
    stamp_schema(board_value);
    Ok(true)
}
