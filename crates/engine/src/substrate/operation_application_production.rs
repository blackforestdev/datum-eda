use uuid::Uuid;

use super::{
    CommitDiff, DesignModel, DomainObject, EngineError, ObjectId, ObjectRevision,
    SourceShardDirtyState, SourceShardKind, SourceShardRef, source_shard_authority_for_kind,
};

pub(super) fn apply_production_create(
    model: &mut DesignModel,
    diff: &mut CommitDiff,
    object_id: ObjectId,
    value: &serde_json::Value,
    shard_kind: SourceShardKind,
    domain: &str,
    kind: &str,
) -> Result<(), EngineError> {
    validate_payload_id(value, object_id)?;
    let relative_path = production_relative_path(shard_kind.clone(), object_id)?;
    let shard_id = Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!("datum-eda:source-shard:{relative_path}").as_bytes(),
    );
    model.objects.insert(
        object_id,
        DomainObject {
            object_id,
            object_revision: ObjectRevision(0),
            source_shard_id: shard_id,
            domain: domain.to_string(),
            kind: kind.to_string(),
        },
    );
    if !model
        .source_shards
        .iter()
        .any(|shard| shard.relative_path == relative_path)
    {
        model.source_shards.push(SourceShardRef {
            shard_id,
            kind: shard_kind.clone(),
            path: std::path::PathBuf::from(&relative_path),
            relative_path,
            authority: source_shard_authority_for_kind(&shard_kind),
            dirty_state: SourceShardDirtyState::Clean,
            schema_version: value
                .get("schema_version")
                .and_then(serde_json::Value::as_u64),
            content_hash: String::new(),
        });
    }
    diff.created.push(object_id);
    Ok(())
}

pub(super) fn apply_production_delete(
    model: &mut DesignModel,
    diff: &mut CommitDiff,
    object_id: ObjectId,
) -> Result<(), EngineError> {
    if model.objects.remove(&object_id).is_some() {
        diff.deleted.push(object_id);
    }
    model.source_shards.retain(|shard| {
        shard.path.file_stem().and_then(|value| value.to_str()) != Some(&object_id.to_string())
    });
    Ok(())
}

pub(super) fn apply_production_set(
    model: &mut DesignModel,
    diff: &mut CommitDiff,
    object_id: ObjectId,
    value: &serde_json::Value,
) -> Result<(), EngineError> {
    validate_payload_id(value, object_id)?;
    let object = model
        .objects
        .get_mut(&object_id)
        .ok_or(EngineError::NotFound {
            object_type: "domain_object",
            uuid: object_id,
        })?;
    let payload_revision = value
        .get("object_revision")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| {
            EngineError::Validation("production payload missing object_revision".to_string())
        })?;
    object.object_revision = ObjectRevision(payload_revision);
    diff.modified.push(object_id);
    Ok(())
}

fn production_relative_path(
    shard_kind: SourceShardKind,
    object_id: ObjectId,
) -> Result<String, EngineError> {
    let directory = match shard_kind {
        SourceShardKind::ManufacturingPlan => ".datum/manufacturing_plans",
        SourceShardKind::PanelProjection => ".datum/panel_projections",
        SourceShardKind::OutputJob => ".datum/output_jobs",
        _ => {
            return Err(EngineError::Operation(format!(
                "unsupported production shard kind for operation: {shard_kind:?}"
            )));
        }
    };
    Ok(format!("{directory}/{object_id}.json"))
}

fn validate_payload_id(value: &serde_json::Value, expected: ObjectId) -> Result<(), EngineError> {
    let actual = value
        .get("id")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| EngineError::Validation("production payload missing id".to_string()))?;
    let actual = Uuid::parse_str(actual).map_err(|error| {
        EngineError::Validation(format!("invalid production payload id: {error}"))
    })?;
    if actual != expected {
        return Err(EngineError::Validation(format!(
            "production payload id {actual} does not match operation id {expected}"
        )));
    }
    Ok(())
}
