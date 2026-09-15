use std::path::Path;

use uuid::Uuid;

use super::{
    DerivedRelationshipStatus, DerivedVariantPopulation, DomainObject, EngineError, FittedState,
    ObjectId, Operation, OperationBatch, RELATIONSHIP_SHARD_SCHEMA_VERSION, Relationship,
    RelationshipKind, SourceShardKind, VARIANT_OVERLAY_SHARD_SCHEMA_VERSION, VariantOverlay,
    journal::{StagedShardWrite, stage_new_shard_write},
    operation_application_relationship::authored_relative_path,
};

pub(super) fn stage_relationship_operation(
    project_root: &Path,
    batch: &OperationBatch,
    operation: &Operation,
) -> Result<Option<StagedShardWrite>, EngineError> {
    let Some((kind, object_id, value, delete)) = relationship_operation_write(operation) else {
        return Ok(None);
    };
    let relative_path = authored_relative_path(kind.clone(), object_id)?;
    let destination = project_root.join(&relative_path);
    if delete {
        return Ok(Some(StagedShardWrite {
            destination,
            staged: None,
            kind,
            relative_path,
            content_hash: String::new(),
            schema_version: None,
            delete: true,
        }));
    }
    let wrapper = wrap_payload(&kind, value.clone());
    stage_new_shard_write(project_root, batch, kind, &relative_path, &wrapper).map(Some)
}

pub(super) fn inverse_relationship_operation(
    operation: &Operation,
    inverse_operations: &mut Vec<Operation>,
) {
    match operation {
        Operation::CreateRelationship {
            relationship_id,
            relationship,
        } => inverse_operations.push(Operation::DeleteRelationship {
            relationship_id: *relationship_id,
            relationship: relationship.clone(),
        }),
        Operation::DeleteRelationship {
            relationship_id,
            relationship,
        } => inverse_operations.push(Operation::CreateRelationship {
            relationship_id: *relationship_id,
            relationship: relationship.clone(),
        }),
        Operation::SetRelationship {
            relationship_id,
            previous_relationship,
            relationship,
        } => inverse_operations.push(Operation::SetRelationship {
            relationship_id: *relationship_id,
            previous_relationship: relationship.clone(),
            relationship: previous_relationship.clone(),
        }),
        Operation::CreateVariantOverlay {
            variant_id,
            variant,
        } => inverse_operations.push(Operation::DeleteVariantOverlay {
            variant_id: *variant_id,
            variant: variant.clone(),
        }),
        Operation::DeleteVariantOverlay {
            variant_id,
            variant,
        } => inverse_operations.push(Operation::CreateVariantOverlay {
            variant_id: *variant_id,
            variant: variant.clone(),
        }),
        Operation::SetVariantOverlay {
            variant_id,
            previous_variant,
            variant,
        } => inverse_operations.push(Operation::SetVariantOverlay {
            variant_id: *variant_id,
            previous_variant: variant.clone(),
            variant: previous_variant.clone(),
        }),
        _ => {}
    }
}

pub(super) fn apply_relationship_shard_operation(
    shard_kind: &SourceShardKind,
    value: &mut serde_json::Value,
    operation: &Operation,
) -> Result<bool, EngineError> {
    let Some((operation_kind, object_id, payload, delete)) =
        relationship_operation_write(operation)
    else {
        return Ok(false);
    };
    if &operation_kind != shard_kind {
        return Ok(false);
    }
    let current_id = match shard_kind {
        SourceShardKind::Relationship => value
            .get("relationships")
            .and_then(serde_json::Value::as_array)
            .and_then(|values| values.first())
            .and_then(|entry| entry.get("id"))
            .and_then(serde_json::Value::as_str),
        SourceShardKind::VariantOverlay => value
            .get("variants")
            .and_then(serde_json::Value::as_array)
            .and_then(|values| values.first())
            .and_then(|entry| entry.get("id"))
            .and_then(serde_json::Value::as_str),
        _ => None,
    };
    if current_id != Some(object_id.to_string().as_str()) {
        return Ok(false);
    }
    *value = if delete {
        serde_json::Value::Null
    } else {
        wrap_payload(shard_kind, payload.clone())
    };
    Ok(true)
}

pub(super) fn relationship_operation_write(
    operation: &Operation,
) -> Option<(SourceShardKind, Uuid, &serde_json::Value, bool)> {
    match operation {
        Operation::CreateRelationship {
            relationship_id,
            relationship,
        } => Some((
            SourceShardKind::Relationship,
            *relationship_id,
            relationship,
            false,
        )),
        Operation::DeleteRelationship {
            relationship_id,
            relationship,
        } => Some((
            SourceShardKind::Relationship,
            *relationship_id,
            relationship,
            true,
        )),
        Operation::SetRelationship {
            relationship_id,
            relationship,
            ..
        } => Some((
            SourceShardKind::Relationship,
            *relationship_id,
            relationship,
            false,
        )),
        Operation::CreateVariantOverlay {
            variant_id,
            variant,
        } => Some((SourceShardKind::VariantOverlay, *variant_id, variant, false)),
        Operation::DeleteVariantOverlay {
            variant_id,
            variant,
        } => Some((SourceShardKind::VariantOverlay, *variant_id, variant, true)),
        Operation::SetVariantOverlay {
            variant_id,
            variant,
            ..
        } => Some((SourceShardKind::VariantOverlay, *variant_id, variant, false)),
        _ => None,
    }
}

pub(super) fn apply_relationship_journal_to_maps(
    journal: &[super::TransactionRecord],
    objects: &std::collections::BTreeMap<ObjectId, DomainObject>,
    relationships: &mut std::collections::BTreeMap<ObjectId, Relationship>,
    relationship_statuses: &mut std::collections::BTreeMap<ObjectId, DerivedRelationshipStatus>,
    variants: &mut std::collections::BTreeMap<ObjectId, VariantOverlay>,
    variant_populations: &mut std::collections::BTreeMap<
        ObjectId,
        std::collections::BTreeMap<ObjectId, DerivedVariantPopulation>,
    >,
) -> Result<(), EngineError> {
    for transaction in journal {
        for operation in &transaction.operations {
            match operation {
                Operation::CreateRelationship {
                    relationship_id,
                    relationship,
                } => {
                    let relationship: Relationship = serde_json::from_value(relationship.clone())?;
                    relationships.insert(*relationship_id, relationship.clone());
                    relationship_statuses.insert(
                        *relationship_id,
                        derive_relationship_status(&relationship, objects),
                    );
                }
                Operation::DeleteRelationship {
                    relationship_id, ..
                } => {
                    relationships.remove(relationship_id);
                    relationship_statuses.remove(relationship_id);
                }
                Operation::SetRelationship {
                    relationship_id,
                    relationship,
                    ..
                } => {
                    let relationship: Relationship = serde_json::from_value(relationship.clone())?;
                    relationships.insert(*relationship_id, relationship.clone());
                    relationship_statuses.insert(
                        *relationship_id,
                        derive_relationship_status(&relationship, objects),
                    );
                }
                Operation::CreateVariantOverlay {
                    variant_id,
                    variant,
                } => {
                    let variant: VariantOverlay = serde_json::from_value(variant.clone())?;
                    variant_populations.insert(*variant_id, derive_variant_population(&variant));
                    variants.insert(*variant_id, variant);
                }
                Operation::DeleteVariantOverlay { variant_id, .. } => {
                    variants.remove(variant_id);
                    variant_populations.remove(variant_id);
                }
                Operation::SetVariantOverlay {
                    variant_id,
                    variant,
                    ..
                } => {
                    let variant: VariantOverlay = serde_json::from_value(variant.clone())?;
                    variant_populations.insert(*variant_id, derive_variant_population(&variant));
                    variants.insert(*variant_id, variant);
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn derive_relationship_status(
    relationship: &Relationship,
    objects: &std::collections::BTreeMap<ObjectId, DomainObject>,
) -> DerivedRelationshipStatus {
    match relationship.kind {
        RelationshipKind::Pending => DerivedRelationshipStatus::PendingImplementation,
        RelationshipKind::Mismatch => DerivedRelationshipStatus::UnresolvedMismatch,
        _ if relationship
            .from
            .iter()
            .chain(&relationship.to)
            .all(|reference| {
                objects
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
                FittedState::Fitted => DerivedVariantPopulation::Applicable,
                FittedState::Unfitted => DerivedVariantPopulation::NotApplicableForVariant,
            };
            (*object_id, population)
        })
        .collect()
}

pub(super) fn wrap_payload(
    kind: &SourceShardKind,
    payload: serde_json::Value,
) -> serde_json::Value {
    match kind {
        SourceShardKind::Relationship => serde_json::json!({
            "schema_version": RELATIONSHIP_SHARD_SCHEMA_VERSION,
            "relationships": [payload]
        }),
        SourceShardKind::VariantOverlay => serde_json::json!({
            "schema_version": VARIANT_OVERLAY_SHARD_SCHEMA_VERSION,
            "variants": [payload]
        }),
        _ => payload,
    }
}
