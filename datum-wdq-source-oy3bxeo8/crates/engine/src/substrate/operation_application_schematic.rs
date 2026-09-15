use std::collections::BTreeMap;

use super::{
    CommitDiff, DesignModel, DomainObject, EngineError, ObjectId, SourceShardKind, SourceShardRef,
    collect_uuid_objects,
};

pub(super) fn apply_schematic_map_create(
    model: &mut DesignModel,
    diff: &mut CommitDiff,
    sheet_id: ObjectId,
    map_name: &str,
    object_id: ObjectId,
    value: &serde_json::Value,
) -> Result<(), EngineError> {
    let created = schematic_map_payload_objects(model, sheet_id, map_name, object_id, value)?;
    for object_id in created.keys() {
        diff.created.push(*object_id);
    }
    model.objects.extend(created);
    Ok(())
}

pub(super) fn apply_schematic_map_delete(
    model: &mut DesignModel,
    diff: &mut CommitDiff,
    sheet_id: ObjectId,
    map_name: &str,
    object_id: ObjectId,
    value: &serde_json::Value,
) -> Result<(), EngineError> {
    let deleted = schematic_map_payload_objects(model, sheet_id, map_name, object_id, value)?;
    for object_id in deleted.keys() {
        if model.objects.remove(object_id).is_some() {
            diff.deleted.push(*object_id);
        }
    }
    Ok(())
}

pub(super) fn apply_schematic_map_set(
    model: &mut DesignModel,
    diff: &mut CommitDiff,
    sheet_id: ObjectId,
    map_name: &str,
    object_id: ObjectId,
    value: &serde_json::Value,
) -> Result<(), EngineError> {
    let old = existing_schematic_map_payload_objects(model, sheet_id, map_name, object_id)?;
    let new = schematic_map_payload_objects(model, sheet_id, map_name, object_id, value)?;

    for old_id in old.keys() {
        if !new.contains_key(old_id) && model.objects.remove(old_id).is_some() {
            diff.deleted.push(*old_id);
        }
    }
    for (new_id, object) in new {
        match model.objects.get_mut(&new_id) {
            Some(existing) => {
                if new_id == object_id {
                    existing.object_revision =
                        super::ObjectRevision(existing.object_revision.0 + 1);
                    diff.modified.push(new_id);
                }
            }
            None => {
                diff.created.push(new_id);
                model.objects.insert(new_id, object);
            }
        }
    }
    Ok(())
}

fn existing_schematic_map_payload_objects(
    model: &DesignModel,
    sheet_id: ObjectId,
    map_name: &str,
    object_id: ObjectId,
) -> Result<BTreeMap<ObjectId, DomainObject>, EngineError> {
    let sheet_shard = schematic_sheet_shard(model, sheet_id)?;
    let relative_path = sheet_shard.relative_path.clone();
    let sheet_value = model.materialized_source_shard_value_by_relative_path(&relative_path)?;
    if let Some(value) = sheet_value
        .get(map_name)
        .and_then(serde_json::Value::as_object)
        .and_then(|values| values.get(&object_id.to_string()))
    {
        schematic_map_payload_objects(model, sheet_id, map_name, object_id, value)
    } else {
        Ok(BTreeMap::new())
    }
}

fn schematic_map_payload_objects(
    model: &DesignModel,
    sheet_id: ObjectId,
    map_name: &str,
    object_id: ObjectId,
    value: &serde_json::Value,
) -> Result<BTreeMap<ObjectId, DomainObject>, EngineError> {
    let sheet_shard = schematic_sheet_shard(model, sheet_id)?;
    let fragment = serde_json::json!({ map_name: { object_id.to_string(): value } });
    let mut objects = BTreeMap::new();
    let mut import_map = BTreeMap::new();
    collect_uuid_objects(
        &fragment,
        sheet_shard,
        "schematic",
        &mut objects,
        &mut import_map,
    );
    Ok(objects)
}

fn schematic_sheet_shard(
    model: &DesignModel,
    sheet_id: ObjectId,
) -> Result<&SourceShardRef, EngineError> {
    for shard in model
        .source_shards
        .iter()
        .filter(|shard| shard.kind == SourceShardKind::SchematicSheet)
    {
        let value = model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
        if value.get("uuid").and_then(serde_json::Value::as_str)
            == Some(sheet_id.to_string().as_str())
        {
            return Ok(shard);
        }
    }
    Err(EngineError::NotFound {
        object_type: "schematic_sheet",
        uuid: sheet_id,
    })
}
