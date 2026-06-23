use super::{EngineError, Operation};

pub(super) fn apply_board_root_operation(
    board_value: &mut serde_json::Value,
    operation: &Operation,
) -> Result<Option<bool>, EngineError> {
    match operation {
        Operation::SetBoardOutline { outline, .. } => {
            set_board_root_field(board_value, "outline", outline.clone())?;
            Ok(Some(true))
        }
        Operation::SetBoardStackup { stackup, .. } => {
            set_board_root_field(board_value, "stackup", stackup.clone())?;
            Ok(Some(true))
        }
        Operation::SetBoardName { name, .. } => {
            set_board_root_field(board_value, "name", serde_json::Value::String(name.clone()))?;
            Ok(Some(true))
        }
        _ => Ok(None),
    }
}

pub(super) fn inverse_board_root_operation(
    board_value: &mut serde_json::Value,
    operation: &Operation,
    inverse_operations: &mut Vec<Operation>,
) -> Result<bool, EngineError> {
    match operation {
        Operation::SetBoardOutline { board_id, outline } => {
            let previous = board_root_field(board_value, "outline")?.clone();
            inverse_operations.push(Operation::SetBoardOutline {
                board_id: *board_id,
                outline: previous,
            });
            set_board_root_field(board_value, "outline", outline.clone())?;
            Ok(true)
        }
        Operation::SetBoardStackup { board_id, stackup } => {
            let previous = board_root_field(board_value, "stackup")?.clone();
            inverse_operations.push(Operation::SetBoardStackup {
                board_id: *board_id,
                stackup: previous,
            });
            set_board_root_field(board_value, "stackup", stackup.clone())?;
            Ok(true)
        }
        Operation::SetBoardName { board_id, name } => {
            let previous = board_root_field(board_value, "name")?
                .as_str()
                .ok_or_else(|| {
                    EngineError::Validation("board shard name field is not a string".to_string())
                })?
                .to_string();
            inverse_operations.push(Operation::SetBoardName {
                board_id: *board_id,
                name: previous,
            });
            set_board_root_field(board_value, "name", serde_json::Value::String(name.clone()))?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn board_root_field<'a>(
    board_value: &'a serde_json::Value,
    field: &str,
) -> Result<&'a serde_json::Value, EngineError> {
    board_value
        .get(field)
        .ok_or_else(|| EngineError::Validation(format!("board shard missing {field} field")))
}

fn set_board_root_field(
    board_value: &mut serde_json::Value,
    field: &str,
    value: serde_json::Value,
) -> Result<(), EngineError> {
    let object = board_value
        .as_object_mut()
        .ok_or_else(|| EngineError::Validation("board shard is not an object".to_string()))?;
    object.insert(field.to_string(), value);
    Ok(())
}
