//! Occurrence-qualified selected-object edits through canonical schematic writers.
//!
//! Native schemas retain existing compatibility verb IDs. Deletion accepts
//! `{symbol|wire|label: UUID, instance_path: [UUID]}`. One parametrized
//! `datum.schematic.move` accepts `{object, instance_path, x_nm, y_nm}`: the
//! kind resolves from the actual source, the destination is the object's
//! anchor (junction point, or wire `from` with rigid `to` translation), and
//! placement-grade duplicate refusal applies to junctions. `set_symbol_reference`
//! requires `{symbol, instance_path, reference}` and optionally `value`;
//! `set_symbol_value` requires `value` and optionally `reference`. Both fields
//! commit together in one SetSchematicSymbol when supplied. The project root
//! and expected revision are supplied by the native write envelope.
use super::{
    context::{BatchComposer, PreparedWrite, WriteProvenance},
    registry::{NativeWriteContext, NativeWriteVerb, parse_verb_params},
    schematic_connectivity::{
        build_delete_schematic_label, build_delete_schematic_noconnect, build_delete_schematic_wire,
    },
    schematic_symbol_payload::{
        member_payload, set_point, source_sheet_payload, source_symbol_payload,
    },
    schematic_symbols::build_delete_schematic_symbol,
};
use crate::{error::EngineError, substrate::Operation};
use serde::Deserialize;
use uuid::Uuid;

pub(crate) const VERBS: &[NativeWriteVerb] = &[
    NativeWriteVerb {
        id: "datum.schematic.delete_symbol",
        build: delete_symbol,
    },
    NativeWriteVerb {
        id: "datum.schematic.delete_wire",
        build: delete_wire,
    },
    NativeWriteVerb {
        id: "datum.schematic.delete_label",
        build: delete_label,
    },
    NativeWriteVerb {
        id: "datum.schematic.move",
        build: move_selected,
    },
    NativeWriteVerb {
        id: "datum.schematic.set_symbol_reference",
        build: set_symbol_reference,
    },
    NativeWriteVerb {
        id: "datum.schematic.set_symbol_value",
        build: set_symbol_value,
    },
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeleteSymbolParams {
    symbol: Uuid,
    instance_path: Vec<Uuid>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeleteWireParams {
    wire: Uuid,
    instance_path: Vec<Uuid>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeleteLabelParams {
    label: Uuid,
    instance_path: Vec<Uuid>,
}

fn delete_symbol(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    let params: DeleteSymbolParams = parse_verb_params("datum.schematic.delete_symbol", params)?;
    delete_selected(
        context,
        provenance,
        "symbols",
        params.symbol,
        &params.instance_path,
    )
}
fn delete_wire(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    let params: DeleteWireParams = parse_verb_params("datum.schematic.delete_wire", params)?;
    delete_selected(
        context,
        provenance,
        "wires",
        params.wire,
        &params.instance_path,
    )
}
fn delete_label(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    let params: DeleteLabelParams = parse_verb_params("datum.schematic.delete_label", params)?;
    delete_selected(
        context,
        provenance,
        "labels",
        params.label,
        &params.instance_path,
    )
}

pub(super) use super::schematic_selected_source::selected_source;

pub(super) fn require_unbound_symbol(
    context: &NativeWriteContext<'_>,
    id: Uuid,
) -> Result<(), EngineError> {
    if context
        .model
        .component_instances
        .values()
        .any(|ci| ci.placed_symbol_refs.contains(&id))
        || super::schematic_assignment::delete_bindings(
            context.model,
            &std::collections::BTreeSet::from([id]),
            &std::collections::BTreeSet::new(),
        )?
        .contains_key(&id)
    {
        return Err(invalid(
            "This symbol is linked to a component; synchronized editing is not yet supported",
        ));
    }
    Ok(())
}

fn delete_selected(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    class: &str,
    id: Uuid,
    path: &[Uuid],
) -> Result<PreparedWrite, EngineError> {
    let (source, sheet_id) = selected_source(context, class, id, path)?;
    let sheet = &source.sheets[&sheet_id];
    match class {
        "symbols" => {
            require_unbound_symbol(context, id)?;
            let mut prepared = build_delete_schematic_symbol(
                context.model,
                provenance.clone(),
                sheet_id,
                &sheet.symbols[&id],
            )?;
            // Owned no-connect markers cannot outlive their symbol/pin target.
            let mut markers: Vec<_> = sheet
                .noconnects
                .values()
                .filter(|marker| marker.symbol == id)
                .collect();
            markers.sort_by_key(|marker| marker.uuid);
            for marker in markers {
                if !sheet.symbols[&id]
                    .pins
                    .iter()
                    .any(|pin| pin.uuid == marker.pin)
                {
                    return Err(invalid(
                        "No-connect marker has inconsistent symbol/pin ownership",
                    ));
                }
                prepared.batch.operations.extend(
                    build_delete_schematic_noconnect(
                        context.model,
                        provenance.clone(),
                        sheet_id,
                        marker,
                    )?
                    .batch
                    .operations,
                );
            }
            Ok(prepared)
        }
        "wires" => {
            if sheet
                .bus_entries
                .values()
                .any(|entry| entry.wire == Some(id))
            {
                return Err(invalid(
                    "Wire is referenced by a bus entry; remove the entry first",
                ));
            }
            build_delete_schematic_wire(context.model, provenance, sheet_id, &sheet.wires[&id])
        }
        "labels" => {
            build_delete_schematic_label(context.model, provenance, sheet_id, &sheet.labels[&id])
        }
        _ => Err(invalid("Unsupported schematic selection")),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MoveParams {
    object: Uuid,
    instance_path: Vec<Uuid>,
    x_nm: i64,
    y_nm: i64,
}

/// One parametrized connectivity move: the kind is resolved from the actual
/// source, never from a caller-declared class. The destination is the moved
/// object's anchor — a junction's point, or a wire's `from` endpoint with the
/// `to` endpoint translated rigidly. Unselected authored geometry stays fixed;
/// connectivity re-derives under the existing explicit-junction rules
/// (PM disposition on dat-ordinary-circuit-authoring-zvhw).
fn move_selected(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    let p: MoveParams = parse_verb_params("datum.schematic.move", params)?;
    let snapshot = ElectricalSelectionSnapshotRef::capture(context)?;
    let class = snapshot.movable_class(p.object)?;
    match class {
        "wires" => move_wire(context, provenance, &p),
        "junctions" => move_junction(context, provenance, &p),
        "ports" => move_port(context, provenance, &p),
        _ => Err(invalid(
            "Selected object is not a movable wire, junction or port",
        )),
    }
}

/// Minimal source probe for kind resolution; path admission stays with
/// `selected_source` so refusal wording is identical to the delete family.
struct ElectricalSelectionSnapshotRef {
    source: crate::schematic::Schematic,
}
impl ElectricalSelectionSnapshotRef {
    fn capture(context: &NativeWriteContext<'_>) -> Result<Self, EngineError> {
        let snapshot = crate::substrate::ElectricalSelectionSnapshot::capture(context.model)
            .map_err(|e| invalid(format!("Schematic source unavailable: {e:?}")))?;
        let source = snapshot
            .schematic_source()
            .ok_or_else(|| invalid("Native schematic source unavailable"))?
            .clone();
        Ok(Self { source })
    }
    fn movable_class(&self, id: Uuid) -> Result<&'static str, EngineError> {
        for sheet in self.source.sheets.values() {
            if sheet.wires.contains_key(&id) {
                return Ok("wires");
            }
            if sheet.junctions.contains_key(&id) {
                return Ok("junctions");
            }
            if sheet.ports.contains_key(&id) {
                return Ok("ports");
            }
        }
        Err(invalid(
            "Selected object is not a movable wire, junction or port",
        ))
    }
}

/// A raw `"locked": true` extension on the authored member refuses the move,
/// matching the selection view's lock derivation from the same raw source.
fn require_unlocked(member: &serde_json::Value) -> Result<(), EngineError> {
    if member.get("locked").and_then(serde_json::Value::as_bool) == Some(true) {
        return Err(invalid("Selected schematic object is locked"));
    }
    Ok(())
}

/// Position-only Port relocation reusing the existing Port coordinate-edit
/// Set semantics: name, direction, bindings and raw metadata are untouched
/// and unselected wires stay fixed.
fn move_port(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    p: &MoveParams,
) -> Result<PreparedWrite, EngineError> {
    let (source, sheet_id) = selected_source(context, "ports", p.object, &p.instance_path)?;
    let port = &source.sheets[&sheet_id].ports[&p.object];
    let position = crate::ir::geometry::Point::new(p.x_nm, p.y_nm);
    if port.position == position {
        return Err(invalid("Port move destination is unchanged"));
    }
    let raw = source_sheet_payload(context.model, sheet_id, p.object)?;
    let mut payload = member_payload(&raw, "ports", p.object, port)?;
    require_unlocked(&payload)?;
    set_point(&mut payload["position"], position);
    BatchComposer::compose(context.model, provenance)
        .push_op(Operation::SetSchematicPort {
            sheet_id,
            port_id: p.object,
            port: payload,
        })
        .primary_object(p.object)
        .finish()
}

/// Rigid identity-preserving translation of one authored wire segment.
/// Coincident geometry stays authored where it is; connectivity re-derives.
fn move_wire(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    p: &MoveParams,
) -> Result<PreparedWrite, EngineError> {
    let (source, sheet_id) = selected_source(context, "wires", p.object, &p.instance_path)?;
    let sheet = &source.sheets[&sheet_id];
    if sheet
        .bus_entries
        .values()
        .any(|entry| entry.wire == Some(p.object))
    {
        return Err(invalid(
            "Wire is referenced by a bus entry; remove the entry first",
        ));
    }
    let wire = &sheet.wires[&p.object];
    let (dx, dy) = (
        i128::from(p.x_nm) - i128::from(wire.from.x),
        i128::from(p.y_nm) - i128::from(wire.from.y),
    );
    if dx == 0 && dy == 0 {
        return Err(invalid("Wire move destination is unchanged"));
    }
    let translate = |point: crate::ir::geometry::Point| {
        let x = i128::from(point.x) + dx;
        let y = i128::from(point.y) + dy;
        match (i64::try_from(x), i64::try_from(y)) {
            (Ok(x), Ok(y)) => Ok(crate::ir::geometry::Point { x, y }),
            _ => Err(invalid("Wire moved coordinate overflow")),
        }
    };
    // Raw authored payload stays authoritative: Delete carries the exact
    // original bytes and Create the same clone with only the endpoints set
    // (the Symbol attached-wire pattern).
    let raw = source_sheet_payload(context.model, sheet_id, p.object)?;
    let original = member_payload(&raw, "wires", p.object, wire)?;
    require_unlocked(&original)?;
    let mut moved = original.clone();
    set_point(&mut moved["from"], translate(wire.from)?);
    set_point(&mut moved["to"], translate(wire.to)?);
    BatchComposer::compose(context.model, provenance)
        .push_op(Operation::DeleteSchematicWire {
            sheet_id,
            wire_id: p.object,
            wire: original,
        })
        .push_op(Operation::CreateSchematicWire {
            sheet_id,
            wire_id: p.object,
            wire: moved,
        })
        .primary_object(p.object)
        .finish()
}

/// Absolute identity-preserving junction relocation under placement admission.
fn move_junction(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    p: &MoveParams,
) -> Result<PreparedWrite, EngineError> {
    let (source, sheet_id) = selected_source(context, "junctions", p.object, &p.instance_path)?;
    let sheet = &source.sheets[&sheet_id];
    let junction = &sheet.junctions[&p.object];
    let position = crate::ir::geometry::Point::new(p.x_nm, p.y_nm);
    if junction.position == position {
        return Err(invalid("Junction move destination is unchanged"));
    }
    if sheet
        .junctions
        .values()
        .any(|other| other.uuid != p.object && other.position == position)
    {
        return Err(invalid("A junction already exists at this point"));
    }
    // Raw authored payload stays authoritative, exactly as for wires.
    let raw = source_sheet_payload(context.model, sheet_id, p.object)?;
    let original = member_payload(&raw, "junctions", p.object, junction)?;
    require_unlocked(&original)?;
    let mut moved = original.clone();
    set_point(&mut moved["position"], position);
    BatchComposer::compose(context.model, provenance)
        .push_op(Operation::DeleteSchematicJunction {
            sheet_id,
            junction_id: p.object,
            junction: original,
        })
        .push_op(Operation::PlaceSchematicMarker {
            sheet_id,
            marker_id: p.object,
            marker_kind: crate::substrate::SchematicMarkerKind::Junction,
            marker: moved,
        })
        .primary_object(p.object)
        .finish()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SymbolFieldsParams {
    pub(super) symbol: Uuid,
    pub(super) instance_path: Vec<Uuid>,
    pub(super) reference: Option<String>,
    pub(super) value: Option<String>,
}

fn set_symbol_reference(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    let params: SymbolFieldsParams =
        parse_verb_params("datum.schematic.set_symbol_reference", params)?;
    if params.reference.is_none() {
        return Err(invalid("reference is required"));
    }
    set_symbol_fields(context, provenance, params)
}
fn set_symbol_value(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    let params: SymbolFieldsParams = parse_verb_params("datum.schematic.set_symbol_value", params)?;
    if params.value.is_none() {
        return Err(invalid("value is required"));
    }
    set_symbol_fields(context, provenance, params)
}

pub(super) fn set_symbol_fields(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: SymbolFieldsParams,
) -> Result<PreparedWrite, EngineError> {
    let (source, sheet) =
        selected_source(context, "symbols", params.symbol, &params.instance_path)?;
    let original = &source.sheets[&sheet].symbols[&params.symbol];
    validate_field_values(params.reference.as_deref(), params.value.as_deref())?;
    let mut symbol = original.clone();
    if let Some(reference) = params.reference {
        if source
            .sheets
            .values()
            .flat_map(|sheet| sheet.symbols.values())
            .any(|other| other.uuid != symbol.uuid && other.reference == reference)
        {
            return Err(invalid("Symbol reference is already in use"));
        }
        symbol.reference = reference;
    }
    if let Some(value) = params.value {
        symbol.value = value;
    }
    if symbol == source.sheets[&sheet].symbols[&params.symbol] {
        return Err(invalid("Symbol fields are unchanged"));
    }
    let mut payload = source_symbol_payload(
        context.model,
        sheet,
        &source.sheets[&sheet].symbols[&params.symbol],
    )?;
    payload["reference"] = serde_json::Value::String(symbol.reference);
    payload["value"] = serde_json::Value::String(symbol.value);
    BatchComposer::compose(context.model, provenance)
        .push_op(Operation::SetSchematicSymbol {
            sheet_id: sheet,
            symbol_id: params.symbol,
            symbol: payload,
        })
        .push_ops(super::schematic_assignment::repin_symbols(
            context.model,
            std::slice::from_ref(original),
        )?)
        .primary_object(params.symbol)
        .finish()
}

pub(super) fn validate_field_values(
    reference: Option<&str>,
    value: Option<&str>,
) -> Result<(), EngineError> {
    if reference
        .is_some_and(|r| r.is_empty() || r.chars().any(|c| c.is_whitespace() || c.is_control()))
    {
        return Err(invalid(
            "Reference must be nonempty with no whitespace or control characters",
        ));
    }
    if value.is_some_and(|v| v.chars().any(char::is_control)) {
        return Err(invalid("Value must be single-line"));
    }
    Ok(())
}

fn invalid(reason: impl Into<String>) -> EngineError {
    EngineError::Validation(reason.into())
}

#[cfg(test)]
use crate::{schematic::Schematic, substrate::ElectricalSelectionSnapshot};

#[cfg(test)]
#[path = "schematic_selection_edit_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "schematic_bound_fields_tests.rs"]
mod bound_fields_tests;

#[cfg(test)]
#[path = "schematic_selection_move_tests.rs"]
mod move_tests;
