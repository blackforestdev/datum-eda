//! Native bus authoring over canonical source and explicit existing scalar-Net identities.
use super::{
    context::{BatchComposer, PreparedWrite, WriteProvenance},
    registry::{NativeWriteContext, NativeWriteVerb, parse_verb_params},
    schematic_connectivity::{
        build_create_schematic_bus, build_create_schematic_bus_entry, build_delete_schematic_bus,
        build_delete_schematic_bus_entry, build_set_schematic_bus,
    },
    schematic_edit::prepare_native_birth,
    schematic_selection_edit::selected_source,
};
use crate::{
    error::EngineError,
    ir::geometry::Point,
    schematic::{Bus, BusEntry},
    substrate::ElectricalSelectionSnapshot,
};
use serde::Deserialize;
use std::collections::BTreeSet;
use uuid::Uuid;
#[path = "schematic_bus_geometry.rs"]
mod geometry;
#[path = "schematic_bus_members.rs"]
mod members;
pub use geometry::{WireEnd, entry_geometry};

pub(crate) const VERBS: &[NativeWriteVerb] = &[
    NativeWriteVerb {
        id: "datum.schematic.create_bus",
        build: create_bus,
    },
    NativeWriteVerb {
        id: "datum.schematic.place_bus_entry",
        build: place_entry,
    },
    NativeWriteVerb {
        id: "datum.schematic.delete_bus",
        build: delete_bus,
    },
    NativeWriteVerb {
        id: "datum.schematic.delete_bus_entry",
        build: delete_entry,
    },
];
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Params {
    sheet: Uuid,
    bus: Option<Uuid>,
    instance_path: Option<Vec<Uuid>>,
    name: Option<String>,
    members: Option<Vec<String>>,
    points: Option<Vec<Point>>,
    #[serde(default)]
    entries: Vec<Entry>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    wire: Uuid,
    x_nm: i64,
    y_nm: i64,
    wire_end: WireEnd,
}

fn create_bus(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    let params: Params = parse_verb_params("datum.schematic.create_bus", params)?;
    prepare(context, provenance, params)
}
fn place_entry(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    let params: Params = parse_verb_params("datum.schematic.place_bus_entry", params)?;
    if params.bus.is_none()
        || params.entries.is_empty()
        || params.name.is_some()
        || params.members.is_some()
        || params.points.is_some()
    {
        return Err(invalid(
            "Bus Entry requires an existing bus and entries only",
        ));
    }
    prepare(context, provenance, params)
}
fn prepare(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: Params,
) -> Result<PreparedWrite, EngineError> {
    let snapshot = ElectricalSelectionSnapshot::capture(context.model)
        .map_err(|e| invalid(format!("Native bus source unavailable: {e:?}")))?;
    let source = snapshot
        .schematic_source()
        .ok_or_else(|| invalid("Native schematic unavailable"))?;
    let sheet = source
        .sheets
        .get(&params.sheet)
        .filter(|s| s.uuid == params.sheet)
        .ok_or_else(|| invalid("Native sheet not found"))?;
    let path = params.instance_path.as_deref().unwrap_or(&[]);
    let original = if let Some(id) = params.bus {
        if params.instance_path.is_none() {
            return Err(invalid("Bus edit requires its occurrence path"));
        }
        let (_, owner) = selected_source(context, "buses", id, path)?;
        if owner != params.sheet {
            return Err(invalid("Bus belongs to a different source sheet"));
        }
        Some(&sheet.buses[&id])
    } else {
        if !path.is_empty() {
            return Err(invalid(
                "Bus creation requires the supported source-sheet view",
            ));
        }
        None
    };
    let bus = Bus {
        uuid: original.map_or_else(Uuid::new_v4, |b| b.uuid),
        name: params
            .name
            .or_else(|| original.map(|b| b.name.clone()))
            .ok_or_else(|| invalid("Bus name required"))?,
        members: params
            .members
            .or_else(|| original.map(|b| b.members.clone()))
            .ok_or_else(|| invalid("Bus members required"))?,
        segments: params
            .points
            .or_else(|| original.map(|b| b.segments.clone()))
            .ok_or_else(|| invalid("Bus points required"))?,
    };
    geometry::validate_bus(&bus)?;
    let nets = members::member_nets(&snapshot, sheet, &bus.members, path)?;
    // Existing entries must stay attached to a current wire on a retained member.
    let mut occupied = BTreeSet::new();
    for entry in sheet.bus_entries.values().filter(|e| e.bus == bus.uuid) {
        let wire_id = entry
            .wire
            .ok_or_else(|| invalid("Existing Bus Entry has no qualified member wire"))?;
        let wire = sheet
            .wires
            .get(&wire_id)
            .ok_or_else(|| invalid("Existing Bus Entry wire is missing"))?;
        members::qualify_wire(&snapshot, wire_id, path, &nets)?;
        let endpoint = Point::new(
            entry
                .position
                .x
                .checked_add(entry.size.x)
                .ok_or_else(|| invalid("Entry geometry overflows"))?,
            entry
                .position
                .y
                .checked_add(entry.size.y)
                .ok_or_else(|| invalid("Entry geometry overflows"))?,
        );
        let end = if endpoint == wire.from {
            WireEnd::From
        } else if endpoint == wire.to {
            WireEnd::To
        } else {
            return Err(invalid(
                "Existing Bus Entry no longer reaches its member wire endpoint",
            ));
        };
        entry_geometry(&bus, wire, entry.position, end)?;
        occupied.insert((wire_id, entry.position.x, entry.position.y));
    }
    let mut operations = Vec::new();
    if original.is_none() {
        operations.extend(
            build_create_schematic_bus(context.model, provenance.clone(), params.sheet, &bus)?
                .batch
                .operations,
        );
    } else if original != Some(&bus) {
        operations.extend(
            build_set_schematic_bus(context.model, provenance.clone(), params.sheet, &bus)?
                .batch
                .operations,
        );
    }
    operations.extend(members::declaration(
        context,
        &snapshot,
        provenance.clone(),
        &bus,
        path,
        nets.clone(),
        original.is_some(),
    )?);
    for entry in params.entries {
        let (_, owner) = selected_source(context, "wires", entry.wire, path)?;
        if owner != params.sheet {
            return Err(invalid("Member wire belongs to a different source sheet"));
        }
        let wire = &sheet.wires[&entry.wire];
        members::qualify_wire(&snapshot, entry.wire, path, &nets)?;
        let at = Point::new(entry.x_nm, entry.y_nm);
        let size = entry_geometry(&bus, wire, at, entry.wire_end)?;
        if !occupied.insert((entry.wire, at.x, at.y)) {
            return Err(invalid(
                "This wire already has an entry at this bus contact",
            ));
        }
        let value = BusEntry {
            uuid: Uuid::new_v4(),
            bus: bus.uuid,
            wire: Some(wire.uuid),
            position: at,
            size,
        };
        operations.extend(
            build_create_schematic_bus_entry(
                context.model,
                provenance.clone(),
                params.sheet,
                &value,
            )?
            .batch
            .operations,
        );
    }
    if operations.is_empty() {
        return Err(invalid("Bus fields are unchanged"));
    }
    let write = BatchComposer::compose(context.model, provenance)
        .push_ops(operations)
        .primary_object(bus.uuid)
        .finish()?;
    prepare_native_birth(context.model, write)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeleteBus {
    bus: Uuid,
    instance_path: Vec<Uuid>,
}
fn delete_bus(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    let params: DeleteBus = parse_verb_params("datum.schematic.delete_bus", params)?;
    let (source, sheet) = selected_source(context, "buses", params.bus, &params.instance_path)?;
    let entries: Vec<_> = source.sheets[&sheet]
        .bus_entries
        .values()
        .filter(|e| e.bus == params.bus)
        .collect();
    if source
        .sheets
        .iter()
        .any(|(id, value)| *id != sheet && value.bus_entries.values().any(|e| e.bus == params.bus))
    {
        return Err(invalid("Bus has an entry in a different source sheet"));
    }
    members::admit_delete(context, params.bus, &params.instance_path, &entries)?;
    let mut operations = Vec::new();
    for entry in entries {
        let (_, owner) =
            selected_source(context, "bus_entries", entry.uuid, &params.instance_path)?;
        if owner != sheet {
            return Err(invalid("Bus Entry source changed"));
        }
        operations.extend(
            build_delete_schematic_bus_entry(context.model, provenance.clone(), sheet, entry)?
                .batch
                .operations,
        );
    }
    operations.extend(
        build_delete_schematic_bus(
            context.model,
            provenance.clone(),
            sheet,
            &source.sheets[&sheet].buses[&params.bus],
        )?
        .batch
        .operations,
    );
    BatchComposer::compose(context.model, provenance)
        .push_ops(operations)
        .primary_object(params.bus)
        .finish()
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeleteEntry {
    bus: Uuid,
    bus_entry: Uuid,
    instance_path: Vec<Uuid>,
}
fn delete_entry(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    let params: DeleteEntry = parse_verb_params("datum.schematic.delete_bus_entry", params)?;
    let (source, sheet) = selected_source(
        context,
        "bus_entries",
        params.bus_entry,
        &params.instance_path,
    )?;
    let (_, bus_sheet) = selected_source(context, "buses", params.bus, &params.instance_path)?;
    let entry = &source.sheets[&sheet].bus_entries[&params.bus_entry];
    if sheet != bus_sheet || entry.bus != params.bus {
        return Err(invalid("Entry does not belong to the selected source Bus"));
    }
    build_delete_schematic_bus_entry(context.model, provenance, sheet, entry)
}
fn invalid(reason: impl Into<String>) -> EngineError {
    EngineError::Validation(reason.into())
}
#[cfg(test)]
#[path = "schematic_bus_tests.rs"]
mod tests;
