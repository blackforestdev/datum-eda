//! Compose identity exactly once from immutable source states, before staging.
use super::{
    DesignModel, ElectricalIdentity, ElectricalIdentityRecord, EngineError, NetAnchorReason,
    ObjectRevision, Operation, OperationBatch, PreviousNetGroup, TransactionKind,
};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

pub(super) fn compose(
    model: &DesignModel,
    mut batch: OperationBatch,
    kind: TransactionKind,
) -> Result<OperationBatch, EngineError> {
    if kind != TransactionKind::Normal || !batch.operations.iter().any(topology_edit) {
        return Ok(batch);
    }
    // Old projects require explicit adoption; reading or editing unadopted source
    // never silently imports identity provenance.
    if !model.electrical_identities.values().any(|r| matches!(&r.identity,ElectricalIdentity::Net {anchor,..} if schematic_class(&anchor.class))) { return Ok(batch); }
    let before = super::electrical_topology_source::partitions(model, &[])?;
    let after = super::electrical_topology_source::partitions(model, &batch.operations)?;
    let previous = previous(model, &before)?;
    if before == after {
        return Ok(batch);
    }
    let unclaimed: Vec<_> = after
        .iter()
        .filter(|group| {
            !previous.iter().any(|old| {
                let surviving = if after.iter().any(|g| g.contains(&old.anchor)) {
                    Some(&old.anchor)
                } else {
                    old.members
                        .iter()
                        .find(|member| after.iter().any(|g| g.contains(*member)))
                };
                surviving.is_some_and(|anchor| group.contains(anchor))
            })
        })
        .collect();
    let mut allocations = unclaimed
        .iter()
        .enumerate()
        .map(|(ordinal, group)| {
            let explicit: Vec<_> = batch
                .operations
                .iter()
                .filter_map(|op| match op {
                    Operation::CreateElectricalIdentity { record } => match &record.identity {
                        ElectricalIdentity::Net {
                            anchor,
                            retired: false,
                            ..
                        } if group.contains(anchor) => Some(record.id),
                        _ => None,
                    },
                    _ => None,
                })
                .collect();
            if explicit.len() > 1 {
                return Err(invalid("multiple explicit allocations for final group"));
            }
            Ok(explicit.first().copied().unwrap_or_else(|| {
                Uuid::new_v5(
                    &batch.batch_id,
                    format!("datum:net-allocation:{ordinal}").as_bytes(),
                )
            }))
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter();
    let transition = super::plan_net_identity_transition(
        model.model_revision.clone(),
        &previous,
        after.clone(),
        || {
            allocations
                .next()
                .expect("one allocation per unclaimed final group")
        },
    )?;
    let explicit: BTreeSet<_> = batch
        .operations
        .iter()
        .filter_map(super::electrical_identity_store::write)
        .map(|(id, _, _)| id)
        .collect();
    for group in &transition.final_groups {
        let previous = model.electrical_identities.get(&group.net_id);
        let record = ElectricalIdentityRecord {
            id: group.net_id,
            object_revision: match previous {
                Some(r) => next(r.object_revision)?,
                None => ObjectRevision(0),
            },
            identity: ElectricalIdentity::Net {
                anchor: group.anchor.clone(),
                anchor_reason: group.anchor_reason,
                retired: false,
                predecessors: group.predecessors.clone(),
            },
        };
        if explicit.contains(&record.id) {
            require_record(&batch.operations, &record)?;
        } else if let Some(previous) = previous {
            if previous.identity != record.identity {
                batch.operations.push(Operation::SetElectricalIdentity {
                    previous: Box::new(previous.clone()),
                    record,
                });
            }
        } else {
            batch
                .operations
                .push(Operation::CreateElectricalIdentity { record });
        }
    }
    for id in &transition.retired {
        let previous = &model.electrical_identities[id];
        let mut record = previous.clone();
        if let ElectricalIdentity::Net { retired, .. } = &mut record.identity {
            *retired = true;
        }
        record.object_revision = next(previous.object_revision)?;
        if explicit.contains(id) {
            require_record(&batch.operations, &record)?;
        } else {
            batch.operations.push(Operation::SetElectricalIdentity {
                previous: Box::new(previous.clone()),
                record,
            });
        }
    }
    super::electrical_transaction_bindings::compose(
        model,
        &previous,
        &transition,
        &mut batch.operations,
    )?;
    Ok(batch)
}

pub(super) fn previous(
    model: &DesignModel,
    groups: &[BTreeSet<super::ElectricalOccurrence>],
) -> Result<Vec<PreviousNetGroup>, EngineError> {
    let mut owners = BTreeMap::new();
    let mut result = vec![];
    for record in model.electrical_identities.values() {
        if let ElectricalIdentity::Net {
            anchor,
            retired: false,
            ..
        } = &record.identity
        {
            if !schematic_class(&anchor.class) {
                continue;
            }
            let group = groups
                .iter()
                .find(|g| g.contains(anchor))
                .ok_or_else(|| invalid("active Net anchor has no occurrence partition"))?;
            if owners.insert(group, record.id).is_some() {
                return Err(invalid("multiple Net identities for one electrical group"));
            }
            result.push(PreviousNetGroup {
                net_id: record.id,
                anchor: anchor.clone(),
                members: group.clone(),
            });
        }
    }
    Ok(result)
}
pub(crate) fn adoption(model: &DesignModel) -> Result<Vec<Operation>, EngineError> {
    let groups = super::electrical_topology_source::partitions(model, &[])?;
    let previous = previous(model, &groups)?;
    Ok(groups
        .into_iter()
        .filter(|g| !previous.iter().any(|old| old.members == *g))
        .map(|g| Operation::CreateElectricalIdentity {
            record: ElectricalIdentityRecord {
                id: Uuid::new_v4(),
                object_revision: ObjectRevision(0),
                identity: ElectricalIdentity::Net {
                    anchor: g.first().expect("nonempty partition").clone(),
                    anchor_reason: NetAnchorReason::Authored,
                    retired: false,
                    predecessors: BTreeSet::new(),
                },
            },
        })
        .collect())
}
fn require_record(
    operations: &[Operation],
    record: &ElectricalIdentityRecord,
) -> Result<(), EngineError> {
    let writes: Vec<_> = operations
        .iter()
        .filter_map(super::electrical_identity_store::write)
        .filter(|(id, _, _)| *id == record.id)
        .collect();
    if writes.len() != 1 || writes[0].2 || writes[0].1 != record {
        return Err(invalid(
            "explicit identity transition disagrees with complete topology",
        ));
    }
    Ok(())
}
pub(super) fn next(revision: ObjectRevision) -> Result<ObjectRevision, EngineError> {
    revision
        .0
        .checked_add(1)
        .map(ObjectRevision)
        .ok_or_else(|| invalid("revision overflow"))
}
pub(super) fn schematic_class(class: &str) -> bool {
    matches!(class, "wires" | "labels" | "junctions" | "ports" | "pins")
}
fn topology_edit(op: &Operation) -> bool {
    matches!(
        op,
        Operation::CreateSchematicWire { .. }
            | Operation::DeleteSchematicWire { .. }
            | Operation::CreateSchematicJunction { .. }
            | Operation::DeleteSchematicJunction { .. }
            | Operation::CreateSchematicLabel { .. }
            | Operation::SetSchematicLabel { .. }
            | Operation::DeleteSchematicLabel { .. }
            | Operation::CreateSchematicPort { .. }
            | Operation::SetSchematicPort { .. }
            | Operation::DeleteSchematicPort { .. }
            | Operation::CreateSchematicSymbol { .. }
            | Operation::SetSchematicSymbol { .. }
            | Operation::DeleteSchematicSymbol { .. }
            | Operation::CreateSchematicSheet { .. }
            | Operation::DeleteSchematicSheet { .. }
            | Operation::CreateSchematicDefinition { .. }
            | Operation::DeleteSchematicDefinition { .. }
            | Operation::CreateSchematicSheetInstance { .. }
            | Operation::SetSchematicSheetInstance { .. }
            | Operation::DeleteSchematicSheetInstance { .. }
    )
}
fn invalid(reason: &str) -> EngineError {
    EngineError::Validation(format!("invalid automatic Net transition: {reason}"))
}

/// Staging may add fresh shard references; existing bytes remain preimages until
/// application finishes. This is used identically by commit and proposal preview.
pub(super) fn retain_preimage_hashes(before: &DesignModel, staged: &mut DesignModel) {
    for shard in &mut staged.source_shards {
        if let Some(old) = before
            .source_shards
            .iter()
            .find(|old| old.shard_id == shard.shard_id)
        {
            shard.content_hash.clone_from(&old.content_hash);
        }
    }
}
