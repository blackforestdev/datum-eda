//! Semantic Bus membership and source drawing lifecycle.
use super::*;
use crate::substrate::{
    ElectricalIdentity, ElectricalIdentityRecord, ElectricalOccurrence, ObjectRevision,
};

pub(super) fn origin(class: &str, id: Uuid, path: &[Uuid]) -> ElectricalOccurrence {
    ElectricalOccurrence {
        class: class.into(),
        source_id: id,
        instance_path: path.to_vec(),
    }
}

pub(super) fn qualify_wire(
    snapshot: &ElectricalSelectionSnapshot,
    wire: Uuid,
    path: &[Uuid],
    nets: &BTreeSet<Uuid>,
) -> Result<(), EngineError> {
    let net = snapshot
        .schematic_run(snapshot.revision(), &origin("wires", wire, path))
        .map_err(|e| invalid(format!("Member wire has no canonical schematic Net: {e:?}")))?;
    if !nets.contains(&net.subject_id) {
        return Err(invalid("Wire is not on a declared scalar bus member"));
    }
    Ok(())
}

pub(super) fn declaration(
    context: &NativeWriteContext<'_>,
    snapshot: &ElectricalSelectionSnapshot,
    provenance: WriteProvenance,
    bus: &Bus,
    path: &[Uuid],
    nets: BTreeSet<Uuid>,
    scalar_declarations: BTreeSet<ElectricalOccurrence>,
    editing: bool,
) -> Result<Vec<crate::substrate::Operation>, EngineError> {
    let reference = origin("buses", bus.uuid, path);
    let identity = ElectricalIdentity::Bus {
        scalar_declarations,
        name: bus.name.clone(),
        scalar_nets: nets,
        representations: BTreeSet::from([reference.clone()]),
        retired: false,
    };
    if !editing {
        return Ok(super::super::electrical::build_create_electrical_identity(
            context.model,
            provenance,
            ElectricalIdentityRecord {
                id: Uuid::new_v4(),
                object_revision: ObjectRevision(0),
                identity,
            },
        )?
        .batch
        .operations);
    }
    let membership = snapshot
        .bus_for_origin(snapshot.revision(), &reference)
        .map_err(|e| {
            invalid(format!(
                "Bus needs one canonical declaration before editing: {e:?}"
            ))
        })?;
    let previous = &context.model.electrical_identities[&membership.subject_id];
    let ElectricalIdentity::Bus {
        representations, ..
    } = &previous.identity
    else {
        unreachable!()
    };
    if representations != &BTreeSet::from([reference]) {
        return Err(invalid(
            "Shared Bus declarations require an explicit semantic edit",
        ));
    }
    if previous.identity == identity {
        return Ok(Vec::new());
    }
    let mut record = previous.clone();
    record.identity = identity;
    Ok(
        super::super::electrical::build_set_electrical_identity(context.model, provenance, record)?
            .batch
            .operations,
    )
}

// A native Bus owns its entries, while explicit shared/interface declarations need
// a semantic rebind. Deleting this source removes projections, not scalar Nets.
pub(in crate::api::native_write) fn admit_delete(
    context: &NativeWriteContext<'_>,
    bus: Uuid,
    path: &[Uuid],
    entries: &[&BusEntry],
) -> Result<(), EngineError> {
    let reference = origin("buses", bus, path);
    let mut declarations = BTreeSet::new();
    for record in context.model.electrical_identities.values() {
        if let ElectricalIdentity::Bus {
            representations,
            retired: false,
            ..
        } = &record.identity
        {
            if representations.contains(&reference) {
                if representations != &BTreeSet::from([reference.clone()]) {
                    return Err(invalid(
                        "Shared Bus declarations require an explicit semantic rebind before deletion",
                    ));
                }
                declarations.insert(record.id);
            }
            if representations
                .iter()
                .any(|r| r.class == "bus_entries" && entries.iter().any(|e| e.uuid == r.source_id))
            {
                return Err(invalid(
                    "Bus Entry has an explicit semantic binding; rebind it before deleting its Bus",
                ));
            }
        }
    }
    if declarations.len() != 1 {
        return Err(invalid(
            "Bus deletion requires one canonical source declaration",
        ));
    }
    if context.model.electrical_identities.values().any(|record| {
        matches!(
            &record.identity, ElectricalIdentity::BusInterface { left_bus, right_bus, .. }
            if declarations.contains(left_bus) || declarations.contains(right_bus)
        )
    }) {
        return Err(invalid(
            "Bus has an interface binding; rebind it before deletion",
        ));
    }
    Ok(())
}
