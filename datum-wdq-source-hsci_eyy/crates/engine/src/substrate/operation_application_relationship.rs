use uuid::Uuid;

use super::{
    CommitDiff, DerivedRelationshipStatus, DerivedVariantPopulation, DesignModel, DomainObject,
    EngineError, ObjectId, Relationship, RelationshipKind, SourceShardDirtyState, SourceShardKind,
    SourceShardRef, VariantOverlay, source_shard::source_shard_taxon_for_path,
    source_shard_authority_for_kind,
};

pub(super) fn apply_relationship_create(
    model: &mut DesignModel,
    diff: &mut CommitDiff,
    relationship_id: ObjectId,
    value: &serde_json::Value,
) -> Result<(), EngineError> {
    let relationship: Relationship = serde_json::from_value(value.clone())?;
    validate_id(relationship.id, relationship_id, "relationship")?;
    let shard_id = authored_shard_id(SourceShardKind::Relationship, relationship_id)?;
    model.objects.insert(
        relationship_id,
        DomainObject {
            object_id: relationship_id,
            object_revision: relationship.object_revision,
            source_shard_id: shard_id,
            domain: "relationship".to_string(),
            kind: "relationship".to_string(),
        },
    );
    model.relationship_statuses.insert(
        relationship_id,
        derive_relationship_status(&relationship, model),
    );
    model.relationships.insert(relationship_id, relationship);
    ensure_authored_shard(model, SourceShardKind::Relationship, relationship_id)?;
    diff.created.push(relationship_id);
    Ok(())
}

pub(super) fn apply_relationship_delete(
    model: &mut DesignModel,
    diff: &mut CommitDiff,
    relationship_id: ObjectId,
) -> Result<(), EngineError> {
    model.relationships.remove(&relationship_id);
    model.relationship_statuses.remove(&relationship_id);
    model.objects.remove(&relationship_id);
    remove_authored_shard(model, SourceShardKind::Relationship, relationship_id)?;
    diff.deleted.push(relationship_id);
    Ok(())
}

pub(super) fn apply_relationship_set(
    model: &mut DesignModel,
    diff: &mut CommitDiff,
    relationship_id: ObjectId,
    value: &serde_json::Value,
) -> Result<(), EngineError> {
    let relationship: Relationship = serde_json::from_value(value.clone())?;
    validate_id(relationship.id, relationship_id, "relationship")?;
    let shard_id = authored_shard_id(SourceShardKind::Relationship, relationship_id)?;
    model.objects.insert(
        relationship_id,
        DomainObject {
            object_id: relationship_id,
            object_revision: relationship.object_revision,
            source_shard_id: shard_id,
            domain: "relationship".to_string(),
            kind: "relationship".to_string(),
        },
    );
    model.relationship_statuses.insert(
        relationship_id,
        derive_relationship_status(&relationship, model),
    );
    model.relationships.insert(relationship_id, relationship);
    ensure_authored_shard(model, SourceShardKind::Relationship, relationship_id)?;
    diff.modified.push(relationship_id);
    Ok(())
}

pub(super) fn apply_variant_create(
    model: &mut DesignModel,
    diff: &mut CommitDiff,
    variant_id: ObjectId,
    value: &serde_json::Value,
) -> Result<(), EngineError> {
    let variant: VariantOverlay = serde_json::from_value(value.clone())?;
    validate_id(variant.id, variant_id, "variant overlay")?;
    let shard_id = authored_shard_id(SourceShardKind::VariantOverlay, variant_id)?;
    model.objects.insert(
        variant_id,
        DomainObject {
            object_id: variant_id,
            object_revision: variant.variant_revision,
            source_shard_id: shard_id,
            domain: "variant".to_string(),
            kind: "variant_overlay".to_string(),
        },
    );
    model
        .variant_populations
        .insert(variant_id, derive_variant_population(&variant));
    model.variants.insert(variant_id, variant);
    ensure_authored_shard(model, SourceShardKind::VariantOverlay, variant_id)?;
    diff.created.push(variant_id);
    Ok(())
}

pub(super) fn apply_variant_delete(
    model: &mut DesignModel,
    diff: &mut CommitDiff,
    variant_id: ObjectId,
) -> Result<(), EngineError> {
    model.variants.remove(&variant_id);
    model.variant_populations.remove(&variant_id);
    model.objects.remove(&variant_id);
    remove_authored_shard(model, SourceShardKind::VariantOverlay, variant_id)?;
    diff.deleted.push(variant_id);
    Ok(())
}

pub(super) fn apply_variant_set(
    model: &mut DesignModel,
    diff: &mut CommitDiff,
    variant_id: ObjectId,
    value: &serde_json::Value,
) -> Result<(), EngineError> {
    let variant: VariantOverlay = serde_json::from_value(value.clone())?;
    validate_id(variant.id, variant_id, "variant overlay")?;
    let shard_id = authored_shard_id(SourceShardKind::VariantOverlay, variant_id)?;
    model.objects.insert(
        variant_id,
        DomainObject {
            object_id: variant_id,
            object_revision: variant.variant_revision,
            source_shard_id: shard_id,
            domain: "variant".to_string(),
            kind: "variant_overlay".to_string(),
        },
    );
    model
        .variant_populations
        .insert(variant_id, derive_variant_population(&variant));
    model.variants.insert(variant_id, variant);
    ensure_authored_shard(model, SourceShardKind::VariantOverlay, variant_id)?;
    diff.modified.push(variant_id);
    Ok(())
}

fn derive_relationship_status(
    relationship: &Relationship,
    model: &DesignModel,
) -> DerivedRelationshipStatus {
    match relationship.kind {
        RelationshipKind::Pending => DerivedRelationshipStatus::PendingImplementation,
        RelationshipKind::Mismatch => DerivedRelationshipStatus::UnresolvedMismatch,
        _ if relationship
            .from
            .iter()
            .chain(&relationship.to)
            .all(|reference| {
                model
                    .objects
                    .get(&reference.object_id)
                    .map(|object| object.object_revision == reference.object_revision)
                    .unwrap_or(false)
            }) =>
        {
            DerivedRelationshipStatus::Implemented
        }
        _ => DerivedRelationshipStatus::UnresolvedMismatch,
    }
}

fn derive_variant_population(
    variant: &VariantOverlay,
) -> std::collections::BTreeMap<ObjectId, DerivedVariantPopulation> {
    variant
        .fitted
        .iter()
        .map(|(object_id, fitted)| {
            let population = match fitted {
                super::FittedState::Fitted => DerivedVariantPopulation::Applicable,
                super::FittedState::Unfitted => DerivedVariantPopulation::NotApplicableForVariant,
            };
            (*object_id, population)
        })
        .collect()
}

fn ensure_authored_shard(
    model: &mut DesignModel,
    kind: SourceShardKind,
    object_id: ObjectId,
) -> Result<(), EngineError> {
    let relative_path = authored_relative_path(kind.clone(), object_id)?;
    if model
        .source_shards
        .iter()
        .any(|shard| shard.relative_path == relative_path)
    {
        return Ok(());
    }
    model.source_shards.push(SourceShardRef {
        shard_id: authored_shard_id(kind.clone(), object_id)?,
        kind: kind.clone(),
        taxon: source_shard_taxon_for_path(&kind, &relative_path),
        path: std::path::PathBuf::from(&relative_path),
        relative_path,
        authority: source_shard_authority_for_kind(&kind),
        dirty_state: SourceShardDirtyState::Clean,
        schema_version: Some(1),
        content_hash: String::new(),
    });
    Ok(())
}

fn remove_authored_shard(
    model: &mut DesignModel,
    kind: SourceShardKind,
    object_id: ObjectId,
) -> Result<(), EngineError> {
    let relative_path = authored_relative_path(kind, object_id)?;
    model
        .source_shards
        .retain(|shard| shard.relative_path != relative_path);
    Ok(())
}

fn authored_shard_id(kind: SourceShardKind, object_id: ObjectId) -> Result<Uuid, EngineError> {
    let relative_path = authored_relative_path(kind, object_id)?;
    Ok(Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!("datum-eda:source-shard:{relative_path}").as_bytes(),
    ))
}

pub(super) fn authored_relative_path(
    kind: SourceShardKind,
    object_id: ObjectId,
) -> Result<String, EngineError> {
    let directory = match kind {
        SourceShardKind::Relationship => ".datum/relationships",
        SourceShardKind::VariantOverlay => ".datum/variants",
        _ => {
            return Err(EngineError::Operation(format!(
                "unsupported authored shard kind for operation: {kind:?}"
            )));
        }
    };
    Ok(format!("{directory}/{object_id}.json"))
}

fn validate_id(actual: ObjectId, expected: ObjectId, label: &str) -> Result<(), EngineError> {
    if actual != expected {
        return Err(EngineError::Validation(format!(
            "{label} id {actual} does not match operation id {expected}"
        )));
    }
    Ok(())
}
