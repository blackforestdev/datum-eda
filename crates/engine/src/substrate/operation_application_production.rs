use uuid::Uuid;

use super::{
    CommitDiff, DesignModel, DomainObject, EngineError, ManufacturingPlan, ObjectId,
    ObjectRevision, OutputJob, PanelProjection, SourceShardDirtyState, SourceShardKind,
    SourceShardRef, artifact_validation::validate_production_record_payload_schema_version,
    source_shard::source_shard_taxon_for_path, source_shard_authority_for_kind,
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
    validate_production_payload(model, value, &shard_kind, object_id)?;
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
            taxon: source_shard_taxon_for_path(&shard_kind, &relative_path),
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
    upsert_production_map(model, &shard_kind, object_id, value)?;
    diff.created.push(object_id);
    Ok(())
}

pub(super) fn apply_production_delete(
    model: &mut DesignModel,
    diff: &mut CommitDiff,
    object_id: ObjectId,
) -> Result<(), EngineError> {
    remove_production_map_entry(model, object_id);
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
    let object = model.objects.get(&object_id).ok_or(EngineError::NotFound {
        object_type: "domain_object",
        uuid: object_id,
    })?;
    let shard_kind = model
        .source_shards
        .iter()
        .find(|shard| shard.shard_id == object.source_shard_id)
        .map(|shard| shard.kind.clone())
        .ok_or_else(|| {
            EngineError::Validation(format!(
                "production object {object_id} is missing source shard metadata"
            ))
        })?;
    validate_production_payload(model, value, &shard_kind, object_id)?;
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
    upsert_production_map(model, &shard_kind, object_id, value)?;
    diff.modified.push(object_id);
    Ok(())
}

fn upsert_production_map(
    model: &mut DesignModel,
    shard_kind: &SourceShardKind,
    object_id: ObjectId,
    value: &serde_json::Value,
) -> Result<(), EngineError> {
    match shard_kind {
        SourceShardKind::ManufacturingPlan => {
            model
                .manufacturing_plans
                .insert(object_id, serde_json::from_value(value.clone())?);
        }
        SourceShardKind::PanelProjection => {
            model
                .panel_projections
                .insert(object_id, serde_json::from_value(value.clone())?);
        }
        SourceShardKind::OutputJob => {
            model
                .output_jobs
                .insert(object_id, serde_json::from_value(value.clone())?);
        }
        _ => {}
    }
    Ok(())
}

fn remove_production_map_entry(model: &mut DesignModel, object_id: ObjectId) {
    model.manufacturing_plans.remove(&object_id);
    model.panel_projections.remove(&object_id);
    model.output_jobs.remove(&object_id);
}

fn validate_production_payload(
    model: &DesignModel,
    value: &serde_json::Value,
    shard_kind: &SourceShardKind,
    object_id: ObjectId,
) -> Result<(), EngineError> {
    match shard_kind {
        SourceShardKind::ManufacturingPlan => {
            let plan: ManufacturingPlan = serde_json::from_value(value.clone())?;
            validate_production_record_payload_schema_version(
                plan.schema_version,
                "manufacturing plan",
            )
            .map_err(EngineError::Validation)?;
            validate_payload_id(value, object_id)?;
            validate_board_or_panel_target(model, plan.board_or_panel, "manufacturing plan")?;
            validate_optional_variant(model, plan.variant, "manufacturing plan")?;
        }
        SourceShardKind::PanelProjection => {
            let panel: PanelProjection = serde_json::from_value(value.clone())?;
            validate_production_record_payload_schema_version(
                panel.schema_version,
                "panel projection",
            )
            .map_err(EngineError::Validation)?;
            validate_payload_id(value, object_id)?;
            for instance in &panel.board_instances {
                validate_project_board_target(
                    model,
                    instance.board,
                    "panel projection board instance",
                )?;
            }
        }
        SourceShardKind::OutputJob => {
            let job: OutputJob = serde_json::from_value(value.clone())?;
            validate_production_record_payload_schema_version(job.schema_version, "output job")
                .map_err(EngineError::Validation)?;
            validate_payload_id(value, object_id)?;
            validate_board_or_panel_target(model, job.board_or_panel, "output job")?;
            validate_optional_variant(model, job.variant, "output job")?;
            if let Some(plan_id) = job.manufacturing_plan
                && !model.manufacturing_plans.contains_key(&plan_id)
                    && !object_has_domain_kind(
                        model,
                        plan_id,
                        "manufacturing",
                        "manufacturing_plan",
                    )
                {
                    return Err(EngineError::Validation(format!(
                        "output job {object_id} references missing manufacturing plan {plan_id}"
                    )));
                }
        }
        _ => {
            return Err(EngineError::Operation(format!(
                "unsupported production shard kind for operation: {shard_kind:?}"
            )));
        }
    }
    Ok(())
}

fn validate_board_or_panel_target(
    model: &DesignModel,
    target: ObjectId,
    subject: &str,
) -> Result<(), EngineError> {
    if validate_project_board_target(model, target, subject).is_ok()
        || model.panel_projections.contains_key(&target)
        || object_has_domain_kind(model, target, "manufacturing", "panel_projection")
    {
        return Ok(());
    }
    Err(EngineError::Validation(format!(
        "{subject} references missing board or panel {target}"
    )))
}

fn object_has_domain_kind(
    model: &DesignModel,
    object_id: ObjectId,
    domain: &str,
    kind: &str,
) -> bool {
    model
        .objects
        .get(&object_id)
        .map(|object| object.domain == domain && object.kind == kind)
        .unwrap_or(false)
}

fn validate_project_board_target(
    model: &DesignModel,
    target: ObjectId,
    subject: &str,
) -> Result<(), EngineError> {
    let Some(object) = model.objects.get(&target) else {
        return Err(EngineError::Validation(format!(
            "{subject} references missing board {target}"
        )));
    };
    let is_board_root = model.source_shards.iter().any(|shard| {
        shard.shard_id == object.source_shard_id && shard.kind == SourceShardKind::BoardRoot
    });
    if object.domain == "board" && is_board_root {
        return Ok(());
    }
    Err(EngineError::Validation(format!(
        "{subject} references non-board object {target}"
    )))
}

fn validate_optional_variant(
    model: &DesignModel,
    variant: Option<ObjectId>,
    subject: &str,
) -> Result<(), EngineError> {
    if let Some(variant_id) = variant
        && !model.variants.contains_key(&variant_id) {
            return Err(EngineError::Validation(format!(
                "{subject} references missing variant {variant_id}"
            )));
        }
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
