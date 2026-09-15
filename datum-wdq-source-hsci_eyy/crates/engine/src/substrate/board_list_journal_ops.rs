use uuid::Uuid;

use super::{EngineError, Operation};

pub(super) fn apply_board_list_operation(
    board_value: &mut serde_json::Value,
    operation: &Operation,
) -> Result<Option<bool>, EngineError> {
    match operation {
        Operation::CreateBoardDimension {
            dimension_id,
            dimension,
        } => {
            insert_board_array_value(board_value, "dimensions", *dimension_id, dimension.clone())?;
            Ok(Some(true))
        }
        Operation::SetBoardDimension {
            dimension_id,
            dimension,
        } => {
            replace_board_array_value(board_value, "dimensions", *dimension_id, dimension.clone())?;
            Ok(Some(true))
        }
        Operation::DeleteBoardDimension { dimension_id, .. } => {
            remove_board_array_value(board_value, "dimensions", *dimension_id)?;
            Ok(Some(true))
        }
        Operation::CreateBoardText { text_id, text } => {
            insert_board_array_value(board_value, "texts", *text_id, text.clone())?;
            Ok(Some(true))
        }
        Operation::SetBoardText { text_id, text } => {
            replace_board_array_value(board_value, "texts", *text_id, text.clone())?;
            Ok(Some(true))
        }
        Operation::DeleteBoardText { text_id, .. } => {
            remove_board_array_value(board_value, "texts", *text_id)?;
            Ok(Some(true))
        }
        Operation::CreateBoardKeepout {
            keepout_id,
            keepout,
        } => {
            insert_board_array_value(board_value, "keepouts", *keepout_id, keepout.clone())?;
            Ok(Some(true))
        }
        Operation::SetBoardKeepout {
            keepout_id,
            keepout,
        } => {
            replace_board_array_value(board_value, "keepouts", *keepout_id, keepout.clone())?;
            Ok(Some(true))
        }
        Operation::DeleteBoardKeepout { keepout_id, .. } => {
            remove_board_array_value(board_value, "keepouts", *keepout_id)?;
            Ok(Some(true))
        }
        _ => Ok(None),
    }
}

pub(super) fn inverse_board_list_operation(
    board_value: &mut serde_json::Value,
    operation: &Operation,
    inverse_operations: &mut Vec<Operation>,
) -> Result<bool, EngineError> {
    match operation {
        Operation::CreateBoardDimension {
            dimension_id,
            dimension,
        } => {
            inverse_operations.push(Operation::DeleteBoardDimension {
                dimension_id: *dimension_id,
                dimension: dimension.clone(),
            });
            insert_board_array_value(board_value, "dimensions", *dimension_id, dimension.clone())?;
            Ok(true)
        }
        Operation::SetBoardDimension {
            dimension_id,
            dimension,
        } => {
            let previous = board_array_value(board_value, "dimensions", *dimension_id)?.clone();
            inverse_operations.push(Operation::SetBoardDimension {
                dimension_id: *dimension_id,
                dimension: previous,
            });
            replace_board_array_value(board_value, "dimensions", *dimension_id, dimension.clone())?;
            Ok(true)
        }
        Operation::DeleteBoardDimension { dimension_id, .. } => {
            let previous = board_array_value(board_value, "dimensions", *dimension_id)?.clone();
            inverse_operations.push(Operation::CreateBoardDimension {
                dimension_id: *dimension_id,
                dimension: previous,
            });
            remove_board_array_value(board_value, "dimensions", *dimension_id)?;
            Ok(true)
        }
        Operation::CreateBoardText { text_id, text } => {
            inverse_operations.push(Operation::DeleteBoardText {
                text_id: *text_id,
                text: text.clone(),
            });
            insert_board_array_value(board_value, "texts", *text_id, text.clone())?;
            Ok(true)
        }
        Operation::SetBoardText { text_id, text } => {
            let previous = board_array_value(board_value, "texts", *text_id)?.clone();
            inverse_operations.push(Operation::SetBoardText {
                text_id: *text_id,
                text: previous,
            });
            replace_board_array_value(board_value, "texts", *text_id, text.clone())?;
            Ok(true)
        }
        Operation::DeleteBoardText { text_id, .. } => {
            let previous = board_array_value(board_value, "texts", *text_id)?.clone();
            inverse_operations.push(Operation::CreateBoardText {
                text_id: *text_id,
                text: previous,
            });
            remove_board_array_value(board_value, "texts", *text_id)?;
            Ok(true)
        }
        Operation::CreateBoardKeepout {
            keepout_id,
            keepout,
        } => {
            inverse_operations.push(Operation::DeleteBoardKeepout {
                keepout_id: *keepout_id,
                keepout: keepout.clone(),
            });
            insert_board_array_value(board_value, "keepouts", *keepout_id, keepout.clone())?;
            Ok(true)
        }
        Operation::SetBoardKeepout {
            keepout_id,
            keepout,
        } => {
            let previous = board_array_value(board_value, "keepouts", *keepout_id)?.clone();
            inverse_operations.push(Operation::SetBoardKeepout {
                keepout_id: *keepout_id,
                keepout: previous,
            });
            replace_board_array_value(board_value, "keepouts", *keepout_id, keepout.clone())?;
            Ok(true)
        }
        Operation::DeleteBoardKeepout { keepout_id, .. } => {
            let previous = board_array_value(board_value, "keepouts", *keepout_id)?.clone();
            inverse_operations.push(Operation::CreateBoardKeepout {
                keepout_id: *keepout_id,
                keepout: previous,
            });
            remove_board_array_value(board_value, "keepouts", *keepout_id)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn board_array_value<'a>(
    board_value: &'a serde_json::Value,
    array_name: &str,
    object_id: Uuid,
) -> Result<&'a serde_json::Value, EngineError> {
    let index = board_array_index(board_value, array_name, object_id)?;
    Ok(&board_array(board_value, array_name)?[index])
}

fn insert_board_array_value(
    board_value: &mut serde_json::Value,
    array_name: &str,
    object_id: Uuid,
    value: serde_json::Value,
) -> Result<(), EngineError> {
    if board_array_index(board_value, array_name, object_id).is_ok() {
        return Ok(());
    }
    board_array_mut(board_value, array_name)?.push(value);
    Ok(())
}

fn replace_board_array_value(
    board_value: &mut serde_json::Value,
    array_name: &str,
    object_id: Uuid,
    value: serde_json::Value,
) -> Result<(), EngineError> {
    let index = board_array_index(board_value, array_name, object_id)?;
    board_array_mut(board_value, array_name)?[index] = value;
    Ok(())
}

fn remove_board_array_value(
    board_value: &mut serde_json::Value,
    array_name: &str,
    object_id: Uuid,
) -> Result<serde_json::Value, EngineError> {
    let index = board_array_index(board_value, array_name, object_id)?;
    Ok(board_array_mut(board_value, array_name)?.remove(index))
}

fn board_array_index(
    board_value: &serde_json::Value,
    array_name: &str,
    object_id: Uuid,
) -> Result<usize, EngineError> {
    board_array(board_value, array_name)?
        .iter()
        .position(|value| value_uuid(value) == Some(object_id))
        .ok_or(EngineError::NotFound {
            object_type: "board_array_object",
            uuid: object_id,
        })
}

fn board_array<'a>(
    board_value: &'a serde_json::Value,
    array_name: &str,
) -> Result<&'a Vec<serde_json::Value>, EngineError> {
    board_value
        .get(array_name)
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| EngineError::Validation(format!("board shard missing {array_name} array")))
}

fn board_array_mut<'a>(
    board_value: &'a mut serde_json::Value,
    array_name: &str,
) -> Result<&'a mut Vec<serde_json::Value>, EngineError> {
    board_value
        .get_mut(array_name)
        .and_then(serde_json::Value::as_array_mut)
        .ok_or_else(|| EngineError::Validation(format!("board shard missing {array_name} array")))
}

fn value_uuid(value: &serde_json::Value) -> Option<Uuid> {
    value
        .get("uuid")
        .and_then(serde_json::Value::as_str)
        .and_then(|value| Uuid::parse_str(value).ok())
}
