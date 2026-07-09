//! Object-revision guard insertion for native write batches.
//!
//! Moved from `crates/cli/src/command_project_operation_guards.rs` (the CLI
//! file is now a thin shim over this module). Every operation that mutates an
//! existing object gets a preceding `Operation::GuardObjectRevision` stamped
//! with the object's current revision, so a concurrent mutation between
//! resolve and commit is rejected instead of silently clobbered. Guard
//! ordering is load-bearing: each guard precedes the first operation that
//! targets its object, and an object is guarded at most once per batch.

use std::collections::BTreeSet;

use crate::error::EngineError;
use crate::substrate::{DesignModel, ObjectId, ObjectRevision, Operation, OperationBatch};

/// Insert revision guards for every existing-object mutation in `batch`.
///
/// Operations that already carry an explicit `GuardObjectRevision` for an
/// object suppress the automatic guard for that object. Each guarded object
/// receives exactly one guard, inserted immediately before the first
/// operation that targets it.
pub fn guarded_operation_batch(
    model: &DesignModel,
    mut batch: OperationBatch,
) -> Result<OperationBatch, EngineError> {
    let mut guarded_objects = batch
        .operations
        .iter()
        .filter_map(|operation| match operation {
            Operation::GuardObjectRevision { object_id, .. } => Some(*object_id),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let mut guarded = Vec::with_capacity(batch.operations.len());
    for operation in batch.operations {
        if let Some(object_id) = existing_object_guard_target(&operation) {
            if guarded_objects.insert(object_id) {
                guarded.push(object_revision_guard(model, object_id)?);
            }
        }
        guarded.push(operation);
    }
    batch.operations = guarded;
    Ok(batch)
}

/// Wrap a single operation with its revision guard when it mutates an
/// existing object; pass creation-style operations through unchanged.
pub fn guarded_existing_object_operation(
    model: &DesignModel,
    operation: Operation,
) -> Result<Vec<Operation>, EngineError> {
    if let Some(object_id) = existing_object_guard_target(&operation) {
        guarded_object_operations(model, object_id, vec![operation])
    } else {
        Ok(vec![operation])
    }
}

/// Prepend a revision guard for `object_id` to `operations`.
pub fn guarded_object_operations(
    model: &DesignModel,
    object_id: ObjectId,
    mut operations: Vec<Operation>,
) -> Result<Vec<Operation>, EngineError> {
    let object = model.objects.get(&object_id).ok_or(EngineError::NotFound {
        object_type: "domain_object",
        uuid: object_id,
    })?;
    operations.insert(
        0,
        object_revision_guard_with_revision(object_id, object.object_revision),
    );
    Ok(operations)
}

fn object_revision_guard(
    model: &DesignModel,
    object_id: ObjectId,
) -> Result<Operation, EngineError> {
    let object = model.objects.get(&object_id).ok_or(EngineError::NotFound {
        object_type: "domain_object",
        uuid: object_id,
    })?;
    Ok(object_revision_guard_with_revision(
        object_id,
        object.object_revision,
    ))
}

fn object_revision_guard_with_revision(
    object_id: ObjectId,
    expected_object_revision: ObjectRevision,
) -> Operation {
    Operation::GuardObjectRevision {
        object_id,
        expected_object_revision,
    }
}

fn existing_object_guard_target(operation: &Operation) -> Option<ObjectId> {
    match operation {
        Operation::DeleteBoardPackage { package_id, .. }
        | Operation::SetBoardPackagePart { package_id, .. }
        | Operation::SetBoardPackagePackage { package_id, .. }
        | Operation::SetBoardPackageValue { package_id, .. }
        | Operation::SetBoardPackageReference { package_id, .. }
        | Operation::SetBoardPackagePosition { package_id, .. }
        | Operation::SetBoardPackageLayer { package_id, .. }
        | Operation::SetComponentSide { package_id, .. }
        | Operation::SetBoardPackageRotation { package_id, .. }
        | Operation::SetBoardPackageLocked { package_id, .. } => Some(*package_id),
        Operation::SetBoardPad { pad_id, .. } | Operation::DeleteBoardPad { pad_id, .. } => {
            Some(*pad_id)
        }
        Operation::SetBoardTrack { track_id, .. }
        | Operation::DeleteBoardTrack { track_id, .. } => Some(*track_id),
        Operation::SetBoardVia { via_id, .. } | Operation::DeleteBoardVia { via_id, .. } => {
            Some(*via_id)
        }
        Operation::SetBoardZone { zone_id, .. } | Operation::DeleteBoardZone { zone_id, .. } => {
            Some(*zone_id)
        }
        Operation::SetBoardNet { net_id, .. } | Operation::DeleteBoardNet { net_id, .. } => {
            Some(*net_id)
        }
        Operation::SetBoardNetClass { net_class_id, .. }
        | Operation::DeleteBoardNetClass { net_class_id, .. } => Some(*net_class_id),
        Operation::SetBoardDimension { dimension_id, .. }
        | Operation::DeleteBoardDimension { dimension_id, .. } => Some(*dimension_id),
        Operation::SetBoardText { text_id, .. } | Operation::DeleteBoardText { text_id, .. } => {
            Some(*text_id)
        }
        Operation::SetBoardKeepout { keepout_id, .. }
        | Operation::DeleteBoardKeepout { keepout_id, .. } => Some(*keepout_id),
        Operation::SetBoardOutline { board_id, .. }
        | Operation::SetBoardStackup { board_id, .. }
        | Operation::SetBoardName { board_id, .. } => Some(*board_id),
        Operation::SetSchematicSheetName { sheet_id, .. } => Some(*sheet_id),
        Operation::SetSchematicLabel { label_id, .. }
        | Operation::DeleteSchematicLabel { label_id, .. } => Some(*label_id),
        Operation::SetSchematicPort { port_id, .. }
        | Operation::DeleteSchematicPort { port_id, .. } => Some(*port_id),
        Operation::SetSchematicBus { bus_id, .. }
        | Operation::DeleteSchematicBus { bus_id, .. } => Some(*bus_id),
        Operation::DeleteSchematicBusEntry { bus_entry_id, .. } => Some(*bus_entry_id),
        Operation::SetSchematicText { text_id, .. }
        | Operation::DeleteSchematicText { text_id, .. } => Some(*text_id),
        Operation::SetSchematicDrawing { drawing_id, .. }
        | Operation::DeleteSchematicDrawing { drawing_id, .. } => Some(*drawing_id),
        Operation::SetSchematicSymbol { symbol_id, .. }
        | Operation::DeleteSchematicSymbol { symbol_id, .. } => Some(*symbol_id),
        Operation::SetComponentInstance {
            component_instance_id,
            ..
        }
        | Operation::DeleteComponentInstance {
            component_instance_id,
            ..
        } => Some(*component_instance_id),
        Operation::SetProjectName { project_id, .. } => Some(*project_id),
        Operation::SetProjectRules { rules_root_id, .. } => Some(*rules_root_id),
        Operation::SetProjectRule { rules_root_id, .. }
        | Operation::DeleteProjectRule { rules_root_id, .. } => Some(*rules_root_id),
        Operation::SetManufacturingPlan {
            manufacturing_plan_id,
            ..
        }
        | Operation::DeleteManufacturingPlan {
            manufacturing_plan_id,
            ..
        } => Some(*manufacturing_plan_id),
        Operation::SetPanelProjection {
            panel_projection_id,
            ..
        }
        | Operation::DeletePanelProjection {
            panel_projection_id,
            ..
        } => Some(*panel_projection_id),
        Operation::SetOutputJob { output_job_id, .. }
        | Operation::DeleteOutputJob { output_job_id, .. } => Some(*output_job_id),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::super::test_support::resolved_model_with_board_package;
    use super::*;
    use crate::substrate::{CommitProvenance, CommitSource, ObjectRevision};

    fn test_batch(model: &DesignModel, operations: Vec<Operation>) -> OperationBatch {
        OperationBatch {
            batch_id: Uuid::new_v4(),
            expected_model_revision: Some(model.model_revision.clone()),
            provenance: CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "guard insertion test".to_string(),
            },
            operations,
        }
    }

    #[test]
    fn guarded_operation_batch_inserts_guard_before_existing_object_mutation() {
        let (_root, model, _board_id, package_id) =
            resolved_model_with_board_package("guards_insert");
        let batch = test_batch(
            &model,
            vec![Operation::SetBoardPackageValue {
                package_id,
                value: "NEW".to_string(),
            }],
        );

        let guarded = guarded_operation_batch(&model, batch).expect("guard insertion");

        assert_eq!(
            guarded.operations,
            vec![
                Operation::GuardObjectRevision {
                    object_id: package_id,
                    expected_object_revision: ObjectRevision(0),
                },
                Operation::SetBoardPackageValue {
                    package_id,
                    value: "NEW".to_string(),
                },
            ]
        );
    }

    #[test]
    fn guarded_operation_batch_guards_each_object_once() {
        let (_root, model, _board_id, package_id) =
            resolved_model_with_board_package("guards_dedup");
        let batch = test_batch(
            &model,
            vec![
                Operation::SetBoardPackageValue {
                    package_id,
                    value: "A".to_string(),
                },
                Operation::SetBoardPackageReference {
                    package_id,
                    reference: "U9".to_string(),
                },
            ],
        );

        let guarded = guarded_operation_batch(&model, batch).expect("guard insertion");

        let guard_count = guarded
            .operations
            .iter()
            .filter(|operation| matches!(operation, Operation::GuardObjectRevision { .. }))
            .count();
        assert_eq!(guard_count, 1);
        assert!(matches!(
            guarded.operations[0],
            Operation::GuardObjectRevision { object_id, .. } if object_id == package_id
        ));
    }

    #[test]
    fn guarded_operation_batch_respects_explicit_guards() {
        let (_root, model, _board_id, package_id) =
            resolved_model_with_board_package("guards_explicit");
        let explicit = Operation::GuardObjectRevision {
            object_id: package_id,
            expected_object_revision: ObjectRevision(0),
        };
        let batch = test_batch(
            &model,
            vec![
                explicit.clone(),
                Operation::SetBoardPackageValue {
                    package_id,
                    value: "NEW".to_string(),
                },
            ],
        );

        let guarded = guarded_operation_batch(&model, batch).expect("guard insertion");

        let guard_count = guarded
            .operations
            .iter()
            .filter(|operation| matches!(operation, Operation::GuardObjectRevision { .. }))
            .count();
        assert_eq!(guard_count, 1);
        assert_eq!(guarded.operations[0], explicit);
    }

    #[test]
    fn guarded_operation_batch_leaves_creation_operations_unguarded() {
        let (_root, model, board_id, _package_id) =
            resolved_model_with_board_package("guards_creation");
        let batch = test_batch(
            &model,
            vec![Operation::BumpObjectRevision {
                object_id: board_id,
            }],
        );

        let guarded = guarded_operation_batch(&model, batch).expect("guard insertion");

        assert_eq!(guarded.operations.len(), 1);
        assert!(matches!(
            guarded.operations[0],
            Operation::BumpObjectRevision { .. }
        ));
    }

    #[test]
    fn guarded_existing_object_operation_wraps_mutation() {
        let (_root, model, _board_id, package_id) =
            resolved_model_with_board_package("guards_wrap");
        let operations = guarded_existing_object_operation(
            &model,
            Operation::SetBoardPackageValue {
                package_id,
                value: "NEW".to_string(),
            },
        )
        .expect("guard insertion");

        assert_eq!(operations.len(), 2);
        assert_eq!(
            operations[0],
            Operation::GuardObjectRevision {
                object_id: package_id,
                expected_object_revision: ObjectRevision(0),
            }
        );
    }

    #[test]
    fn guarded_object_operations_rejects_unknown_object() {
        let (_root, model, _board_id, _package_id) =
            resolved_model_with_board_package("guards_unknown");
        let missing = Uuid::new_v4();
        let error = guarded_object_operations(&model, missing, Vec::new())
            .expect_err("unknown object should fail");
        assert!(matches!(
            error,
            EngineError::NotFound {
                object_type: "domain_object",
                uuid,
            } if uuid == missing
        ));
    }
}
