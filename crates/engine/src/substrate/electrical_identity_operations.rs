//! Typed electrical source operations. Cross-record validation happens at the
//! final batch state, allowing an atomic semantic redistribution in either order.
use super::{CommitDiff, DesignModel, DomainObject, EngineError, ObjectId, Operation};
use std::collections::BTreeMap;

pub(super) fn apply(
    model: &mut DesignModel,
    op: &Operation,
    diff: &mut CommitDiff,
) -> Result<bool, EngineError> {
    let Some((id, record, delete)) = super::electrical_identity_store::write(op) else {
        return Ok(false);
    };
    match op {
        Operation::CreateElectricalIdentity { .. } if model.objects.contains_key(&id) => {
            return Err(invalid("identity already exists"));
        }
        Operation::SetElectricalIdentity { previous, .. } => {
            if previous.id != id
                || model.electrical_identities.get(&id) != Some(previous.as_ref())
                || std::mem::discriminant(&previous.identity)
                    != std::mem::discriminant(&record.identity)
            {
                return Err(invalid("stale preimage or identity kind change"));
            }
        }
        Operation::DeleteElectricalIdentity { .. }
            if model.electrical_identities.get(&id) != Some(record) =>
        {
            return Err(invalid("stale delete preimage"));
        }
        _ => {}
    }
    apply_objects(&mut model.objects, Some(diff), op)?;
    if delete {
        model.electrical_identities.remove(&id);
        let path = super::electrical_identity_store::relative_path(id);
        model.source_shards.retain(|s| s.relative_path != path);
    } else {
        model.electrical_identities.insert(id, record.clone());
        super::electrical_identity_store::insert_object(model, record);
    }
    Ok(true)
}

pub(super) fn apply_objects(
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    diff: Option<&mut CommitDiff>,
    op: &Operation,
) -> Result<(), EngineError> {
    let Some((id, record, delete)) = super::electrical_identity_store::write(op) else {
        return Ok(());
    };
    if delete {
        objects.remove(&id);
        if let Some(d) = diff {
            d.deleted.push(id);
        }
    } else {
        let path = super::electrical_identity_store::relative_path(id);
        let shard_id = uuid::Uuid::new_v5(
            &uuid::Uuid::NAMESPACE_URL,
            format!("datum-eda:source-shard:{path}").as_bytes(),
        );
        let existed = objects
            .insert(
                id,
                super::electrical_identity_store::object(record, shard_id),
            )
            .is_some();
        if let Some(d) = diff {
            if existed {
                d.modified.push(id);
            } else {
                d.created.push(id);
            }
        }
    }
    Ok(())
}

fn invalid(reason: &str) -> EngineError {
    EngineError::Validation(format!("electrical identity write refused: {reason}"))
}
