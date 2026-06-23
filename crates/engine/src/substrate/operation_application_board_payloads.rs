use std::collections::BTreeMap;

use super::{
    DesignModel, DomainObject, EngineError, ObjectId, SourceShardKind, collect_uuid_objects,
};

pub(super) fn board_pad_payload_objects(
    model: &DesignModel,
    pad_id: ObjectId,
    pad: &serde_json::Value,
) -> Result<BTreeMap<ObjectId, DomainObject>, EngineError> {
    payload_objects_with_map(model, "pads", pad_id, pad)
}

pub(super) fn board_track_payload_objects(
    model: &DesignModel,
    track_id: ObjectId,
    track: &serde_json::Value,
) -> Result<BTreeMap<ObjectId, DomainObject>, EngineError> {
    payload_objects_with_map(model, "tracks", track_id, track)
}

pub(super) fn board_via_payload_objects(
    model: &DesignModel,
    via_id: ObjectId,
    via: &serde_json::Value,
) -> Result<BTreeMap<ObjectId, DomainObject>, EngineError> {
    payload_objects_with_map(model, "vias", via_id, via)
}

pub(super) fn board_zone_payload_objects(
    model: &DesignModel,
    zone_id: ObjectId,
    zone: &serde_json::Value,
) -> Result<BTreeMap<ObjectId, DomainObject>, EngineError> {
    payload_objects_with_map(model, "zones", zone_id, zone)
}

pub(super) fn board_net_payload_objects(
    model: &DesignModel,
    net_id: ObjectId,
    net: &serde_json::Value,
) -> Result<BTreeMap<ObjectId, DomainObject>, EngineError> {
    payload_objects_with_map(model, "nets", net_id, net)
}

pub(super) fn board_net_class_payload_objects(
    model: &DesignModel,
    net_class_id: ObjectId,
    net_class: &serde_json::Value,
) -> Result<BTreeMap<ObjectId, DomainObject>, EngineError> {
    payload_objects_with_map(model, "net_classes", net_class_id, net_class)
}

pub(super) fn board_dimension_payload_objects(
    model: &DesignModel,
    _dimension_id: ObjectId,
    dimension: &serde_json::Value,
) -> Result<BTreeMap<ObjectId, DomainObject>, EngineError> {
    payload_objects_with_array(model, "dimensions", dimension)
}

pub(super) fn board_text_payload_objects(
    model: &DesignModel,
    _text_id: ObjectId,
    text: &serde_json::Value,
) -> Result<BTreeMap<ObjectId, DomainObject>, EngineError> {
    payload_objects_with_array(model, "texts", text)
}

pub(super) fn board_keepout_payload_objects(
    model: &DesignModel,
    _keepout_id: ObjectId,
    keepout: &serde_json::Value,
) -> Result<BTreeMap<ObjectId, DomainObject>, EngineError> {
    payload_objects_with_array(model, "keepouts", keepout)
}

pub(super) fn materialized_payload_objects(
    model: &DesignModel,
    package_id: ObjectId,
    materialized: &serde_json::Value,
) -> Result<BTreeMap<ObjectId, DomainObject>, EngineError> {
    payload_objects(model, package_id, None, materialized)
}

pub(super) fn board_payload_objects(
    model: &DesignModel,
    package_id: ObjectId,
    package: &serde_json::Value,
    materialized: &serde_json::Value,
) -> Result<BTreeMap<ObjectId, DomainObject>, EngineError> {
    payload_objects(model, package_id, Some(package), materialized)
}

fn payload_objects(
    model: &DesignModel,
    package_id: ObjectId,
    package: Option<&serde_json::Value>,
    materialized: &serde_json::Value,
) -> Result<BTreeMap<ObjectId, DomainObject>, EngineError> {
    let board_shard = model
        .source_shards
        .iter()
        .find(|shard| shard.kind == SourceShardKind::BoardRoot)
        .ok_or_else(|| EngineError::Validation("model missing board root shard".to_string()))?;
    let mut fragment = serde_json::Map::new();
    if let Some(package) = package {
        fragment.insert(
            "packages".to_string(),
            serde_json::json!({ package_id.to_string(): package }),
        );
    }
    if let Some(payload) = materialized.as_object() {
        for (key, value) in payload {
            fragment.insert(
                key.clone(),
                serde_json::json!({ package_id.to_string(): value }),
            );
        }
    }
    let mut objects = BTreeMap::new();
    let mut import_map = BTreeMap::new();
    collect_uuid_objects(
        &serde_json::Value::Object(fragment),
        board_shard,
        "board",
        &mut objects,
        &mut import_map,
    );
    Ok(objects)
}

fn payload_objects_with_map(
    model: &DesignModel,
    map_name: &str,
    object_id: ObjectId,
    value: &serde_json::Value,
) -> Result<BTreeMap<ObjectId, DomainObject>, EngineError> {
    let board_shard = model
        .source_shards
        .iter()
        .find(|shard| shard.kind == SourceShardKind::BoardRoot)
        .ok_or_else(|| EngineError::Validation("model missing board root shard".to_string()))?;
    let fragment = serde_json::json!({ map_name: { object_id.to_string(): value } });
    let mut objects = BTreeMap::new();
    let mut import_map = BTreeMap::new();
    collect_uuid_objects(
        &fragment,
        board_shard,
        "board",
        &mut objects,
        &mut import_map,
    );
    Ok(objects)
}

fn payload_objects_with_array(
    model: &DesignModel,
    array_name: &str,
    value: &serde_json::Value,
) -> Result<BTreeMap<ObjectId, DomainObject>, EngineError> {
    let board_shard = model
        .source_shards
        .iter()
        .find(|shard| shard.kind == SourceShardKind::BoardRoot)
        .ok_or_else(|| EngineError::Validation("model missing board root shard".to_string()))?;
    let fragment = serde_json::json!({ array_name: [value] });
    let mut objects = BTreeMap::new();
    let mut import_map = BTreeMap::new();
    collect_uuid_objects(
        &fragment,
        board_shard,
        "board",
        &mut objects,
        &mut import_map,
    );
    Ok(objects)
}
