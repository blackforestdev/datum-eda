use std::path::Path;

use super::{
    DesignModel, EngineError, Operation, OperationBatch, SourceShardKind, TransactionRecord,
    journal::{StagedShardWrite, stage_new_shard_write},
};

pub(super) const FORWARD_ANNOTATION_REVIEW_RELATIVE_PATH: &str =
    ".datum/forward_annotation_review/review.json";

pub(super) fn maybe_stage_forward_annotation_review_operation(
    project_root: &Path,
    batch: &OperationBatch,
    operation: &Operation,
    staged: &mut Vec<StagedShardWrite>,
) -> Result<(), EngineError> {
    match operation {
        Operation::SetForwardAnnotationReview {
            relative_path,
            review,
            ..
        } => {
            validate_forward_annotation_review_path(relative_path)?;
            staged.push(stage_new_shard_write(
                project_root,
                batch,
                SourceShardKind::ForwardAnnotationReview,
                relative_path,
                review,
            )?);
        }
        Operation::DeleteForwardAnnotationReview { relative_path, .. } => {
            validate_forward_annotation_review_path(relative_path)?;
            staged.push(delete_forward_annotation_review_shard(
                project_root,
                relative_path,
            ));
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn apply_forward_annotation_review_shard_operation(
    shard_kind: &SourceShardKind,
    value: &mut serde_json::Value,
    operation: &Operation,
) -> Result<bool, EngineError> {
    if shard_kind != &SourceShardKind::ForwardAnnotationReview {
        return Ok(false);
    }
    match operation {
        Operation::SetForwardAnnotationReview {
            relative_path,
            review,
            ..
        } => {
            validate_forward_annotation_review_path(relative_path)?;
            *value = review.clone();
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub(super) fn apply_forward_annotation_review_model_operation(
    _model: &mut DesignModel,
    operation: &Operation,
) -> Result<bool, EngineError> {
    match operation {
        Operation::SetForwardAnnotationReview { relative_path, .. }
        | Operation::DeleteForwardAnnotationReview { relative_path, .. } => {
            validate_forward_annotation_review_path(relative_path)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub(super) fn inverse_forward_annotation_review_operation(
    operation: &Operation,
    inverse_operations: &mut Vec<Operation>,
) {
    match operation {
        Operation::SetForwardAnnotationReview {
            relative_path,
            previous_review,
            review,
        } => {
            if let Some(previous_review) = previous_review {
                inverse_operations.push(Operation::SetForwardAnnotationReview {
                    relative_path: relative_path.clone(),
                    previous_review: Some(review.clone()),
                    review: previous_review.clone(),
                });
            } else {
                inverse_operations.push(Operation::DeleteForwardAnnotationReview {
                    relative_path: relative_path.clone(),
                    review: review.clone(),
                });
            }
        }
        Operation::DeleteForwardAnnotationReview {
            relative_path,
            review,
        } => inverse_operations.push(Operation::SetForwardAnnotationReview {
            relative_path: relative_path.clone(),
            previous_review: None,
            review: review.clone(),
        }),
        _ => {}
    }
}

pub(super) fn forward_annotation_review_operation_write(
    operation: &Operation,
) -> Option<(&str, Option<&serde_json::Value>)> {
    match operation {
        Operation::SetForwardAnnotationReview {
            relative_path,
            review,
            ..
        } => Some((relative_path.as_str(), Some(review))),
        Operation::DeleteForwardAnnotationReview { relative_path, .. } => {
            Some((relative_path.as_str(), None))
        }
        _ => None,
    }
}

pub(super) fn reconstruct_forward_annotation_review_value(
    relative_path: &str,
    journal: &[TransactionRecord],
) -> Result<serde_json::Value, EngineError> {
    validate_forward_annotation_review_path(relative_path)?;
    let mut value = None;
    for transaction in journal {
        for operation in &transaction.operations {
            if let Some((operation_path, next_value)) =
                forward_annotation_review_operation_write(operation)
                && operation_path == relative_path {
                    value = next_value.cloned();
                }
        }
    }
    value.ok_or_else(|| {
        EngineError::Validation(format!(
            "cannot reconstruct missing forward-annotation review shard `{relative_path}`"
        ))
    })
}

fn validate_forward_annotation_review_path(relative_path: &str) -> Result<(), EngineError> {
    if relative_path == FORWARD_ANNOTATION_REVIEW_RELATIVE_PATH {
        Ok(())
    } else {
        Err(EngineError::Validation(format!(
            "forward-annotation review must use `{FORWARD_ANNOTATION_REVIEW_RELATIVE_PATH}`, got `{relative_path}`"
        )))
    }
}

fn delete_forward_annotation_review_shard(
    project_root: &Path,
    relative_path: &str,
) -> StagedShardWrite {
    StagedShardWrite {
        destination: project_root.join(relative_path),
        staged: None,
        kind: SourceShardKind::ForwardAnnotationReview,
        relative_path: relative_path.to_string(),
        content_hash: String::new(),
        schema_version: None,
        delete: true,
    }
}
