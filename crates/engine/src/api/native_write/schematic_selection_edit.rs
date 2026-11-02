//! Occurrence-qualified selected-object edits through canonical schematic writers.
//!
//! Native schemas retain existing compatibility verb IDs. Deletion accepts
//! `{symbol|wire|label: UUID, instance_path: [UUID]}`. `set_symbol_reference`
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
    schematic_symbol_payload::source_symbol_payload,
    schematic_symbols::build_delete_schematic_symbol,
};
use crate::{error::EngineError, substrate::Operation};
#[cfg(test)]
use crate::{schematic::Schematic, substrate::ElectricalSelectionSnapshot};
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
        .any(|instance| instance.placed_symbol_refs.contains(&id))
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
struct SymbolFieldsParams {
    symbol: Uuid,
    instance_path: Vec<Uuid>,
    reference: Option<String>,
    value: Option<String>,
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

fn set_symbol_fields(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: SymbolFieldsParams,
) -> Result<PreparedWrite, EngineError> {
    let (source, sheet) =
        selected_source(context, "symbols", params.symbol, &params.instance_path)?;
    require_unbound_symbol(context, params.symbol)?;
    let mut symbol = source.sheets[&sheet].symbols[&params.symbol].clone();
    if let Some(reference) = params.reference {
        if reference.is_empty()
            || reference
                .chars()
                .any(|c| c.is_whitespace() || c.is_control())
        {
            return Err(invalid(
                "Reference must be nonempty with no whitespace or control characters",
            ));
        }
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
        if value.chars().any(char::is_control) {
            return Err(invalid("Value must be single-line"));
        }
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
        .primary_object(params.symbol)
        .finish()
}

fn invalid(reason: impl Into<String>) -> EngineError {
    EngineError::Validation(reason.into())
}

#[cfg(test)]
#[path = "schematic_selection_edit_tests.rs"]
mod tests;
