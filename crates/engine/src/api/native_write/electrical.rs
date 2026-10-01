//! Authored electrical identity/binding operations over the canonical write facade.
use super::{BatchComposer, PreparedWrite, WriteProvenance};
use crate::error::EngineError;
use crate::substrate::{
    DesignModel, ElectricalIdentity, ElectricalIdentityRecord, NetIdentityTransition,
    ObjectRevision, Operation,
};

pub fn build_create_electrical_identity(
    model: &DesignModel,
    provenance: WriteProvenance,
    mut record: ElectricalIdentityRecord,
) -> Result<PreparedWrite, EngineError> {
    record.object_revision = ObjectRevision(0);
    BatchComposer::compose(model, provenance)
        .primary_object(record.id)
        .push_op(Operation::CreateElectricalIdentity { record })
        .finish()
}

pub fn build_set_electrical_identity(
    model: &DesignModel,
    provenance: WriteProvenance,
    mut record: ElectricalIdentityRecord,
) -> Result<PreparedWrite, EngineError> {
    let previous = model
        .electrical_identities
        .get(&record.id)
        .ok_or(EngineError::NotFound {
            object_type: "electrical_identity",
            uuid: record.id,
        })?
        .clone();
    record.object_revision = ObjectRevision(
        previous
            .object_revision
            .0
            .checked_add(1)
            .ok_or_else(|| EngineError::Validation("identity revision overflow".into()))?,
    );
    BatchComposer::compose(model, provenance)
        .primary_object(record.id)
        .push_op(Operation::SetElectricalIdentity {
            previous: Box::new(previous),
            record,
        })
        .finish()
}

pub fn build_delete_electrical_identity(
    model: &DesignModel,
    provenance: WriteProvenance,
    id: uuid::Uuid,
) -> Result<PreparedWrite, EngineError> {
    let record = model
        .electrical_identities
        .get(&id)
        .ok_or(EngineError::NotFound {
            object_type: "electrical_identity",
            uuid: id,
        })?
        .clone();
    let mut retired_record = record.clone();
    match &mut retired_record.identity {
        ElectricalIdentity::Net { retired, .. } | ElectricalIdentity::Bus { retired, .. } => {
            *retired = true;
            return build_set_electrical_identity(model, provenance, retired_record);
        }
        _ => {}
    }
    BatchComposer::compose(model, provenance)
        .primary_object(id)
        .push_op(Operation::DeleteElectricalIdentity { record })
        .finish()
}

/// Attach an engine-planned identity transition to the *same* topology batch.
/// This validates record/preimage ownership, not the E3 graph partition itself.
/// It must not be used as a substitute for a resolved pre/final topology basis.
pub fn build_net_identity_transition(
    model: &DesignModel,
    provenance: WriteProvenance,
    transition: NetIdentityTransition,
    topology_operations: Vec<Operation>,
) -> Result<PreparedWrite, EngineError> {
    if transition.source_revision != model.model_revision {
        return Err(EngineError::Validation("stale Net transition basis".into()));
    }
    let mut operations = topology_operations;
    let mut seen = std::collections::BTreeSet::new();
    let mut occurrences = std::collections::BTreeSet::new();
    for group in transition.final_groups {
        if group.members.is_empty()
            || !group.members.contains(&group.anchor)
            || group
                .members
                .iter()
                .any(|member| !occurrences.insert(member.clone()))
        {
            return Err(EngineError::Validation(
                "invalid final Net membership witness".into(),
            ));
        }
        if !seen.insert(group.net_id) {
            return Err(EngineError::Validation("duplicate final NetId".into()));
        }
        let old = model.electrical_identities.get(&group.net_id);
        if group.allocated && old.is_some() {
            return Err(EngineError::Validation(
                "allocated Net already exists".into(),
            ));
        }
        let record = ElectricalIdentityRecord {
            id: group.net_id,
            object_revision: ObjectRevision(
                old.map_or(Ok(0), |r| next_revision(r.object_revision))?,
            ),
            identity: ElectricalIdentity::Net {
                anchor: group.anchor,
                anchor_reason: group.anchor_reason,
                retired: false,
                predecessors: group.predecessors,
            },
        };
        if group.allocated {
            operations.push(Operation::CreateElectricalIdentity { record });
        } else {
            let previous = old
                .ok_or(EngineError::NotFound {
                    object_type: "net_identity",
                    uuid: group.net_id,
                })?
                .clone();
            if !matches!(
                previous.identity,
                ElectricalIdentity::Net { retired: false, .. }
            ) {
                return Err(EngineError::Validation("survivor is not active Net".into()));
            }
            operations.push(Operation::SetElectricalIdentity {
                previous: Box::new(previous),
                record,
            });
        }
    }
    for id in transition.retired {
        if seen.contains(&id) {
            return Err(EngineError::Validation(
                "Net both retained and retired".into(),
            ));
        }
        let previous = model
            .electrical_identities
            .get(&id)
            .ok_or(EngineError::NotFound {
                object_type: "net_identity",
                uuid: id,
            })?
            .clone();
        let mut record = previous.clone();
        let ElectricalIdentity::Net { retired, .. } = &mut record.identity else {
            return Err(EngineError::Validation("retirement targets non-Net".into()));
        };
        *retired = true;
        record.object_revision = ObjectRevision(next_revision(record.object_revision)?);
        operations.push(Operation::SetElectricalIdentity {
            previous: Box::new(previous),
            record,
        });
    }
    BatchComposer::compose(model, provenance)
        .push_ops(operations)
        .finish()
}

#[cfg(test)]
mod binding_tests;
#[cfg(test)]
mod hierarchy_tests;
#[cfg(test)]
mod pool_reference_tests;
#[cfg(test)]
mod projection_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod topology_tests;
#[cfg(test)]
mod transition_tests;

mod bus_transition;
pub use bus_transition::{BusDistribution, build_bus_transition};

fn next_revision(revision: ObjectRevision) -> Result<u64, EngineError> {
    revision
        .0
        .checked_add(1)
        .ok_or_else(|| EngineError::Validation("identity revision overflow".into()))
}

mod adoption;
pub use adoption::{PlacedPinAdoption, build_adopt_pin_correspondence};

mod projection_lifecycle;
pub(super) use projection_lifecycle::projection_removals;

/// Explicitly adopt all currently resolved schematic Net groups. Reads allocate
/// nothing; this authored command preserves source IDs and existing Net owners.
pub fn build_adopt_schematic_net_identities(
    model: &DesignModel,
    provenance: WriteProvenance,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_ops(crate::substrate::electrical_adoption_operations(model)?)
        .finish()
}
