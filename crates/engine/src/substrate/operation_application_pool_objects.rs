//! Keep canonical pool root/leaf object exposure identical to resolver loading.
use super::{
    CommitDiff, DomainObject, EngineError, ObjectId, ObjectRevision, SourceShardKind,
    SourceShardRef,
};
use serde_json::Value;
use std::{collections::BTreeMap, path::PathBuf};
use uuid::Uuid;

fn shard(path: &str) -> SourceShardRef {
    SourceShardRef {
        shard_id: Uuid::new_v5(
            &Uuid::NAMESPACE_URL,
            format!("datum-eda:source-shard:{path}").as_bytes(),
        ),
        kind: SourceShardKind::Pool,
        taxon: None,
        path: PathBuf::from(path),
        relative_path: path.into(),
        authority: super::SourceShardAuthority::AuthoredDesign,
        dirty_state: super::SourceShardDirtyState::Clean,
        schema_version: None,
        content_hash: String::new(),
    }
}
fn payload(
    id: Uuid,
    path: &str,
    value: &Value,
    kind: &str,
) -> Result<BTreeMap<Uuid, DomainObject>, EngineError> {
    let mut objects = BTreeMap::new();
    super::collect_uuid_objects(
        value,
        &shard(path),
        "pool",
        &mut objects,
        &mut BTreeMap::new(),
    );
    let root = objects
        .get_mut(&id)
        .ok_or_else(|| invalid("missing root UUID"))?;
    root.kind = kind.into();
    Ok(objects)
}
pub(super) fn create(
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    diff: Option<&mut CommitDiff>,
    id: Uuid,
    path: &str,
    value: &Value,
    kind: &str,
) -> Result<(), EngineError> {
    let additions = payload(id, path, value, kind)?;
    if additions.keys().any(|id| objects.contains_key(id)) {
        return Err(invalid("source UUID already exists"));
    }
    if let Some(diff) = diff {
        diff.created.extend(additions.keys());
    }
    objects.extend(additions);
    Ok(())
}
pub(super) fn set(
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    mut diff: Option<&mut CommitDiff>,
    id: Uuid,
    path: &str,
    previous: &Value,
    value: &Value,
    kind: &str,
) -> Result<(), EngineError> {
    let source = shard(path).shard_id;
    if objects.get(&id).is_none_or(|o| o.source_shard_id != source) {
        return Err(invalid("missing/foreign pool root"));
    }
    let mut replacements = payload(id, path, value, kind)?;
    if replacements
        .keys()
        .any(|id| objects.get(id).is_some_and(|o| o.source_shard_id != source))
    {
        return Err(invalid("foreign source UUID collision"));
    }
    let removed: Vec<_> = objects
        .iter()
        .filter(|(id, o)| o.source_shard_id == source && !replacements.contains_key(id))
        .map(|(id, _)| *id)
        .collect();
    for (next_id, next) in &mut replacements {
        if let Some(old) = objects.get(next_id) {
            let changed = *next_id == id || record(previous, *next_id) != record(value, *next_id);
            next.object_revision = if changed {
                ObjectRevision(
                    old.object_revision
                        .0
                        .checked_add(1)
                        .ok_or_else(|| invalid("revision overflow"))?,
                )
            } else {
                old.object_revision
            };
            if changed && let Some(diff) = diff.as_deref_mut() {
                diff.modified.push(*next_id);
            }
        } else if let Some(diff) = diff.as_deref_mut() {
            diff.created.push(*next_id);
        }
    }
    for id in removed {
        objects.remove(&id);
        if let Some(diff) = diff.as_deref_mut() {
            diff.deleted.push(id);
        }
    }
    objects.extend(replacements);
    Ok(())
}
pub(super) fn delete(
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    diff: Option<&mut CommitDiff>,
    id: Uuid,
) -> Result<(), EngineError> {
    let source = objects
        .get(&id)
        .ok_or_else(|| invalid("missing pool root"))?
        .source_shard_id;
    let ids: Vec<_> = objects
        .iter()
        .filter(|(_, o)| o.source_shard_id == source)
        .map(|(id, _)| *id)
        .collect();
    for id in &ids {
        objects.remove(id);
    }
    if let Some(diff) = diff {
        diff.deleted.extend(ids);
    }
    Ok(())
}
fn record(value: &Value, id: Uuid) -> Option<&Value> {
    if value.get("uuid").and_then(Value::as_str) == Some(id.to_string().as_str()) {
        return Some(value);
    }
    match value {
        Value::Object(o) => o.values().find_map(|v| record(v, id)),
        Value::Array(a) => a.iter().find_map(|v| record(v, id)),
        _ => None,
    }
}
fn invalid(reason: &str) -> EngineError {
    EngineError::Validation(format!("pool source objects: {reason}"))
}
