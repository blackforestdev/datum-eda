use anyhow::Result;
use std::collections::BTreeSet;

use eda_engine::substrate::{DesignModel, ObjectId, Operation, OperationBatch};

pub(crate) fn guarded_operation_batch(
    model: &DesignModel,
    mut batch: OperationBatch,
) -> Result<OperationBatch> {
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

pub(crate) fn guarded_existing_object_operation(
    model: &DesignModel,
    operation: Operation,
) -> Result<Vec<Operation>> {
    if let Some(object_id) = existing_object_guard_target(&operation) {
        guarded_object_operations(model, object_id, vec![operation])
    } else {
        Ok(vec![operation])
    }
}

pub(crate) fn guarded_object_operations(
    model: &DesignModel,
    object_id: ObjectId,
    mut operations: Vec<Operation>,
) -> Result<Vec<Operation>> {
    let object = model.objects.get(&object_id).ok_or_else(|| {
        anyhow::anyhow!("domain object not found for revision guard: {object_id}")
    })?;
    operations.insert(
        0,
        object_revision_guard_with_revision(object_id, object.object_revision),
    );
    Ok(operations)
}

fn object_revision_guard(model: &DesignModel, object_id: ObjectId) -> Result<Operation> {
    let object = model.objects.get(&object_id).ok_or_else(|| {
        anyhow::anyhow!("domain object not found for revision guard: {object_id}")
    })?;
    Ok(object_revision_guard_with_revision(
        object_id,
        object.object_revision,
    ))
}

fn object_revision_guard_with_revision(
    object_id: ObjectId,
    expected_object_revision: eda_engine::substrate::ObjectRevision,
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
