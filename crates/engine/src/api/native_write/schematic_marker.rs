//! Point markers share one canonical placement operation and qualified deletion.
use super::{
    context::{PreparedWrite, WriteProvenance},
    registry::{NativeWriteContext, NativeWriteVerb, parse_verb_params},
    schematic_connectivity::{
        build_delete_schematic_junction, build_delete_schematic_noconnect,
        build_place_schematic_marker,
    },
    schematic_edit::prepare_native_birth,
    schematic_selection_edit::selected_source,
};
use crate::{
    error::EngineError,
    ir::geometry::Point,
    schematic::{Junction, NoConnectMarker},
    substrate::{ElectricalSelectionSnapshot, SchematicMarkerKind},
};
use serde::Deserialize;
use uuid::Uuid;

pub(crate) const VERBS: &[NativeWriteVerb] = &[
    NativeWriteVerb {
        id: "datum.schematic.place_marker",
        build: place_marker,
    },
    NativeWriteVerb {
        id: "datum.schematic.place_junction",
        build: place_junction,
    },
    NativeWriteVerb {
        id: "datum.schematic.place_noconnect",
        build: place_noconnect,
    },
    NativeWriteVerb {
        id: "datum.schematic.delete_junction",
        build: delete_junction,
    },
    NativeWriteVerb {
        id: "datum.schematic.delete_noconnect",
        build: delete_noconnect,
    },
];

#[derive(Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Kind {
    Junction,
    Noconnect,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlaceParams {
    sheet: Uuid,
    kind: Option<Kind>,
    x_nm: i64,
    y_nm: i64,
    symbol: Option<Uuid>,
    pin: Option<Uuid>,
}

fn place_marker(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    place(context, provenance, params, None)
}
fn place_junction(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    place(context, provenance, params, Some(Kind::Junction))
}
fn place_noconnect(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    place(context, provenance, params, Some(Kind::Noconnect))
}
fn place(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
    alias: Option<Kind>,
) -> Result<PreparedWrite, EngineError> {
    let params: PlaceParams = parse_verb_params("datum.schematic.place_marker", params)?;
    if alias.is_some() && params.kind.is_some() && alias != params.kind {
        return Err(invalid("Marker kind does not match the placement action"));
    }
    let kind = alias
        .or(params.kind)
        .ok_or_else(|| invalid("Marker kind is required"))?;
    let snapshot = ElectricalSelectionSnapshot::capture(context.model)
        .map_err(|e| invalid(format!("Schematic source unavailable: {e:?}")))?;
    let source = snapshot
        .schematic_source()
        .ok_or_else(|| invalid("Native schematic source unavailable"))?;
    let sheet = source
        .sheets
        .get(&params.sheet)
        .filter(|s| s.uuid == params.sheet)
        .ok_or_else(|| invalid("Native sheet not found"))?;
    let position = Point::new(params.x_nm, params.y_nm);
    let id = Uuid::new_v4();
    let (kind, marker) =
        match kind {
            Kind::Junction => {
                if params.symbol.is_some() || params.pin.is_some() {
                    return Err(invalid(
                        "Junction placement does not accept a symbol or pin",
                    ));
                }
                if sheet.junctions.values().any(|j| j.position == position) {
                    return Err(invalid("A junction already exists at this point"));
                }
                (
                    SchematicMarkerKind::Junction,
                    serde_json::to_value(Junction { uuid: id, position })?,
                )
            }
            Kind::Noconnect => {
                let pins: Vec<_> = sheet
                    .symbols
                    .iter()
                    .flat_map(|(key, symbol)| {
                        symbol
                            .pins
                            .iter()
                            .filter(move |pin| pin.position == position)
                            .map(move |pin| (*key, symbol, pin))
                    })
                    .collect();
                if pins.len() != 1 {
                    return Err(invalid("No-connect requires exactly one pin at this point"));
                }
                let (key, symbol, pin) = pins[0];
                if key != symbol.uuid
                    || symbol.uuid.is_nil()
                    || pin.uuid.is_nil()
                    || source
                        .sheets
                        .values()
                        .flat_map(|s| s.symbols.values())
                        .flat_map(|s| &s.pins)
                        .filter(|p| p.uuid == pin.uuid)
                        .count()
                        != 1
                {
                    return Err(invalid("No-connect pin identity is inconsistent"));
                }
                match (params.symbol, params.pin) {
                    (None, None) => {}
                    (Some(s), Some(p)) if s == symbol.uuid && p == pin.uuid => {}
                    _ => {
                        return Err(invalid(
                            "No-connect symbol and pin must match the pin at this point",
                        ));
                    }
                }
                if sheet.noconnects.values().any(|n| {
                    n.position == position || (n.symbol == symbol.uuid && n.pin == pin.uuid)
                }) {
                    return Err(invalid("A no-connect marker already exists for this pin"));
                }
                (
                    SchematicMarkerKind::NoConnect,
                    serde_json::to_value(NoConnectMarker {
                        uuid: id,
                        symbol: symbol.uuid,
                        pin: pin.uuid,
                        position,
                    })?,
                )
            }
        };
    let prepared =
        build_place_schematic_marker(context.model, provenance, params.sheet, id, kind, marker)?;
    prepare_native_birth(context.model, prepared)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeleteJunction {
    junction: Uuid,
    instance_path: Vec<Uuid>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeleteNoconnect {
    noconnect: Uuid,
    instance_path: Vec<Uuid>,
}

fn delete_junction(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    let p: DeleteJunction = parse_verb_params("datum.schematic.delete_junction", params)?;
    let (source, sheet) = selected_source(context, "junctions", p.junction, &p.instance_path)?;
    build_delete_schematic_junction(
        context.model,
        provenance,
        sheet,
        &source.sheets[&sheet].junctions[&p.junction],
    )
}
fn delete_noconnect(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    let p: DeleteNoconnect = parse_verb_params("datum.schematic.delete_noconnect", params)?;
    let (source, sheet) = selected_source(context, "noconnects", p.noconnect, &p.instance_path)?;
    build_delete_schematic_noconnect(
        context.model,
        provenance,
        sheet,
        &source.sheets[&sheet].noconnects[&p.noconnect],
    )
}
fn invalid(reason: impl Into<String>) -> EngineError {
    EngineError::Validation(reason.into())
}

#[cfg(test)]
#[path = "schematic_marker_tests.rs"]
mod tests;
