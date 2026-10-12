mod index;
pub(super) use index::BoardPass;
#[cfg(test)]
mod integration_tests;

use super::{EngineError, Operation};

pub(super) fn apply_board_list_operation(
    board_value: &mut serde_json::Value,
    operation: &Operation,
) -> Result<Option<bool>, EngineError> {
    apply_indexed(&mut BoardPass::new(board_value), operation)
}

fn apply_indexed(
    pass: &mut BoardPass<'_>,
    operation: &Operation,
) -> Result<Option<bool>, EngineError> {
    match operation {
        Operation::CreateBoardDimension {
            dimension_id,
            dimension,
        } => {
            pass.insert("dimensions", *dimension_id, dimension.clone())?;
            Ok(Some(true))
        }
        Operation::SetBoardDimension {
            dimension_id,
            dimension,
        } => {
            pass.replace("dimensions", *dimension_id, dimension.clone())?;
            Ok(Some(true))
        }
        Operation::DeleteBoardDimension { dimension_id, .. } => {
            pass.remove("dimensions", *dimension_id)?;
            Ok(Some(true))
        }
        Operation::CreateBoardText { text_id, text } => {
            pass.insert("texts", *text_id, text.clone())?;
            Ok(Some(true))
        }
        Operation::SetBoardText { text_id, text } => {
            pass.replace("texts", *text_id, text.clone())?;
            Ok(Some(true))
        }
        Operation::DeleteBoardText { text_id, .. } => {
            pass.remove("texts", *text_id)?;
            Ok(Some(true))
        }
        Operation::CreateBoardKeepout {
            keepout_id,
            keepout,
        } => {
            pass.insert("keepouts", *keepout_id, keepout.clone())?;
            Ok(Some(true))
        }
        Operation::SetBoardKeepout {
            keepout_id,
            keepout,
        } => {
            pass.replace("keepouts", *keepout_id, keepout.clone())?;
            Ok(Some(true))
        }
        Operation::DeleteBoardKeepout { keepout_id, .. } => {
            pass.remove("keepouts", *keepout_id)?;
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
    inverse_indexed(
        &mut BoardPass::new(board_value),
        operation,
        inverse_operations,
    )
}

fn inverse_indexed(
    pass: &mut BoardPass<'_>,
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
            pass.insert("dimensions", *dimension_id, dimension.clone())?;
            Ok(true)
        }
        Operation::SetBoardDimension {
            dimension_id,
            dimension,
        } => {
            let previous = pass.value("dimensions", *dimension_id)?.clone();
            inverse_operations.push(Operation::SetBoardDimension {
                dimension_id: *dimension_id,
                dimension: previous,
            });
            pass.replace("dimensions", *dimension_id, dimension.clone())?;
            Ok(true)
        }
        Operation::DeleteBoardDimension { dimension_id, .. } => {
            let previous = pass.value("dimensions", *dimension_id)?.clone();
            inverse_operations.push(Operation::CreateBoardDimension {
                dimension_id: *dimension_id,
                dimension: previous,
            });
            pass.remove("dimensions", *dimension_id)?;
            Ok(true)
        }
        Operation::CreateBoardText { text_id, text } => {
            inverse_operations.push(Operation::DeleteBoardText {
                text_id: *text_id,
                text: text.clone(),
            });
            pass.insert("texts", *text_id, text.clone())?;
            Ok(true)
        }
        Operation::SetBoardText { text_id, text } => {
            let previous = pass.value("texts", *text_id)?.clone();
            inverse_operations.push(Operation::SetBoardText {
                text_id: *text_id,
                text: previous,
            });
            pass.replace("texts", *text_id, text.clone())?;
            Ok(true)
        }
        Operation::DeleteBoardText { text_id, .. } => {
            let previous = pass.value("texts", *text_id)?.clone();
            inverse_operations.push(Operation::CreateBoardText {
                text_id: *text_id,
                text: previous,
            });
            pass.remove("texts", *text_id)?;
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
            pass.insert("keepouts", *keepout_id, keepout.clone())?;
            Ok(true)
        }
        Operation::SetBoardKeepout {
            keepout_id,
            keepout,
        } => {
            let previous = pass.value("keepouts", *keepout_id)?.clone();
            inverse_operations.push(Operation::SetBoardKeepout {
                keepout_id: *keepout_id,
                keepout: previous,
            });
            pass.replace("keepouts", *keepout_id, keepout.clone())?;
            Ok(true)
        }
        Operation::DeleteBoardKeepout { keepout_id, .. } => {
            let previous = pass.value("keepouts", *keepout_id)?.clone();
            inverse_operations.push(Operation::CreateBoardKeepout {
                keepout_id: *keepout_id,
                keepout: previous,
            });
            pass.remove("keepouts", *keepout_id)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}
