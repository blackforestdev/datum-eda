//! Final-state validation shared by native writes and resolver materialization.
use super::{
    BusInterfaceEquivalence, DesignModel, ElectricalIdentity, ElectricalOccurrence, EngineError,
    NetCorrespondenceStatus, NetRelationshipIntent, RevisionedRef,
};
use std::collections::BTreeMap;
use uuid::Uuid;

pub(super) fn validate(model: &DesignModel) -> Result<(), EngineError> {
    validate_operations(model, &[])
}

pub(super) fn validate_operations(
    model: &DesignModel,
    operations: &[super::Operation],
) -> Result<(), EngineError> {
    if model.electrical_identities.is_empty() {
        return Ok(());
    }
    let basis = super::electrical_basis::ElectricalBasis::new(model, operations)?;
    if model.electrical_identities.values().any(|r| matches!(&r.identity, ElectricalIdentity::Net {anchor,retired:false,..} if super::electrical_transaction::schematic_class(&anchor.class))) {
        let groups=super::electrical_topology_source::partitions(model,operations)?;
        super::electrical_transaction::previous(model,&groups)?;
    }
    let mut owners = BTreeMap::new();
    let mut net_anchors = BTreeMap::new();
    for record in model.electrical_identities.values() {
        if record.id.is_nil() {
            return Err(invalid("nil identity"));
        }
        match &record.identity {
            ElectricalIdentity::Net {
                anchor, retired, ..
            } => {
                if !retired {
                    basis.occurrence(model, anchor)?;
                    if net_anchors.insert(anchor, record.id).is_some() {
                        return Err(invalid("active Net anchor has multiple identity owners"));
                    }
                }
            }
            ElectricalIdentity::Bus {
                scalar_nets,
                representations,
                retired,
                ..
            } => {
                if *retired {
                    continue;
                }
                for id in scalar_nets {
                    net(model, *id)?;
                }
                for reference in representations {
                    basis.occurrence(model, reference)?;
                    if !matches!(
                        reference.class.as_str(),
                        "buses" | "bus_entries" | "labels" | "ports"
                    ) {
                        return Err(invalid("invalid Bus representation class"));
                    }
                    if owners.insert(reference, record.id).is_some() {
                        return Err(invalid("Bus occurrence has multiple owners"));
                    }
                }
            }
            ElectricalIdentity::NetRelationship {
                logical_net,
                board_net,
                intent,
                evidence,
            } => {
                if matches!(
                    intent,
                    NetRelationshipIntent::Pending | NetRelationshipIntent::Mismatch
                ) {
                    if !matches!(
                        model
                            .electrical_identities
                            .get(logical_net)
                            .map(|r| &r.identity),
                        Some(ElectricalIdentity::Net { .. })
                    ) {
                        return Err(invalid("missing historical logical Net"));
                    }
                } else {
                    net(model, *logical_net)?;
                }
                if let Some(reference) = board_net {
                    let object = model
                        .objects
                        .get(&reference.object_id)
                        .ok_or_else(|| invalid("missing board Net"))?;
                    if !basis.class(model, object.object_id, "nets", "board") {
                        return Err(invalid("relationship endpoint is not board Net"));
                    }
                }
                if matches!(
                    intent,
                    NetRelationshipIntent::Implemented | NetRelationshipIntent::BoardOnly
                ) && board_net.is_none()
                {
                    return Err(invalid("missing board endpoint"));
                }
                if *intent == NetRelationshipIntent::SchematicOnly && board_net.is_some() {
                    return Err(invalid("schematic-only has board endpoint"));
                }
                if *intent == NetRelationshipIntent::Implemented && evidence.is_empty() {
                    return Err(invalid(
                        "implemented correspondence needs explicit pin/pad evidence",
                    ));
                }
                let source_basis = &basis;
                for basis in evidence {
                    let ci = model
                        .component_instances
                        .get(&basis.component_instance.object_id)
                        .ok_or_else(|| invalid("missing ComponentInstance"))?;
                    let pin_map = model
                        .objects
                        .get(&basis.pin_pad_map.object_id)
                        .ok_or_else(|| invalid("missing PinPadMap"))?;
                    if !source_basis.class(model, pin_map.object_id, "pin_pad_maps", "pool") {
                        return Err(invalid("invalid PinPadMap source"));
                    }
                    if !ci.library_bindings.values().any(|b| {
                        b.target_object_id == basis.pin_pad_map.object_id
                            && b.pinned_object_revision == basis.pin_pad_map.object_revision
                            && b.binding_role == super::LibraryBindingRole::PinPadMap
                    }) {
                        return Err(invalid("PinPadMap not pinned by ComponentInstance"));
                    }
                    source_basis.occurrence(model, &basis.schematic_terminal)?;
                    let pad = model
                        .objects
                        .get(&basis.board_pad.object_id)
                        .ok_or_else(|| invalid("missing board pad"))?;
                    if !source_basis.class(model, pad.object_id, "pads", "board") {
                        return Err(invalid("invalid board pad"));
                    }
                    if *intent == NetRelationshipIntent::Implemented
                        && super::electrical_correspondence::certify(
                            model,
                            source_basis,
                            basis,
                            board_net.as_ref().map(|b| b.object_id),
                        )? == NetCorrespondenceStatus::Mismatch
                    {
                        return Err(invalid(
                            "correspondence contradicts placement/PinPadMap source",
                        ));
                    }
                }
            }
            ElectricalIdentity::BusInterface {
                left_bus,
                right_bus,
                left_occurrence,
                right_occurrence,
                equivalence,
                scalar_mapping,
            } => {
                let (left_members, left_refs) = bus(model, *left_bus)?;
                let (right_members, right_refs) = bus(model, *right_bus)?;
                if !left_refs.contains(left_occurrence) || !right_refs.contains(right_occurrence) {
                    return Err(invalid("interface occurrence is not bound to Bus"));
                }
                match equivalence {
                    BusInterfaceEquivalence::Identity if left_bus != right_bus => {
                        return Err(invalid("identity interface must name same declaration"));
                    }
                    BusInterfaceEquivalence::Related if left_bus == right_bus => {
                        return Err(invalid("related interface must keep distinct declarations"));
                    }
                    _ => {}
                }
                for (left, right) in scalar_mapping {
                    if !left_members.contains(left) || !right_members.contains(right) {
                        return Err(invalid("interface scalar mapping outside declaration"));
                    }
                    if *equivalence == BusInterfaceEquivalence::Identity && left != right {
                        return Err(invalid("identity interface cannot remap roles"));
                    }
                }
                if *equivalence == BusInterfaceEquivalence::Identity
                    && scalar_mapping != &left_members.iter().map(|id| (*id, *id)).collect()
                {
                    return Err(invalid("identity interface needs full member mapping"));
                }
            }
        }
    }
    Ok(())
}

fn net(model: &DesignModel, id: Uuid) -> Result<(), EngineError> {
    if matches!(
        model.electrical_identities.get(&id).map(|r| &r.identity),
        Some(ElectricalIdentity::Net { retired: false, .. })
    ) {
        Ok(())
    } else {
        Err(invalid("missing/retired logical Net"))
    }
}

fn bus(
    model: &DesignModel,
    id: Uuid,
) -> Result<
    (
        &std::collections::BTreeSet<Uuid>,
        &std::collections::BTreeSet<ElectricalOccurrence>,
    ),
    EngineError,
> {
    if let Some(ElectricalIdentity::Bus {
        scalar_nets,
        representations,
        retired: false,
        ..
    }) = model.electrical_identities.get(&id).map(|r| &r.identity)
    {
        Ok((scalar_nets, representations))
    } else {
        Err(invalid("missing/retired Bus declaration"))
    }
}

pub fn net_correspondence_status(model: &DesignModel, net_id: Uuid) -> NetCorrespondenceStatus {
    let mut found = false;
    let mut unverified = false;
    let source_basis = super::electrical_basis::ElectricalBasis::new(model, &[]);
    let mut board_owners = std::collections::BTreeSet::new();
    for record in model.electrical_identities.values() {
        if let ElectricalIdentity::NetRelationship {
            logical_net,
            board_net,
            intent,
            evidence,
        } = &record.identity
        {
            if *logical_net != net_id {
                continue;
            }
            found = true;

            if *intent == NetRelationshipIntent::Pending {
                return NetCorrespondenceStatus::Pending;
            }
            if *intent == NetRelationshipIntent::Mismatch {
                return NetCorrespondenceStatus::Mismatch;
            }
            if let Some(reference) = board_net {
                if !current(model, reference) {
                    return NetCorrespondenceStatus::Stale;
                }
                if !board_owners.insert(reference.object_id) {
                    return NetCorrespondenceStatus::Mismatch;
                }
                if model.electrical_identities.values().any(|r| matches!(&r.identity, ElectricalIdentity::NetRelationship { logical_net: other, board_net: Some(b), intent: NetRelationshipIntent::Implemented, .. } if *other != net_id && b.object_id == reference.object_id)) { return NetCorrespondenceStatus::Mismatch }
            }
            for basis in evidence {
                if !current(model, &basis.component_instance)
                    || !current(model, &basis.pin_pad_map)
                    || !current(model, &basis.board_pad)
                {
                    return NetCorrespondenceStatus::Stale;
                }
                if *intent == NetRelationshipIntent::Implemented {
                    let status = source_basis
                        .as_ref()
                        .ok()
                        .and_then(|source| {
                            super::electrical_correspondence::certify(
                                model,
                                source,
                                basis,
                                board_net.as_ref().map(|b| b.object_id),
                            )
                            .ok()
                        })
                        .unwrap_or(NetCorrespondenceStatus::Unverified);
                    match status {
                        NetCorrespondenceStatus::Complete => {}
                        NetCorrespondenceStatus::Unverified => unverified = true,
                        other => return other,
                    }
                }
            }
            unverified |= *intent == NetRelationshipIntent::Implemented && evidence.is_empty();
        }
    }
    if unverified {
        NetCorrespondenceStatus::Unverified
    } else if found {
        NetCorrespondenceStatus::Complete
    } else {
        NetCorrespondenceStatus::Absent
    }
}

fn current(model: &DesignModel, reference: &RevisionedRef) -> bool {
    model
        .objects
        .get(&reference.object_id)
        .is_some_and(|o| o.object_revision == reference.object_revision)
}
fn invalid(reason: &str) -> EngineError {
    EngineError::Validation(format!("invalid electrical authority: {reason}"))
}
