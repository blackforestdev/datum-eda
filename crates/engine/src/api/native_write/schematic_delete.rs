//! Complete selected-set deletion; only the Proposal owner exposes this builder.
use super::{
    context::{BatchComposer, PreparedWrite, WriteProvenance},
    registry::NativeWriteContext,
    schematic_bus::admit_bus_delete,
    schematic_connectivity::*,
    schematic_delete_proposal::{DeleteClass, DeleteTarget, DeletedObject},
    schematic_selected_source::SelectedSchematicSource,
    schematic_selection_edit::require_unbound_symbol,
    schematic_symbols::build_delete_schematic_symbol,
};
use crate::{error::EngineError, substrate::Operation};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

pub(super) fn build(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    sheet_id: Uuid,
    path: &[Uuid],
    targets: &[DeleteTarget],
) -> Result<PreparedWrite, EngineError> {
    if targets.is_empty() {
        return Err(invalid("Select schematic objects to delete"));
    }
    let source = SelectedSchematicSource::capture(context)?;
    let mut selected = BTreeSet::new();
    for target in targets {
        if !selected.insert((target.class, target.id)) {
            return Err(invalid("Selection contains a duplicate target"));
        }
        if source.sheet(target.class.source_class(), target.id, path)? != sheet_id {
            return Err(invalid(
                "Delete requires one complete source sheet occurrence",
            ));
        }
    }
    let sheet = &source.source.sheets[&sheet_id];
    let removed_entries: BTreeSet<_> = sheet
        .bus_entries
        .values()
        .filter(|e| selected.contains(&(DeleteClass::BusSection, e.bus)))
        .map(|e| e.uuid)
        .collect();
    let mut operations = Vec::new();
    let mut append = |prepared: PreparedWrite| {
        operations.extend(prepared.batch.operations);
    };
    for (class, id) in selected {
        match class {
            DeleteClass::Symbol => {
                require_unbound_symbol(context, id)?;
                let symbol = &sheet.symbols[&id];
                append(build_delete_schematic_symbol(
                    context.model,
                    provenance.clone(),
                    sheet_id,
                    symbol,
                )?);
                for marker in sheet.noconnects.values().filter(|n| n.symbol == id) {
                    if source.sheet("noconnects", marker.uuid, path)? != sheet_id
                        || !symbol.pins.iter().any(|p| p.uuid == marker.pin)
                    {
                        return Err(invalid(
                            "No-connect marker has inconsistent symbol/pin ownership",
                        ));
                    }
                    append(build_delete_schematic_noconnect(
                        context.model,
                        provenance.clone(),
                        sheet_id,
                        marker,
                    )?);
                }
            }
            DeleteClass::Wire => {
                if source
                    .source
                    .sheets
                    .values()
                    .flat_map(|s| s.bus_entries.values())
                    .any(|e| e.wire == Some(id) && !removed_entries.contains(&e.uuid))
                {
                    return Err(invalid(
                        "Wire has a surviving Bus Entry; include its owning Bus or remove the entry first",
                    ));
                }
                append(build_delete_schematic_wire(
                    context.model,
                    provenance.clone(),
                    sheet_id,
                    &sheet.wires[&id],
                )?);
            }
            DeleteClass::Label => append(build_delete_schematic_label(
                context.model,
                provenance.clone(),
                sheet_id,
                &sheet.labels[&id],
            )?),
            DeleteClass::Port => {
                if source
                    .source
                    .sheet_instances
                    .values()
                    .any(|i| i.ports.contains(&id))
                {
                    return Err(invalid(
                        "Port is bound to a sheet instance; remove its hierarchy binding first",
                    ));
                }
                append(build_delete_schematic_port(
                    context.model,
                    provenance.clone(),
                    sheet_id,
                    &sheet.ports[&id],
                )?);
            }
            DeleteClass::Junction => append(build_delete_schematic_junction(
                context.model,
                provenance.clone(),
                sheet_id,
                &sheet.junctions[&id],
            )?),
            DeleteClass::NoConnect => append(build_delete_schematic_noconnect(
                context.model,
                provenance.clone(),
                sheet_id,
                &sheet.noconnects[&id],
            )?),
            DeleteClass::BusSection => {
                if source.source.sheets.iter().any(|(owner, s)| {
                    *owner != sheet_id && s.bus_entries.values().any(|e| e.bus == id)
                }) {
                    return Err(invalid("Bus has an entry in a different source sheet"));
                }
                let entries: Vec<_> = sheet.bus_entries.values().filter(|e| e.bus == id).collect();
                admit_bus_delete(context, id, path, &entries)?;
                for entry in entries {
                    if source.sheet("bus_entries", entry.uuid, path)? != sheet_id {
                        return Err(invalid("Bus Entry source ownership is ambiguous"));
                    }
                    append(build_delete_schematic_bus_entry(
                        context.model,
                        provenance.clone(),
                        sheet_id,
                        entry,
                    )?);
                }
                append(build_delete_schematic_bus(
                    context.model,
                    provenance.clone(),
                    sheet_id,
                    &sheet.buses[&id],
                )?);
            }
        }
    }
    // A selected child and its selected owner describe one deletion, not two.
    // Recompose guards exactly once against the original captured model.
    let mut unique: BTreeMap<Uuid, Operation> = BTreeMap::new();
    let mut lifecycle = Vec::new();
    for operation in operations {
        if matches!(operation, Operation::GuardObjectRevision { .. }) {
            continue;
        }
        if matches!(operation, Operation::SetElectricalIdentity { .. }) {
            lifecycle.push(operation);
            continue;
        }
        let deleted = deleted_object(&operation)
            .ok_or_else(|| invalid("Deletion builder emitted an unexpected operation"))?;
        if let Some(previous) = unique.get(&deleted.id) {
            if previous != &operation {
                return Err(invalid("Owned-child deletion payloads disagree"));
            }
        } else {
            unique.insert(deleted.id, operation);
        }
    }
    let sources: BTreeSet<_> = unique
        .values()
        .filter_map(|op| match op {
            Operation::DeleteSchematicLabel { label_id, .. } => {
                Some(("labels".to_string(), *label_id))
            }
            Operation::DeleteSchematicPort { port_id, .. } => Some(("ports".to_string(), *port_id)),
            Operation::DeleteSchematicBus { bus_id, .. } => Some(("buses".to_string(), *bus_id)),
            Operation::DeleteSchematicBusEntry { bus_entry_id, .. } => {
                Some(("bus_entries".to_string(), *bus_entry_id))
            }
            _ => None,
        })
        .collect();
    let lifecycle =
        super::electrical::merge_projection_removals(context.model, &sources, lifecycle)?;
    // Undo reverses this order. Restore Bus/Wire before their entries and Symbol
    // before its no-connects, regardless of the source UUID ordering. Only checked
    // source deletions reach this list; UUID remains the deterministic tiebreaker.
    let mut source_deletions: Vec<_> = unique.into_iter().collect();
    source_deletions.sort_by_key(|(id, operation)| {
        let owned_child = matches!(
            operation,
            Operation::DeleteSchematicBusEntry { .. } | Operation::DeleteSchematicNoConnect { .. }
        );
        (!owned_child, *id)
    });
    BatchComposer::compose(context.model, provenance)
        .push_ops(source_deletions.into_iter().map(|(_, operation)| operation))
        .push_ops(lifecycle)
        .finish()
}

pub(super) fn deleted_object(op: &Operation) -> Option<DeletedObject> {
    let (id, class, payload, key) = match op {
        Operation::DeleteSchematicSymbol {
            symbol_id, symbol, ..
        } => (*symbol_id, "symbol", symbol, "reference"),
        Operation::DeleteSchematicWire { wire_id, wire, .. } => (*wire_id, "wire", wire, ""),
        Operation::DeleteSchematicLabel {
            label_id, label, ..
        } => (*label_id, "label", label, "name"),
        Operation::DeleteSchematicPort { port_id, port, .. } => (*port_id, "port", port, "name"),
        Operation::DeleteSchematicBus { bus_id, bus, .. } => (*bus_id, "bus_section", bus, "name"),
        Operation::DeleteSchematicBusEntry {
            bus_entry_id,
            bus_entry,
            ..
        } => (*bus_entry_id, "bus_entry", bus_entry, ""),
        Operation::DeleteSchematicJunction {
            junction_id,
            junction,
            ..
        } => (*junction_id, "junction", junction, ""),
        Operation::DeleteSchematicNoConnect {
            noconnect_id,
            noconnect,
            ..
        } => (*noconnect_id, "no_connect", noconnect, ""),
        _ => return None,
    };
    let name = payload
        .get(key)
        .and_then(serde_json::Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_default();
    Some(DeletedObject {
        id,
        class: class.into(),
        name,
    })
}
fn invalid(reason: &str) -> EngineError {
    EngineError::Validation(reason.into())
}
