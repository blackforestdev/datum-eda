//! Canonical PM057 graphics, including explicit map presence in history.
use super::{CommitDiff, DesignModel, DomainObject, EngineError, ObjectRevision, Operation};
use crate::board::graphics::BoardGraphic;
use serde_json::Value;
use uuid::Uuid;

fn refusal(message: &str) -> EngineError {
    EngineError::Validation(format!("BoardGraphic source: {message}"))
}

pub(super) fn identity(op: &Operation) -> Option<(Uuid, Uuid, &Value)> {
    match op {
        Operation::CreateBoardGraphic {
            board_id,
            graphic_id,
            graphic,
            ..
        }
        | Operation::SetBoardGraphic {
            board_id,
            graphic_id,
            graphic,
        }
        | Operation::DeleteBoardGraphic {
            board_id,
            graphic_id,
            graphic,
            ..
        } => Some((*board_id, *graphic_id, graphic)),
        _ => None,
    }
}

fn validate_payload(value: &Value, id: Uuid, board: &Value) -> Result<(), EngineError> {
    let graphic: BoardGraphic = serde_json::from_value(value.clone())?;
    if graphic.uuid != id {
        return Err(refusal("map/operation UUID mismatch"));
    }
    graphic
        .validate()
        .map_err(|e| refusal(&format!("invalid geometry: {e:?}")))?;
    let layers = board
        .pointer("/stackup/layers")
        .and_then(Value::as_array)
        .ok_or_else(|| refusal("missing documentation layer declaration"))?;
    let matching: Vec<_> = layers
        .iter()
        .filter(|v| v["id"].as_i64() == Some(graphic.layer.into()))
        .collect();
    if matching.len() != 1
        || !matches!(
            matching[0]["layer_type"].as_str(),
            Some("Silkscreen" | "Mechanical")
        )
    {
        return Err(refusal(
            "layer must be a uniquely declared nonconductive documentation layer",
        ));
    }
    Ok(())
}

pub(super) fn validate_native(board: &Value) -> Result<(), EngineError> {
    let Some(graphics) = board.get("graphics") else {
        return Ok(());
    };
    if board["schema_version"].as_u64() != Some(super::board_source_profile::BOARD_SOURCE_PROFILE) {
        return Err(refusal(
            "graphics require explicitly adopted BoardRoot profile 4",
        ));
    }
    let graphics = graphics
        .as_object()
        .ok_or_else(|| refusal("graphics must be a UUID-keyed map"))?;
    let mut imports = std::collections::BTreeSet::new();
    for (key, value) in graphics {
        let id = Uuid::parse_str(key).map_err(|_| refusal("invalid graphic map UUID"))?;
        if board["uuid"].as_str().and_then(|s| Uuid::parse_str(s).ok()) == Some(id) {
            return Err(refusal("graphic UUID collides with BoardRoot"));
        }
        validate_payload(value, id, board)?;
        for (field, source) in board
            .as_object()
            .ok_or_else(|| refusal("board must be an object"))?
        {
            if field != "graphics" && contains_uuid(source, id) {
                return Err(refusal("graphic UUID collides with another source object"));
            }
        }
        if let Some(key) = value.get("import_key").and_then(Value::as_str)
            && (key.is_empty() || !imports.insert(key))
        {
            return Err(refusal("empty or duplicate graphic import key"));
        }
    }
    Ok(())
}

fn contains_uuid(value: &Value, id: Uuid) -> bool {
    match value {
        Value::Object(map) => {
            map.get("uuid")
                .and_then(Value::as_str)
                .is_some_and(|v| Uuid::parse_str(v).ok() == Some(id))
                || map.values().any(|v| contains_uuid(v, id))
        }
        Value::Array(list) => list.iter().any(|v| contains_uuid(v, id)),
        _ => false,
    }
}

pub(super) fn apply(board: &mut Value, op: &Operation) -> Result<Option<bool>, EngineError> {
    let Some((board_id, id, graphic)) = identity(op) else {
        return Ok(None);
    };
    if board["uuid"].as_str().and_then(|s| Uuid::parse_str(s).ok()) != Some(board_id)
        || board["schema_version"].as_u64()
            != Some(super::board_source_profile::BOARD_SOURCE_PROFILE)
    {
        return Err(refusal(
            "wrong BoardRoot identity or profile; explicit adoption required",
        ));
    }
    validate_native(board)?;
    validate_payload(graphic, id, board)?;
    let root = board
        .as_object_mut()
        .ok_or_else(|| refusal("board must be an object"))?;
    let key = id.to_string();
    match op {
        Operation::CreateBoardGraphic {
            map_was_present, ..
        } => {
            if *map_was_present != root.contains_key("graphics") {
                return Err(refusal("graphics map presence guard mismatch"));
            }
            let map = root
                .entry("graphics")
                .or_insert_with(|| serde_json::json!({}))
                .as_object_mut()
                .ok_or_else(|| refusal("graphics must be a map"))?;
            if map.contains_key(&key) {
                return Err(refusal("graphic already exists"));
            }
            map.insert(key, graphic.clone());
        }
        Operation::SetBoardGraphic { .. } => {
            let slot = root
                .get_mut("graphics")
                .and_then(Value::as_object_mut)
                .and_then(|map| map.get_mut(&key))
                .ok_or_else(|| refusal("graphic does not exist"))?;
            if slot.get("import_key") != graphic.get("import_key") {
                return Err(refusal(
                    "graphic edits cannot replace recorded import provenance",
                ));
            }
            *slot = graphic.clone();
        }
        Operation::DeleteBoardGraphic { remove_map, .. } => {
            let map = root
                .get_mut("graphics")
                .and_then(Value::as_object_mut)
                .ok_or_else(|| refusal("graphics map absent"))?;
            if map.get(&key) != Some(graphic) {
                return Err(refusal("graphic deletion preimage mismatch"));
            }
            if *remove_map && map.len() != 1 {
                return Err(refusal("cannot remove a populated graphics map"));
            }
            map.remove(&key);
            if *remove_map {
                root.remove("graphics");
            }
        }
        _ => unreachable!(),
    }
    validate_native(board)?;
    Ok(Some(true))
}

pub(super) fn inverse(
    board: &mut Value,
    op: &Operation,
    inverse: &mut Vec<Operation>,
) -> Result<bool, EngineError> {
    let Some((board_id, graphic_id, graphic)) = identity(op) else {
        return Ok(false);
    };
    let previous = || {
        board
            .get("graphics")
            .and_then(|v| v.get(graphic_id.to_string()))
            .cloned()
            .ok_or_else(|| refusal("missing graphic inverse preimage"))
    };
    let undo = match op {
        Operation::CreateBoardGraphic {
            map_was_present, ..
        } => Operation::DeleteBoardGraphic {
            board_id,
            graphic_id,
            graphic: graphic.clone(),
            remove_map: !map_was_present,
        },
        Operation::SetBoardGraphic { .. } => Operation::SetBoardGraphic {
            board_id,
            graphic_id,
            graphic: previous()?,
        },
        Operation::DeleteBoardGraphic { remove_map, .. } => Operation::CreateBoardGraphic {
            board_id,
            graphic_id,
            graphic: previous()?,
            map_was_present: !remove_map,
        },
        _ => unreachable!(),
    };
    apply(board, op)?;
    inverse.push(undo);
    Ok(true)
}

pub(super) fn apply_model(
    model: &mut DesignModel,
    op: &Operation,
    diff: &mut CommitDiff,
) -> Result<bool, EngineError> {
    let Some((board_id, id, value)) = identity(op) else {
        return Ok(false);
    };
    let graphic: BoardGraphic = serde_json::from_value(value.clone())?;
    if graphic.uuid != id {
        return Err(refusal("operation UUID mismatch"));
    }
    graphic
        .validate()
        .map_err(|e| refusal(&format!("invalid geometry: {e:?}")))?;
    let board = model
        .objects
        .get(&board_id)
        .filter(|o| o.domain == "board" && o.kind == "$")
        .ok_or_else(|| refusal("missing authoritative BoardRoot object"))?;
    let shard_id = board.source_shard_id;
    if !model.source_shards.iter().any(|s| {
        s.shard_id == shard_id
            && s.schema_version == Some(super::board_source_profile::BOARD_SOURCE_PROFILE)
    }) {
        return Err(refusal("explicit profile-4 adoption required"));
    }
    if let Some(key) = &graphic.import_key
        && !model.import_map.get(key).is_some_and(|entry| {
            entry.object_id == id
                && entry.source_shard_id == shard_id
                && entry.status == super::ImportMapEntryStatus::Active
        })
    {
        return Err(refusal(
            "graphic import provenance requires its authoritative ImportMap binding",
        ));
    }
    match op {
        Operation::CreateBoardGraphic { .. } => {
            if model.objects.contains_key(&id) {
                return Err(refusal("UUID already registered"));
            }
            model.objects.insert(
                id,
                DomainObject {
                    object_id: id,
                    object_revision: ObjectRevision(0),
                    source_shard_id: shard_id,
                    domain: "board".into(),
                    kind: id.to_string(),
                },
            );
            diff.created.push(id);
        }
        Operation::SetBoardGraphic { .. } | Operation::DeleteBoardGraphic { .. } => {
            if !model
                .objects
                .get(&id)
                .is_some_and(|o| o.domain == "board" && o.source_shard_id == shard_id)
            {
                return Err(refusal("graphic source registration mismatch"));
            }
            if matches!(op, Operation::SetBoardGraphic { .. }) {
                super::operation_application_object_revision::bump_existing_object(
                    &mut model.objects,
                    id,
                    Some(diff),
                )?;
            } else {
                model.objects.remove(&id);
                diff.deleted.push(id);
            }
        }
        _ => unreachable!(),
    }
    Ok(true)
}
