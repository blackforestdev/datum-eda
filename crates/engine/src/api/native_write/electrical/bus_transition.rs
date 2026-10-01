//! Explicit semantic split/merge; drawing cuts never invoke this builder.
use super::*;
use crate::substrate::ElectricalOccurrence;
use std::collections::BTreeSet;
use uuid::Uuid;

/// Explicit accounting for removed/new membership. Scalar membership may be
/// shared between declarations; each representation occurrence has one owner.
#[derive(Debug, Clone, Default)]
pub struct BusDistribution {
    pub removed_scalars: BTreeSet<Uuid>,
    pub added_scalars: BTreeSet<Uuid>,
    pub removed_representations: BTreeSet<ElectricalOccurrence>,
    pub added_representations: BTreeSet<ElectricalOccurrence>,
}

/// Exactly one source survivor is retained. Split allocates explicitly supplied
/// fresh IDs; merge retires all other sources. Interface rebindings supplied in
/// the same batch are subject to the final-state canonical validator.
pub fn build_bus_transition(
    model: &DesignModel,
    provenance: WriteProvenance,
    sources: BTreeSet<Uuid>,
    survivor: Uuid,
    destinations: Vec<ElectricalIdentityRecord>,
    distribution: BusDistribution,
    interface_operations: Vec<Operation>,
) -> Result<PreparedWrite, EngineError> {
    let invalid =
        |reason: &str| EngineError::Validation(format!("invalid Bus transition: {reason}"));
    if sources.is_empty() || !sources.contains(&survivor) || destinations.is_empty() {
        return Err(invalid("missing source/survivor/destination"));
    }
    let mut old_scalars = BTreeSet::new();
    let mut old_refs = BTreeSet::new();
    for id in &sources {
        let record = model
            .electrical_identities
            .get(id)
            .ok_or_else(|| invalid("missing source"))?;
        let ElectricalIdentity::Bus {
            scalar_nets,
            representations,
            retired: false,
            ..
        } = &record.identity
        else {
            return Err(invalid("source is not active Bus"));
        };
        old_scalars.extend(scalar_nets);
        old_refs.extend(representations.iter().cloned());
    }
    let mut ids = BTreeSet::new();
    let mut new_scalars = BTreeSet::new();
    let mut new_refs = BTreeSet::new();
    for record in &destinations {
        if record.id.is_nil() || !ids.insert(record.id) {
            return Err(invalid("nil/duplicate destination"));
        }
        if record.id != survivor && model.objects.contains_key(&record.id) {
            return Err(invalid("new destination identity already exists"));
        }
        let ElectricalIdentity::Bus {
            scalar_nets,
            representations,
            retired: false,
            ..
        } = &record.identity
        else {
            return Err(invalid("destination is not active Bus"));
        };
        new_scalars.extend(scalar_nets);
        for reference in representations {
            if !new_refs.insert(reference.clone()) {
                return Err(invalid("duplicate representation owner"));
            }
        }
    }
    if !ids.contains(&survivor) {
        return Err(invalid("survivor not retained"));
    }
    check_distribution(
        &old_scalars,
        &new_scalars,
        &distribution.removed_scalars,
        &distribution.added_scalars,
    )?;
    check_distribution(
        &old_refs,
        &new_refs,
        &distribution.removed_representations,
        &distribution.added_representations,
    )?;
    if interface_operations.iter().any(|op| {
        !matches!(
            op,
            Operation::CreateElectricalIdentity {
                record: ElectricalIdentityRecord {
                    identity: ElectricalIdentity::BusInterface { .. },
                    ..
                }
            } | Operation::SetElectricalIdentity {
                record: ElectricalIdentityRecord {
                    identity: ElectricalIdentity::BusInterface { .. },
                    ..
                },
                ..
            } | Operation::DeleteElectricalIdentity {
                record: ElectricalIdentityRecord {
                    identity: ElectricalIdentity::BusInterface { .. },
                    ..
                }
            }
        )
    }) {
        return Err(invalid("extra operations must be interface rebindings"));
    }
    let mut operations = interface_operations;
    for mut record in destinations {
        if record.id == survivor {
            let previous = model.electrical_identities[&survivor].clone();
            record.object_revision = ObjectRevision(next_revision(previous.object_revision)?);
            operations.push(Operation::SetElectricalIdentity {
                previous: Box::new(previous),
                record,
            });
        } else {
            record.object_revision = ObjectRevision(0);
            operations.push(Operation::CreateElectricalIdentity { record });
        }
    }
    for id in sources.into_iter().filter(|id| *id != survivor) {
        let previous = model.electrical_identities[&id].clone();
        let mut record = previous.clone();
        if let ElectricalIdentity::Bus { retired, .. } = &mut record.identity {
            *retired = true;
        }
        record.object_revision = ObjectRevision(next_revision(previous.object_revision)?);
        operations.push(Operation::SetElectricalIdentity {
            previous: Box::new(previous),
            record,
        });
    }
    BatchComposer::compose(model, provenance)
        .primary_object(survivor)
        .push_ops(operations)
        .finish()
}

fn check_distribution<T: Ord + Clone>(
    old: &BTreeSet<T>,
    new: &BTreeSet<T>,
    removed: &BTreeSet<T>,
    added: &BTreeSet<T>,
) -> Result<(), EngineError> {
    if !removed.is_subset(old)
        || !added.is_disjoint(old)
        || removed != &old.difference(new).cloned().collect()
        || added != &new.difference(old).cloned().collect()
    {
        return Err(EngineError::Validation(
            "Bus distribution omits or invents removed/new membership".into(),
        ));
    }
    Ok(())
}
