use super::{
    CommitDiff, DesignModel, DomainObject, EngineError, ObjectId, ObjectRevision, Operation,
};

pub(super) fn apply_schematic_disposition_operation(
    model: &mut DesignModel,
    diff: &mut CommitDiff,
    operation: &Operation,
) -> Result<bool, EngineError> {
    match operation {
        Operation::CreateSchematicWaiver {
            schematic_id,
            waiver_id,
            waiver,
        } => apply_schematic_waiver_create(model, diff, *schematic_id, *waiver_id, waiver)?,
        Operation::DeleteSchematicWaiver {
            schematic_id,
            waiver_id,
            ..
        } => apply_schematic_waiver_delete(model, diff, *schematic_id, *waiver_id)?,
        Operation::CreateSchematicDeviation {
            schematic_id,
            deviation_id,
            deviation,
        } => {
            apply_schematic_deviation_create(model, diff, *schematic_id, *deviation_id, deviation)?
        }
        Operation::DeleteSchematicDeviation {
            schematic_id,
            deviation_id,
            ..
        } => apply_schematic_deviation_delete(model, diff, *schematic_id, *deviation_id)?,
        _ => return Ok(false),
    }
    Ok(true)
}

pub(super) fn apply_schematic_waiver_create(
    model: &mut DesignModel,
    diff: &mut CommitDiff,
    schematic_id: ObjectId,
    waiver_id: ObjectId,
    waiver: &serde_json::Value,
) -> Result<(), EngineError> {
    let schematic_root = model
        .objects
        .get(&schematic_id)
        .ok_or(EngineError::NotFound {
            object_type: "schematic_root",
            uuid: schematic_id,
        })?;
    let source_shard_id = schematic_root.source_shard_id;
    if model.objects.contains_key(&waiver_id) {
        return Err(EngineError::Validation(format!(
            "schematic waiver {waiver_id} already exists"
        )));
    }
    model.objects.insert(
        waiver_id,
        DomainObject {
            object_id: waiver_id,
            object_revision: ObjectRevision(
                waiver
                    .get("object_revision")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0),
            ),
            source_shard_id,
            domain: "schematic".to_string(),
            kind: "waivers".to_string(),
        },
    );
    diff.created.push(waiver_id);
    Ok(())
}

pub(super) fn apply_schematic_waiver_delete(
    model: &mut DesignModel,
    diff: &mut CommitDiff,
    schematic_id: ObjectId,
    waiver_id: ObjectId,
) -> Result<(), EngineError> {
    model
        .objects
        .get(&schematic_id)
        .ok_or(EngineError::NotFound {
            object_type: "schematic_root",
            uuid: schematic_id,
        })?;
    if model.objects.remove(&waiver_id).is_some() {
        diff.deleted.push(waiver_id);
        Ok(())
    } else {
        Err(EngineError::NotFound {
            object_type: "schematic_waiver",
            uuid: waiver_id,
        })
    }
}

pub(super) fn apply_schematic_deviation_create(
    model: &mut DesignModel,
    diff: &mut CommitDiff,
    schematic_id: ObjectId,
    deviation_id: ObjectId,
    deviation: &serde_json::Value,
) -> Result<(), EngineError> {
    let schematic_root = model
        .objects
        .get(&schematic_id)
        .ok_or(EngineError::NotFound {
            object_type: "schematic_root",
            uuid: schematic_id,
        })?;
    let source_shard_id = schematic_root.source_shard_id;
    if model.objects.contains_key(&deviation_id) {
        return Err(EngineError::Validation(format!(
            "schematic deviation {deviation_id} already exists"
        )));
    }
    model.objects.insert(
        deviation_id,
        DomainObject {
            object_id: deviation_id,
            object_revision: ObjectRevision(
                deviation
                    .get("object_revision")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0),
            ),
            source_shard_id,
            domain: "schematic".to_string(),
            kind: "deviations".to_string(),
        },
    );
    diff.created.push(deviation_id);
    Ok(())
}

pub(super) fn apply_schematic_deviation_delete(
    model: &mut DesignModel,
    diff: &mut CommitDiff,
    schematic_id: ObjectId,
    deviation_id: ObjectId,
) -> Result<(), EngineError> {
    model
        .objects
        .get(&schematic_id)
        .ok_or(EngineError::NotFound {
            object_type: "schematic_root",
            uuid: schematic_id,
        })?;
    if model.objects.remove(&deviation_id).is_some() {
        diff.deleted.push(deviation_id);
        Ok(())
    } else {
        Err(EngineError::NotFound {
            object_type: "schematic_deviation",
            uuid: deviation_id,
        })
    }
}
