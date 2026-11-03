//! Complete selected-set deletion; only the Proposal owner exposes this builder.
use super::{
    context::{BatchComposer, PreparedWrite, WriteProvenance},
    registry::NativeWriteContext,
    schematic_bus::admit_bus_delete,
    schematic_connectivity::*,
    schematic_delete_proposal::{DeleteClass, DeleteTarget, DeletedObject},
    schematic_selected_source::SelectedSchematicSource,
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
    let raw_sheet = super::schematic_symbol_payload::source_sheet_payload(
        context.model,
        sheet_id,
        targets[0].id,
    )?;
    let removed_entries: BTreeSet<_> = sheet
        .bus_entries
        .values()
        .filter(|e| selected.contains(&(DeleteClass::BusSection, e.bus)))
        .map(|e| e.uuid)
        .collect();
    let symbol_ids: BTreeSet<_> = selected
        .iter()
        .filter_map(|(class, id)| (*class == DeleteClass::Symbol).then_some(*id))
        .collect();
    let children = owned_symbol_objects(context.model, sheet, &symbol_ids, &raw_sheet)?;
    let bindings =
        super::schematic_assignment::delete_bindings(context.model, &symbol_ids, &children)?;
    let mut operations: Vec<_> = bindings
        .values()
        .map(|(id, raw)| Operation::DeleteComponentInstance {
            component_instance_id: *id,
            component_instance: raw.clone(),
        })
        .collect();
    let mut append = |prepared: PreparedWrite| {
        operations.extend(prepared.batch.operations);
    };
    for (class, id) in selected {
        match class {
            DeleteClass::Symbol => {
                let symbol = &sheet.symbols[&id];
                let raw = super::schematic_symbol_payload::member_payload(
                    &raw_sheet, "symbols", id, symbol,
                )?;
                append(
                    BatchComposer::compose(context.model, provenance.clone())
                        .push_op(Operation::DeleteSchematicSymbol {
                            sheet_id,
                            symbol_id: id,
                            symbol: raw,
                        })
                        .finish()?,
                );
                for marker in sheet.noconnects.values().filter(|n| n.symbol == id) {
                    if source.sheet("noconnects", marker.uuid, path)? != sheet_id
                        || !symbol.pins.iter().any(|p| p.uuid == marker.pin)
                    {
                        return Err(invalid(
                            "No-connect marker has inconsistent symbol/pin ownership",
                        ));
                    }
                    let raw = super::schematic_symbol_payload::member_payload(
                        &raw_sheet,
                        "noconnects",
                        marker.uuid,
                        marker,
                    )?;
                    append(
                        BatchComposer::compose(context.model, provenance.clone())
                            .push_op(Operation::DeleteSchematicNoConnect {
                                sheet_id,
                                noconnect_id: marker.uuid,
                                noconnect: raw,
                            })
                            .finish()?,
                    );
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
            DeleteClass::NoConnect => append(
                BatchComposer::compose(context.model, provenance.clone())
                    .push_op(Operation::DeleteSchematicNoConnect {
                        sheet_id,
                        noconnect_id: id,
                        noconnect: super::schematic_symbol_payload::member_payload(
                            &raw_sheet,
                            "noconnects",
                            id,
                            &sheet.noconnects[&id],
                        )?,
                    })
                    .finish()?,
            ),
            DeleteClass::SchematicText => append(super::schematic_text::delete_in_sheet(
                context,
                provenance.clone(),
                sheet_id,
                &sheet.texts[&id],
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
            Operation::DeleteSchematicBusEntry { .. }
                | Operation::DeleteSchematicNoConnect { .. }
                | Operation::DeleteComponentInstance { .. }
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
        Operation::DeleteComponentInstance {
            component_instance_id,
            component_instance,
        } => (
            *component_instance_id,
            "component_instance",
            component_instance,
            "",
        ),
        Operation::DeleteSchematicSymbol {
            symbol_id, symbol, ..
        } => (*symbol_id, "symbol", symbol, "reference"),
        Operation::DeleteSchematicText { text_id, text, .. } => {
            (*text_id, "schematic_text", text, "text")
        }
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

/// Recorded Proposal qualification never infers ownership from an opaque ID alone.
pub(super) fn validate_owned_components(ops: &[Operation]) -> Result<(), EngineError> {
    let symbols: BTreeSet<_> = ops
        .iter()
        .filter_map(|op| match op {
            Operation::DeleteSchematicSymbol { symbol_id, .. } => Some(*symbol_id),
            _ => None,
        })
        .collect();
    let mut owners = BTreeSet::new();
    for op in ops {
        if let Operation::DeleteComponentInstance {
            component_instance_id,
            component_instance,
        } = op
        {
            let ci: crate::substrate::ComponentInstanceShard = serde_json::from_value(
                serde_json::json!({"schema_version":1,"component_instance":component_instance}),
            )?;
            let ci = ci.component_instance;
            if ci.uuid != *component_instance_id
                || ci.placed_symbol_refs.len() != 1
                || !ci.placed_package_refs.is_empty()
                || !symbols.contains(&ci.placed_symbol_refs[0].object_id)
                || !owners.insert(ci.placed_symbol_refs[0].object_id)
                || ci.library_bindings.values().any(|b| {
                    !matches!(
                        b.binding_role,
                        crate::substrate::LibraryBindingRole::Part
                            | crate::substrate::LibraryBindingRole::Symbol
                    ) || !b.local_override_refs.is_empty()
                })
            {
                return Err(invalid(
                    "Deletion requires an exact owned schematic-only component",
                ));
            }
        }
    }
    Ok(())
}

/// The canonical delete's complete recursive identity set, shared by invocation
/// and complete-set admission. Typed fields alone omit retained extension UUIDs.
pub(super) fn owned_symbol_objects(
    model: &crate::substrate::DesignModel,
    sheet: &crate::schematic::Sheet,
    symbols: &BTreeSet<Uuid>,
    raw_sheet: &serde_json::Value,
) -> Result<BTreeSet<Uuid>, EngineError> {
    let mut removed = BTreeSet::new();
    for id in symbols {
        let raw = super::schematic_symbol_payload::member_payload(
            raw_sheet,
            "symbols",
            *id,
            &sheet.symbols[id],
        )?;
        removed.extend(model.schematic_member_object_ids(sheet.uuid, "symbols", *id, &raw)?);
    }
    for marker in sheet
        .noconnects
        .values()
        .filter(|m| symbols.contains(&m.symbol))
    {
        let raw = super::schematic_symbol_payload::member_payload(
            raw_sheet,
            "noconnects",
            marker.uuid,
            marker,
        )?;
        removed.extend(model.schematic_member_object_ids(
            sheet.uuid,
            "noconnects",
            marker.uuid,
            &raw,
        )?);
    }
    Ok(removed)
}
