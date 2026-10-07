//! Pins name actual pool shards and journal incarnations, not display aliases.
use super::*;
use crate::substrate::{Operation, SourceShardKind, SourceShardTaxon};
use serde_json::Value;

pub(crate) fn hash_value(value: &Value) -> Result<String, EngineError> {
    Ok(crate::substrate::sha256_hex(
        crate::ir::serialization::to_json_deterministic(value)?.as_bytes(),
    ))
}
pub(crate) fn source_value(
    model: &DesignModel,
    pin: &PinnedPadLibrarySource,
) -> Result<Value, EngineError> {
    let shard = model
        .source_shards
        .iter()
        .find(|s| {
            s.kind == SourceShardKind::Pool
                && s.shard_id == pin.source_shard_id
                && s.relative_path == pin.relative_path
        })
        .ok_or_else(|| invalid("missing pinned pool shard"))?;
    let value = model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
    if value.get("uuid").and_then(Value::as_str) != Some(pin.owner_id.to_string().as_str()) {
        return Err(invalid("foreign pinned library owner"));
    }
    Ok(value)
}
fn record(value: &Value, id: Uuid) -> Option<&Value> {
    if value.get("uuid").and_then(Value::as_str) == Some(id.to_string().as_str()) {
        return Some(value);
    }
    value
        .get("pads")?
        .as_object()?
        .get(&id.to_string())
        .filter(|p| p.get("uuid").and_then(Value::as_str) == Some(id.to_string().as_str()))
}
fn incarnation(model: &DesignModel, path: &str, id: Uuid) -> Option<Uuid> {
    let mut birth = None;
    for t in &model.journal {
        for op in &t.operations {
            let added = match op {
                Operation::CreatePoolLibraryObject {
                    relative_path,
                    object,
                    ..
                } if relative_path == path => record(object, id).is_some(),
                Operation::CreatePoolPackage {
                    relative_path,
                    package,
                    ..
                } if relative_path == path => record(package, id).is_some(),
                Operation::CreatePoolPadstack {
                    relative_path,
                    padstack,
                    ..
                } if relative_path == path => record(padstack, id).is_some(),
                Operation::SetPoolLibraryObject {
                    relative_path,
                    previous_object,
                    object,
                    ..
                } if relative_path == path => {
                    record(previous_object, id).is_none() && record(object, id).is_some()
                }
                _ => false,
            };
            if added {
                birth = Some(t.transaction_id);
            }
        }
    }
    birth
}
fn capture(
    model: &DesignModel,
    path: &str,
    id: Uuid,
) -> Result<PinnedPadLibrarySource, EngineError> {
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.kind == SourceShardKind::Pool && s.relative_path == path)
        .ok_or_else(|| invalid("missing library source shard"))?;
    let value = model.materialized_source_shard_value_by_relative_path(path)?;
    let owner_id = serde_json::from_value(
        value
            .get("uuid")
            .cloned()
            .ok_or_else(|| invalid("missing library owner UUID"))?,
    )?;
    let source = record(&value, id)
        .ok_or_else(|| invalid("library object does not belong to pinned owner"))?;
    let mut historical_revision = source
        .get("object_revision")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    for t in &model.journal {
        for operation in &t.operations {
            if let Operation::SetPoolLibraryObject {
                relative_path,
                previous_object,
                object,
                ..
            } = operation
                && relative_path == path
            {
                match (record(previous_object, id), record(object, id)) {
                    (None, Some(_)) => historical_revision = 0,
                    (Some(old), Some(new)) if old != new || id == owner_id => {
                        historical_revision = historical_revision
                            .checked_add(1)
                            .ok_or_else(|| invalid("library revision overflow"))?
                    }
                    _ => {}
                }
            }
        }
    }
    if model
        .objects
        .get(&id)
        .is_some_and(|o| o.domain == "pool" && o.source_shard_id != shard.shard_id)
    {
        return Err(invalid(
            "ambiguous library UUID belongs to another source owner",
        ));
    }
    let object_revision = model
        .objects
        .get(&id)
        .filter(|o| o.domain == "pool" && o.source_shard_id == shard.shard_id)
        .map_or(ObjectRevision(historical_revision), |o| o.object_revision);
    Ok(PinnedPadLibrarySource {
        object_id: id,
        owner_id,
        source_shard_id: shard.shard_id,
        relative_path: path.into(),
        object_revision,
        owner_value_hash: hash_value(&value)?,
        record_hash: hash_value(source)?,
        incarnation: incarnation(model, path, id),
        journal_tip: model.journal.last().map(|t| t.transaction_id),
    })
}
pub(crate) fn capture_root(
    model: &DesignModel,
    id: Uuid,
    taxon: SourceShardTaxon,
) -> Result<PinnedPadLibrarySource, EngineError> {
    let mut matches = Vec::new();
    for shard in model
        .source_shards
        .iter()
        .filter(|s| s.kind == SourceShardKind::Pool && s.taxon == Some(taxon))
    {
        let value = model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
        if value.get("uuid").and_then(Value::as_str) == Some(id.to_string().as_str()) {
            matches.push(shard.relative_path.as_str());
        }
    }
    if matches.len() != 1 {
        return Err(invalid("library source owner is absent or ambiguous"));
    }
    capture(model, matches[0], id)
}
pub(crate) fn capture_pad(
    model: &DesignModel,
    owner: &PinnedPadLibrarySource,
    id: Uuid,
) -> Result<PinnedPadLibrarySource, EngineError> {
    capture(model, &owner.relative_path, id)
}
pub(crate) fn check_pin(
    model: &DesignModel,
    pin: &PinnedPadLibrarySource,
) -> Result<(), EngineError> {
    let now = capture(model, &pin.relative_path, pin.object_id)?;
    if now.owner_id != pin.owner_id
        || now.source_shard_id != pin.source_shard_id
        || now.object_revision != pin.object_revision
        || now.owner_value_hash != pin.owner_value_hash
        || now.record_hash != pin.record_hash
        || now.incarnation != pin.incarnation
    {
        return Err(invalid("stale library source pin"));
    }
    Ok(())
}

pub(crate) fn recorded_pin(
    model: &DesignModel,
    pin: &PinnedPadLibrarySource,
) -> Result<(), EngineError> {
    if check_pin(model, pin).is_ok() {
        return Ok(());
    }
    let start = pin
        .journal_tip
        .and_then(|id| model.journal.iter().position(|t| t.transaction_id == id))
        .ok_or_else(|| invalid("missing pinned history anchor"))?;
    for transaction in &model.journal[start + 1..] {
        for operation in &transaction.operations {
            let values: Vec<&Value> = match operation {
                Operation::SetPoolLibraryObject {
                    relative_path,
                    previous_object,
                    object,
                    ..
                } if relative_path == &pin.relative_path => vec![previous_object, object],
                Operation::DeletePoolLibraryObject {
                    relative_path,
                    object,
                    ..
                } if relative_path == &pin.relative_path => vec![object],
                Operation::DeletePoolPackage {
                    relative_path,
                    package,
                    ..
                } if relative_path == &pin.relative_path => vec![package],
                Operation::DeletePoolPadstack {
                    relative_path,
                    padstack,
                    ..
                } if relative_path == &pin.relative_path => vec![padstack],
                _ => vec![],
            };
            for value in values {
                if hash_value(value)? == pin.owner_value_hash
                    && record(value, pin.object_id)
                        .is_some_and(|r| hash_value(r).is_ok_and(|h| h == pin.record_hash))
                {
                    return Ok(());
                }
            }
        }
    }
    Err(invalid(
        "pinned source snapshot has no canonical history proof",
    ))
}
pub(crate) fn continuity(
    model: &DesignModel,
    old: &PinnedPadLibrarySource,
    new: &PinnedPadLibrarySource,
) -> Result<(), EngineError> {
    recorded_pin(model, old)?;
    check_pin(model, new)?;
    if old.object_id == new.object_id
        && (old.owner_id != new.owner_id || old.incarnation != new.incarnation)
    {
        return Err(invalid("library UUID rebirth is not Pad continuity"));
    }
    if old.owner_id == new.owner_id && old.object_id == new.object_id {
        let start = old
            .journal_tip
            .and_then(|id| model.journal.iter().position(|t| t.transaction_id == id))
            .ok_or_else(|| invalid("continuity needs a recorded source-history anchor"))?;
        for t in &model.journal[start + 1..] {
            for op in &t.operations {
                match op {
                    Operation::SetPoolLibraryObject {
                        relative_path,
                        previous_object,
                        object,
                        ..
                    } if relative_path == &old.relative_path => {
                        if record(previous_object, old.object_id).is_none()
                            || record(object, old.object_id).is_none()
                        {
                            return Err(invalid("library Pad did not survive source history"));
                        }
                    }
                    Operation::DeletePoolLibraryObject { relative_path, .. }
                    | Operation::DeletePoolPackage { relative_path, .. }
                        if relative_path == &old.relative_path =>
                    {
                        return Err(invalid("land-pattern retirement is not continuity"));
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
