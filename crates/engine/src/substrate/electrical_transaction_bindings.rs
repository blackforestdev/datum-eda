//! Reconcile affected source bindings; never rewrite board copper or duplicate it.
use super::{
    BusInterfaceEquivalence, DesignModel, ElectricalIdentity, EngineError, NetIdentityTransition,
    NetRelationshipIntent, Operation, PreviousNetGroup,
};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

pub(super) fn compose(
    model: &DesignModel,
    previous: &[PreviousNetGroup],
    transition: &NetIdentityTransition,
    operations: &mut Vec<Operation>,
) -> Result<(), EngineError> {
    let successor: BTreeMap<_, _> = transition
        .final_groups
        .iter()
        .flat_map(|g| g.predecessors.iter().map(move |id| (*id, g.net_id)))
        .collect();
    let alive: BTreeSet<_> = transition
        .final_groups
        .iter()
        .flat_map(|g| g.members.iter().cloned())
        .collect();
    let remap = |id: Uuid| -> Option<Uuid> {
        if transition.retired.contains(&id) {
            successor.get(&id).copied()
        } else {
            Some(id)
        }
    };
    let mut finals = model.electrical_identities.clone();
    let mut explicitly_written = BTreeSet::new();
    for op in operations.iter() {
        if let Some((id, record, delete)) = super::electrical_identity_store::write(op) {
            explicitly_written.insert(id);
            if delete {
                finals.remove(&id);
            } else {
                finals.insert(id, record.clone());
            }
        }
    }
    for record in finals.values_mut() {
        if explicitly_written.contains(&record.id) {
            continue;
        }
        match &mut record.identity {
            ElectricalIdentity::NetRelationship {
                logical_net,
                intent,
                evidence,
                ..
            } => {
                if let Some(old) = previous.iter().find(|g| g.net_id == *logical_net) {
                    let next = transition
                        .final_groups
                        .iter()
                        .find(|g| g.net_id == *logical_net);
                    if next.is_none_or(|g| g.members != old.members) {
                        // Keep an absent successor as a pending historical endpoint.
                        // It cannot certify any active cross-domain subject.
                        if let Some(id) = remap(*logical_net) {
                            *logical_net = id;
                        }
                        if *intent != NetRelationshipIntent::SchematicOnly || next.is_none() {
                            *intent = NetRelationshipIntent::Pending;
                        }
                        evidence.retain(|e| alive.contains(&e.schematic_terminal));
                    }
                }
            }
            ElectricalIdentity::Bus {
                scalar_nets,
                retired: false,
                ..
            } => {
                *scalar_nets = scalar_nets.iter().filter_map(|id| remap(*id)).collect();
            }
            _ => {}
        }
    }
    let bus_members: BTreeMap<_, _> = finals
        .iter()
        .filter_map(|(id, r)| match &r.identity {
            ElectricalIdentity::Bus {
                scalar_nets,
                retired: false,
                ..
            } => Some((*id, scalar_nets.clone())),
            _ => None,
        })
        .collect();
    for record in finals.values_mut() {
        if explicitly_written.contains(&record.id) {
            continue;
        }
        if let ElectricalIdentity::BusInterface {
            left_bus,
            equivalence,
            scalar_mapping,
            ..
        } = &mut record.identity
        {
            if *equivalence == BusInterfaceEquivalence::Identity {
                if let Some(members) = bus_members.get(left_bus) {
                    *scalar_mapping = members.iter().map(|id| (*id, *id)).collect();
                }
            } else {
                *scalar_mapping = scalar_mapping
                    .iter()
                    .filter_map(|(a, b)| Some((remap(*a)?, remap(*b)?)))
                    .collect();
            }
        }
    }
    for (id, mut record) in finals {
        let Some(previous) = model.electrical_identities.get(&id) else {
            continue;
        };
        let explicit = operations.iter_mut().find_map(|op| match op {
            Operation::SetElectricalIdentity { record, .. } if record.id == id => Some(record),
            _ => None,
        });
        if let Some(current) = explicit {
            record.object_revision = current.object_revision;
            *current = record;
        } else if previous.identity != record.identity {
            record.object_revision = super::electrical_transaction::next(previous.object_revision)?;
            operations.push(Operation::SetElectricalIdentity {
                previous: Box::new(previous.clone()),
                record,
            });
        }
    }
    Ok(())
}
